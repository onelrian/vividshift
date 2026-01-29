use axum::{
    extract::{State, Json, Path},
    http::StatusCode,
};
use serde::Serialize;
use std::collections::HashMap;
use crate::api::AppState;
use crate::api::auth::User;
use crate::models::{Person, Assignment, NewPerson, UpdatePerson, Setting};
use crate::assignment_engine;
use diesel::prelude::*;
use chrono::{NaiveDateTime, Utc};

#[derive(Serialize)]
pub struct DashboardData {
    pub people: Vec<Person>,
    pub recent_assignments: Vec<Assignment>,
    pub next_shuffle_in_days: i64,
}

pub async fn get_dashboard_data(
    User(_claims): User,
    State(state): State<AppState>,
) -> Result<Json<DashboardData>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 1. Fetch active people
    let people = crate::schema::people::table
        .filter(crate::schema::people::active.eq(true))
        .load::<Person>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 2. Fetch recent assignments
    let recent_assignments = crate::schema::assignments::table
        .order(crate::schema::assignments::assigned_at.desc())
        .limit(10)
        .load::<Assignment>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3. Calculate next shuffle using settings
    use diesel::dsl::max;
    let last_run: Option<NaiveDateTime> = crate::schema::assignments::table
        .select(max(crate::schema::assignments::assigned_at))
        .first(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // 3. Calculate next shuffle using settings from DB
    let db_settings = crate::db::fetch_db_settings(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    let interval_days = db_settings.get("assignment_interval_days")
        .and_then(|v| v.parse::<i64>().ok())
        .unwrap_or(14);

    let next_shuffle_in_days = match last_run {
        Some(date) => {
            let now = Utc::now().naive_utc();
            let days_diff = (now - date).num_days();
            (interval_days - days_diff).max(0)
        }
        None => 0,
    };

    Ok(Json(DashboardData {
        people,
        recent_assignments,
        next_shuffle_in_days,
    }))
}

#[derive(Serialize)]
pub struct ShuffleResponse {
    pub success: bool,
    pub message: String,
}


pub async fn trigger_shuffle(
    User(_claims): User,
    State(state): State<AppState>,
) -> Result<Json<ShuffleResponse>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    match assignment_engine::perform_distribution(&mut conn, &state.settings, true).await {
        Ok(run) => {
            Ok(Json(ShuffleResponse {
                success: true,
                message: if run { "Shuffle completed successfully".to_string() } else { "Shuffle skipped (already completed)".to_string() },
            }))
        }
        Err(e) => {
            tracing::error!("Shuffle failed: {}", e);
            Ok(Json(ShuffleResponse {
                success: false,
                message: format!("Shuffle failed: {}", e),
            }))
        }
    }
}

// --- People Management ---

pub async fn list_people(
    User(_claims): User,
    State(state): State<AppState>,
) -> Result<Json<Vec<Person>>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    crate::schema::people::table
        .order(crate::schema::people::name.asc())
        .load::<Person>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn create_person(
    User(_claims): User,
    State(state): State<AppState>,
    Json(payload): Json<NewPerson>,
) -> Result<Json<Person>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    diesel::insert_into(crate::schema::people::table)
        .values(&payload)
        .get_result::<Person>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn update_person(
    User(_claims): User,
    State(state): State<AppState>,
    Path(id): Path<i32>,
    Json(payload): Json<UpdatePerson>,
) -> Result<Json<Person>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    diesel::update(crate::schema::people::table.find(id))
        .set(&payload)
        .get_result::<Person>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn delete_person(
    User(_claims): User,
    State(state): State<AppState>,
    Path(id): Path<i32>,
) -> Result<StatusCode, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    diesel::delete(crate::schema::people::table.find(id))
        .execute(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
        
    Ok(StatusCode::NO_CONTENT)
}

// --- Assignment History ---

pub async fn get_assignment_history(
    User(_claims): User,
    State(state): State<AppState>,
) -> Result<Json<Vec<Assignment>>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    crate::schema::assignments::table
        .order(crate::schema::assignments::assigned_at.desc())
        .load::<Assignment>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

// --- Settings Management ---

pub async fn get_app_settings(
    User(_claims): User,
    State(state): State<AppState>,
) -> Result<Json<HashMap<String, String>>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    crate::db::fetch_db_settings(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}

pub async fn update_app_setting(
    User(_claims): User,
    State(state): State<AppState>,
    Json(payload): Json<Setting>,
) -> Result<Json<Setting>, StatusCode> {
    let mut conn = state.db_pool.get().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;
    
    diesel::insert_into(crate::schema::settings::table)
        .values(&payload)
        .on_conflict(crate::schema::settings::key)
        .do_update()
        .set(crate::schema::settings::value.eq(&payload.value))
        .get_result::<Setting>(&mut conn)
        .map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)
        .map(Json)
}
