use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use std::collections::HashMap;

use crate::models::*;
use crate::schema::assignments::dsl as assignments_dsl;
use crate::schema::people::dsl as people_dsl;
use tracing::info;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection(database_url: &str) -> DbPool {
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

/// Fetches all active people from the database, separated by group.
/// Uses people.toml as the source of truth for group membership and active status.
pub fn fetch_people(
    conn: &mut PgConnection,
) -> QueryResult<(Vec<String>, Vec<String>, HashMap<String, i32>)> {
    use crate::people_config::PeopleConfiguration;
    use tracing::warn;

    // Load configuration from people.toml
    let config = PeopleConfiguration::load().map_err(|e| {
        diesel::result::Error::DatabaseError(
            diesel::result::DatabaseErrorKind::Unknown,
            Box::new(format!("Failed to load people configuration: {}", e)),
        )
    })?;

    // Fetch all people from database to get their IDs
    let all_db_people = people_dsl::people.load::<Person>(conn)?;

    let mut names_a = Vec::new();
    let mut names_b = Vec::new();
    let mut name_to_id = HashMap::new();

    // Build name-to-id mapping from database
    let db_name_to_id: HashMap<String, i32> = all_db_people
        .iter()
        .map(|p| (p.name.clone(), p.id))
        .collect();

    // Use config as source of truth for active people and groups
    for person_config in config.get_active_people() {
        if let Some(&person_id) = db_name_to_id.get(&person_config.name) {
            name_to_id.insert(person_config.name.clone(), person_id);

            if person_config.group == "A" {
                names_a.push(person_config.name.clone());
            } else if person_config.group == "B" {
                names_b.push(person_config.name.clone());
            } else {
                warn!(
                    "Person '{}' has unknown group '{}', skipping",
                    person_config.name, person_config.group
                );
            }
        } else {
            warn!(
                "Person '{}' from config not found in database, skipping",
                person_config.name
            );
        }
    }

    info!(
        "Loaded {} people from config (Group A: {}, Group B: {})",
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
            if entry.len() < 5 {
                entry.push(assignment.task_name);
            }
        }
    }

    Ok(history_map)
}

/// Checks if it has been 14 days since the last assignment run.
pub fn should_run(conn: &mut PgConnection) -> QueryResult<bool> {
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
            info!("Days Left: {} ", days_diff);
            Ok(days_diff >= 14)
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
