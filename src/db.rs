use anyhow::Context;
use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::collections::HashMap;

use crate::models::*;
use crate::schema::assignments::dsl as assignments_dsl;
use crate::schema::people::dsl as people_dsl;
use tracing::info;

use diesel_migrations::{embed_migrations, EmbeddedMigrations, MigrationHarness};

pub const MIGRATIONS: EmbeddedMigrations = embed_migrations!();

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub async fn init_admin_user(pool: &DbPool, settings: &crate::config::Settings) -> anyhow::Result<()> {
    use crate::schema::users::dsl::*;
    use uuid::Uuid;
    use crate::models::NewUser;
    
    let mut conn = pool.get().context("Failed to get DB connection for admin init")?;
    
    let admin_email_value = &settings.admin_email;
        
    // Extract username from email (part before @)
    let admin_username_value = admin_email_value
        .split('@')
        .next()
        .unwrap_or("admin")
        .to_string();
    
    // Check if admin already exists in database by email OR username
    let existing = users
        .filter(email.eq(admin_email_value).or(username.eq(&admin_username_value)))
        .first::<UserRole>(&mut conn)
        .optional()?;

    if let Some(user_role) = existing {
        info!("👤 Admin user exists. Ensuring credentials are up to date...");
        
        let hashed_pwd = crate::auth::hash_password(&settings.admin_password)
            .context("Failed to hash admin password")?;
            
        diesel::update(users.find(&user_role.id))
            .set((
                email.eq(admin_email_value),
                password_hash.eq(hashed_pwd),
                role.eq("ADMIN"),
            ))
            .execute(&mut conn)
            .context("Failed to update admin credentials")?;
            
        info!("✅ Admin credentials updated.");
    } else {
        info!("👤 Initializing default admin user (local auth)...");
        
        // Hash the admin password
        let hashed_pwd = crate::auth::hash_password(&settings.admin_password)
            .context("Failed to hash admin password")?;
        
        // Generate a new UUID for the admin
        let new_user_id = Uuid::new_v4().to_string();
            
        // Create corresponding database record
        let new_admin = NewUser {
            id: new_user_id.clone(),
            username: admin_username_value,
            email: admin_email_value.to_string(),
            role: "ADMIN".to_string(),
            password_hash: match Some(hashed_pwd) { Some(h) => h, None => return Err(anyhow::anyhow!("Hash failed")) },
        };

        // Use INSERT ... ON CONFLICT (email) DO UPDATE to ensure the admin from .env exists and is correct
        // We use email as the primary key for conflict resolution here
        let rows = diesel::insert_into(users)
            .values(&new_admin)
            .on_conflict(email) 
            .do_update()
            .set((
                password_hash.eq(&new_admin.password_hash),
                role.eq("ADMIN"),
                username.eq(&new_admin.username),
            ))
            .execute(&mut conn)
            .context("Failed to initialize default admin user in database")?;
            
        if rows > 0 {
            info!("✅ Admin user '{}' initialized successfully.", admin_email_value);
        }
    }

    Ok(())
}

/// Checks if the given email matches the default admin email from settings
pub fn is_default_admin(email: &str, settings: &crate::config::Settings) -> bool {
    email == settings.admin_email
}



pub fn establish_connection(database_url: &str) -> anyhow::Result<DbPool> {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    let pool = r2d2::Pool::builder()
        .build(manager)
        .context("Failed to create database pool")?;
    
    // Run migrations on startup
    if let Ok(mut conn) = pool.get() {
        if let Err(e) = conn.run_pending_migrations(MIGRATIONS) {
            tracing::error!("Failed to run database migrations: {}", e);
        } else {
            tracing::info!("Database migrations applied successfully.");
        }
    }

    Ok(pool)
}

pub fn sync_people(conn: &mut PgConnection) -> anyhow::Result<()> {
    use crate::people_config::PeopleConfiguration;
    
    let config = PeopleConfiguration::load().context("Failed to load people.toml")?;
    info!("🔄 Syncing {} people from people.toml to database...", config.people.len());
    
    // 1. Mark all current people as inactive (we will reactivate those found in the TOML)
    diesel::update(people_dsl::people)
        .set(people_dsl::active.eq(false))
        .execute(conn)?;
        
    for p in config.people {
        let existing = people_dsl::people
            .filter(people_dsl::name.eq(&p.name))
            .first::<Person>(conn)
            .optional()?;
            
        if let Some(person) = existing {
            diesel::update(people_dsl::people.find(person.id))
                .set((
                    people_dsl::group_type.eq(&p.group),
                    people_dsl::active.eq(p.active),
                ))
                .execute(conn)?;
        } else {
            diesel::insert_into(people_dsl::people)
                .values((
                    people_dsl::name.eq(&p.name),
                    people_dsl::group_type.eq(&p.group),
                    people_dsl::active.eq(p.active),
                ))
                .execute(conn)?;
        }
    }
    
    Ok(())
}

