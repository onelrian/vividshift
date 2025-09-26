use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::{Arc, Mutex, OnceLock}};
use uuid::Uuid;

use crate::{
    api::AppState,
    auth::{jwt::Claims, models::UserRole},
    models::{AssignmentRequest, AssignmentResult, GenericEntity},
    services::{EntityManager, ExecutionContext, StrategyConfig},
};

// Simple in-memory history storage for legacy compatibility
static ASSIGNMENT_HISTORY: OnceLock<Arc<Mutex<HashMap<String, serde_json::Value>>>> = OnceLock::new();

fn get_history_storage() -> &'static Arc<Mutex<HashMap<String, serde_json::Value>>> {
    ASSIGNMENT_HISTORY.get_or_init(|| Arc::new(Mutex::new(HashMap::new())))
}

#[derive(Debug, Deserialize)]
pub struct GenerateRequest {
    pub names_a: Vec<String>,
    pub names_b: Vec<String>,
    pub custom_assignments: Option<HashMap<String, usize>>,
}

#[derive(Debug, Serialize)]
pub struct GenerateResponse {
    pub assignments: HashMap<String, Vec<String>>,
    pub generation_info: GenerationInfo,
}

#[derive(Debug, Serialize)]
pub struct GenerationInfo {
    pub attempt_number: u32,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub total_people: usize,
    pub total_assignments: usize,
}

