use chrono::{NaiveDateTime, Utc};
use diesel::prelude::*;
use diesel::r2d2::{self, ConnectionManager};
use dotenvy::dotenv;
use std::collections::HashMap;
use std::env;

use crate::models::*;
use crate::schema::assignments::dsl as assignments_dsl;
use crate::schema::people::dsl as people_dsl;

pub type DbPool = r2d2::Pool<ConnectionManager<PgConnection>>;

pub fn establish_connection() -> DbPool {
    dotenv().ok();

    let database_url = env::var("DATABASE_URL").expect("DATABASE_URL must be set");
    let manager = ConnectionManager::<PgConnection>::new(database_url);
    r2d2::Pool::builder()
        .build(manager)
        .expect("Failed to create pool.")
}

/// Fetches all active people from the database, separated by group.
pub fn fetch_people(
    conn: &mut PgConnection,
) -> QueryResult<(Vec<String>, Vec<String>, HashMap<String, i32>)> {
    let all_people = people_dsl::people
        .filter(people_dsl::active.eq(true))
        .load::<Person>(conn)?;

    let mut names_a = Vec::new();
    let mut names_b = Vec::new();
    let mut name_to_id = HashMap::new();

    for person in all_people {
        name_to_id.insert(person.name.clone(), person.id);
        if person.group_type == "A" {
            names_a.push(person.name);
        } else {
            names_b.push(person.name);
        }
    }

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
            Ok(days_diff >= 14)
            println!("Days Now: {} ", now );
            println!("Days Date: {} ", date );
            println!("Days Left: {} ", days_diff );
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
