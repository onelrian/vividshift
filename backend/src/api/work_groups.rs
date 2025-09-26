use axum::{
    extract::{Extension, Path, State},
    http::StatusCode,
    Json,
};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, sync::Arc};
use uuid::Uuid;

use crate::{
    auth::{jwt::Claims, models::UserRole},
    config::AppConfig,
    models::{AssignmentRequest, AssignmentResult, GenericEntity},
    services::{EntityManager, RuleEngine, ExecutionContext, StrategyConfig},
    // Legacy imports
    group, history,
};

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
    pub history: HashMap<String, Vec<String>>,
    pub last_updated: Option<chrono::DateTime<chrono::Utc>>,
}

#[derive(Debug, Deserialize)]
pub struct UpdateAssignmentsRequest {
    pub assignments: HashMap<String, usize>,
}

/// Generic assignment generation endpoint
pub async fn generate_assignments(
    State(rule_engine): State<Arc<RuleEngine>>,
    State(entity_manager): State<Arc<EntityManager>>,
    State(config): State<Arc<AppConfig>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<AssignmentRequest>,
) -> Result<Json<AssignmentResult>, StatusCode> {
    // Get participants and targets from entity manager
    let participants = entity_manager
        .list_entities("participant")
        .await
        .map_err(|e| {
            tracing::error!("Failed to get participants: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?;

    let targets = entity_manager
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
        domain: config.domain.name.clone(),
        metadata: HashMap::new(),
    };

    // Execute assignment
    match rule_engine
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
    State(rule_engine): State<Arc<RuleEngine>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let strategies = rule_engine.list_strategies();
    Ok(Json(strategies))
}

/// List available validation rules
pub async fn list_validators(
    State(rule_engine): State<Arc<RuleEngine>>,
    Extension(_claims): Extension<Claims>,
) -> Result<Json<Vec<String>>, StatusCode> {
    let validators = rule_engine.list_validators();
    Ok(Json(validators))
}

/// Legacy work groups endpoint for backward compatibility
pub async fn generate_work_groups_legacy(
    State(config): State<Arc<AppConfig>>,
    Extension(claims): Extension<Claims>,
    Json(request): Json<GenerateRequest>,
) -> Result<Json<GenerateResponse>, StatusCode> {
    // Load existing history
    let history = history::load_history().map_err(|_| StatusCode::INTERNAL_SERVER_ERROR)?;

    // Use custom assignments if provided, otherwise use config
    let work_assignments = request
        .custom_assignments
        .unwrap_or_else(|| config.work_assignments.assignments.clone());

    // Generate work distribution with retry logic
    let mut final_assignments = None;
    let mut attempt_number = 0;

    for attempt in 1..=config.work_assignments.max_attempts {
        attempt_number = attempt;
        match group::distribute_work(&request.names_a, &request.names_b, &work_assignments, &history) {
            Ok(new_assignments) => {
                final_assignments = Some(new_assignments);
                break;
            }
            Err(_) => continue,
        }
    }

    match final_assignments {
        Some(assignments) => {
            // Save updated history
            if let Err(_) = history::save_history(&assignments, &history) {
                tracing::error!("Failed to save assignment history for user: {}", claims.sub);
            }

            let total_people = request.names_a.len() + request.names_b.len();
            let total_assignments = assignments.values().map(|v| v.len()).sum();

            Ok(Json(GenerateResponse {
                assignments,
                generation_info: GenerationInfo {
                    attempt_number,
                    timestamp: chrono::Utc::now(),
                    total_people,
                    total_assignments,
                },
            }))
        }
        None => {
            tracing::error!(
                "Failed to generate work groups after {} attempts for user: {}",
                config.work_assignments.max_attempts,
                claims.sub
            );
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_history(
    Extension(claims): Extension<Claims>,
) -> Result<Json<HistoryResponse>, StatusCode> {
    match history::load_history() {
        Ok(history) => Ok(Json(HistoryResponse {
            history,
            last_updated: Some(chrono::Utc::now()), // In real app, get from file metadata
        })),
        Err(_) => {
            tracing::error!("Failed to load history for user: {}", claims.sub);
            Err(StatusCode::INTERNAL_SERVER_ERROR)
        }
    }
}

pub async fn get_assignments_config(
    State(config): State<Arc<AppConfig>>,
    Extension(claims): Extension<Claims>,
) -> Result<Json<HashMap<String, usize>>, StatusCode> {
    Ok(Json(config.work_assignments.assignments.clone()))
}

pub async fn update_assignments_config(
    State(config): State<Arc<AppConfig>>,
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