#[derive(Debug, Serialize)]
pub struct HistoryResponse {
    pub history: HashMap<String, serde_json::Value>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignmentsRequest {
    pub assignments: HashMap<String, usize>,
}

/// Generic assignment generation endpoint
pub async fn generate_assignments(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<AssignmentRequest>,
) -> Result<Json<AssignmentResult>, StatusCode> {
    // Get participants and targets from entity manager
    let participants = app_state.entity_manager
        .list_entities("participant")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get participants: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let targets = app_state.entity_manager
        .list_entities("assignment_target")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get targets: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    // Filter entities based on request
    let filtered_participants: Vec<GenericEntity> = if request.participants.is_empty() {
        participants
    } else {
        participants
            .into_iter()
            .filter(|p| request.participants.contains(&p.id))
            .collect()
    };

    let filtered_targets: Vec<GenericEntity> = if request.targets.is_empty() {
        targets
    } else {
        targets
            .into_iter()
            .filter(|t| request.targets.contains(&t.id))
            .collect()
    };

    // Create strategy configuration
    let strategy_config = StrategyConfig {
        name: request.strategy.clone(),
        parameters: request.parameters.unwrap_or_default(),
        constraints: request.constraints.unwrap_or_default(),
        validation_rules: vec![
            "capacity_check".to_string(),
            "availability_check".to_string(),
        ],
    };

    // Create execution context
    let context = ExecutionContext {
        request_id: Uuid::new_v4(),
        user_id: Some(Uuid::parse_str(&claims.sub).map_err(|_| StatusCode::BAD_REQUEST)?),
        domain: app_state.config.domain.name.clone(),
        metadata: HashMap::new(),
    };

    // Execute assignment
    match app_state.rule_engine
        .execute_assignment(
            &request.strategy,
            filtered_participants,
            filtered_targets,
            strategy_config,
            context,
        )
        .await
    {
        Ok(result) => {
            tracing::info!(
                "Assignment generated successfully for user: {} using strategy: {}",
                claims.sub,
                request.strategy
            );
            Ok(Json(result))
        }
        Err(e) => {
            tracing::error!("Assignment generation failed: {}", e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// List entities of a specific type
pub async fn list_entities(
    Path(entity_type): Path<String>,
    State(entity_manager): State<Arc<EntityManager>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<GenericEntity>>, StatusCode> {
    match entity_manager.list_entities(&entity_type).await {
        Ok(entities) => Ok(Json(entities)),
        Err(e) => {
            tracing::error!("Failed to list entities of type {}: {}", entity_type, e);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

/// Create a new entity
pub async fn create_entity(
    Path(entity_type): Path<String>,
    State(entity_manager): State<Arc<EntityManager>>,
    Extension(_claims): Extension<Claims>,
    Json(attributes): Json<HashMap<String, serde_json::Value>>,
) -> Result<Json<GenericEntity>, StatusCode> {
    match entity_manager.create_entity(entity_type, attributes).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => {
            tracing::error!("Failed to create entity: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Get a specific entity
pub async fn get_entity(
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    State(entity_manager): State<Arc<EntityManager>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<GenericEntity>, StatusCode> {
    match entity_manager.get_entity(&entity_type, entity_id).await {
        Ok(entity) => Ok(Json(entity)),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// Update an entity
pub async fn update_entity(
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    State(entity_manager): State<Arc<EntityManager>>,
    Extension(_claims): Extension<Claims>,
    Json(updates): Json<HashMap<String, serde_json::Value>>,
) -> Result<Json<GenericEntity>, StatusCode> {
    match entity_manager.update_entity(&entity_type, entity_id, updates).await {
        Ok(entity) => Ok(Json(entity)),
        Err(e) => {
            tracing::error!("Failed to update entity: {}", e);
            Err(StatusCode::BAD_REQUEST)
        }
    }
}

/// Delete an entity
pub async fn delete_entity(
    Path((entity_type, entity_id)): Path<(String, Uuid)>,
    State(entity_manager): State<Arc<EntityManager>>,
    Extension(_claims): Extension<Claims>,
) -> Result<StatusCode, StatusCode> {
    match entity_manager.delete_entity(&entity_type, entity_id).await {
        Ok(_) => Ok(StatusCode::NO_CONTENT),
        Err(_) => Err(StatusCode::NOT_FOUND),
    }
}

/// List available assignment strategies
pub async fn list_strategies(
    State(app_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let strategies = app_state.rule_engine.list_strategies();
    Ok(Json(strategies))
}

/// List available validation rules
pub async fn list_validators(
    State(app_state): State<AppState>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let validators = app_state.rule_engine.list_validators();
    Ok(Json(validators))
}

/// Legacy work groups endpoint for backward compatibility
pub async fn generate_work_groups_legacy(
    State(app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, StatusCode> {
    // Simple legacy implementation for backward compatibility
    let mut assignments = std::collections::HashMap::new();
    
    // Default assignments if none provided
    let work_assignments = request.custom_assignments.unwrap_or_else(|| {
        let mut default = std::collections::HashMap::new();
        default.insert("Parlor".to_string(), 2);
        default.insert("Kitchen".to_string(), 1);
        default.insert("Bathroom".to_string(), 1);
        default
    });

    // Simple round-robin assignment
    let all_names: Vec<String> = [request.names_a, request.names_b].concat();
    let mut name_index = 0;

    for (task, count) in work_assignments {
        let mut task_assignments = Vec::new();
        for _ in 0..count {
            if name_index < all_names.len() {
                task_assignments.push(all_names[name_index].clone());
                name_index += 1;
            }
        }
        assignments.insert(task, task_assignments);
    }

    let total_people = all_names.len();
    let total_assignments = assignments.values().map(|v| v.len()).sum();

    let generation_info = GenerationInfo {
        attempt_number: 1,
        timestamp: chrono::Utc::now(),
        total_people,
        total_assignments,
    };

    // Store assignment in history (simple in-memory storage for legacy compatibility)
    let history_key = format!("assignment_{}", generation_info.timestamp.timestamp());
    let history_entry = serde_json::json!({
        "assignments": assignments,
        "generation_info": generation_info,
        "user_id": claims.sub
    });

    // Store in global history
    if let Ok(mut history) = get_history_storage().lock() {
        history.insert(history_key.clone(), history_entry);
    }

    tracing::info!(
        "Legacy work groups generated and stored in history for user: {}, key: {}",
        claims.sub, history_key
    );

    Ok(Json(GenerateResponse {
        assignments,
        generation_info,
    }))
}

pub async fn get_history(
    Extension(claims): Extension<Claims>,
) -> Result<Json<HistoryResponse>, StatusCode> {
    tracing::info!("History requested by user: {}", claims.sub);
    
    // Retrieve history from global storage
    let history = if let Ok(stored_history) = get_history_storage().lock() {
        // Filter history for the requesting user or return all for simplicity
        stored_history.clone()
    } else {
        HashMap::new()
    };

    Ok(Json(HistoryResponse {
        history,
        last_updated: Some(chrono::Utc::now()),
    }))
}

pub async fn get_assignments_config(
    State(_app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<HashMap<String, usize>>, StatusCode> {
    // Simple legacy implementation - return default assignments
    tracing::info!("Assignment config requested by user: {}", claims.sub);
    let mut default_assignments = HashMap::new();
    default_assignments.insert("Parlor".to_string(), 2);
    default_assignments.insert("Kitchen".to_string(), 1);
    default_assignments.insert("Bathroom".to_string(), 1);
    Ok(Json(default_assignments))
}

pub async fn update_assignments_config(
    State(_app_state): State<AppState>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<UpdateAssignmentsRequest>,
) -> Result<Json<HashMap<String, usize>>, StatusCode> {
    // Only admins can update assignments
    if claims.role != UserRole::Admin.to_string() {
        return Err(StatusCode::FORBIDDEN);
    }

    // In a real application, you'd persist this to database
    // For now, we'll just return the updated assignments
    tracing::info!(
        "Admin {} updated work assignments: {:?}",
        claims.sub,
        request.assignments
    );

    Ok(Json(request.assignments))
}