/// Fetches all active people from the database, separated by group.
/// Uses people.toml as the source of truth for group membership and active status.
pub fn fetch_people(
    conn: &mut PgConnection,
) -> QueryResult<(Vec<String>, Vec<String>, HashMap<String, i32>)> {
    // Fetch active people directly from database
    let active_people = people_dsl::people
        .filter(people_dsl::active.eq(true))
        .load::<Person>(conn)?;

    let mut names_a = Vec::new();
    let mut names_b = Vec::new();
    let mut name_to_id = HashMap::new();

    for person in active_people {
        name_to_id.insert(person.name.clone(), person.id);

        match person.group_type.as_str() {
            "A" => names_a.push(person.name),
            "B" => names_b.push(person.name),
            _ => {
                tracing::warn!(
                    "Person '{}' (ID: {}) has unknown group '{}', skipping for distribution",
                    person.name, person.id, person.group_type
                );
            }
        }
    }

    info!(
        "Loaded {} active people from database (Group A: {}, Group B: {})",
        names_a.len() + names_b.len(),
        names_a.len(),
        names_b.len()
    );

    Ok((names_a, names_b, name_to_id))
}

/// Fetches the recent history for all people.
/// Returns a HashMap where key is person's name and value is list of recent tasks.
pub fn fetch_history(
    conn: &mut PgConnection,
    name_to_id: &HashMap<String, i32>,
) -> QueryResult<HashMap<String, Vec<String>>> {
    let mut history_map = HashMap::new();

    // Get all assignments, ordered by date desc
    // Ideally we'd limit this per person, but for simplicity we can fetch recent ones globally
    // or just fetch all and filter in memory if the dataset is small.
    // Given the context, let's fetch the last 50 assignments per person roughly.
    // Actually, let's just fetch all assignments for now as the dataset seems small (household chores).

    let all_assignments = assignments_dsl::assignments
        .order(assignments_dsl::assigned_at.desc())
        .load::<Assignment>(conn)?;

    // Create a reverse lookup for IDs to Names
    let id_to_name: HashMap<i32, String> =
        name_to_id.iter().map(|(n, i)| (*i, n.clone())).collect();

    for assignment in all_assignments {
        if let Some(name) = id_to_name.get(&assignment.person_id) {
            let entry = history_map.entry(name.clone()).or_insert_with(Vec::new);
            // We only care about the last few assignments for the logic
            if entry.len() < 2 {
                entry.push(assignment.task_name);
            }
        }
    }

    Ok(history_map)
}

/// Checks if enough time has passed since the last assignment run.
/// Uses the configured interval_days instead of a hardcoded value.
pub fn should_run(conn: &mut PgConnection, interval_days: i64) -> QueryResult<bool> {
    use diesel::dsl::max;

    let last_run: Option<NaiveDateTime> = assignments_dsl::assignments
        .select(max(assignments_dsl::assigned_at))
        .first(conn)?;

    match last_run {
        Some(date) => {
            let now = Utc::now().naive_utc();
            let days_diff = (now - date).num_days();
            info!("Days Now: {} ", now);
            info!("Days Date: {} ", date);
            info!("Days since last run: {} (interval: {})", days_diff, interval_days);
            Ok(days_diff >= interval_days)
        }
        None => Ok(true), // No history, so we should run
    }
}

pub fn save_assignments(
    conn: &mut PgConnection,
    assignments: &HashMap<String, Vec<String>>,
    name_to_id: &HashMap<String, i32>,
) -> QueryResult<()> {
    let now = Utc::now().naive_utc();

    for (task, people_names) in assignments {
        for name in people_names {
            if let Some(&person_id) = name_to_id.get(name) {
                let new_assign = NewAssignment {
                    person_id,
                    task_name: task,
                    assigned_at: now,
                };

                diesel::insert_into(assignments_dsl::assignments)
                    .values(&new_assign)
                    .execute(conn)?;
            }
        }
    }
    Ok(())
}

pub fn fetch_db_settings(conn: &mut PgConnection) -> QueryResult<HashMap<String, String>> {
    let settings = crate::schema::settings::table
        .load::<Setting>(conn)?;
    
    Ok(settings.into_iter().map(|s| (s.key, s.value)).collect())
}
