use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Validation error types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    FieldRequired { field: String, entity_type: String },
    FieldInvalid { field: String, value: String, reason: String },
    ConstraintViolation { constraint: String, message: String },
    EntityNotFound { entity_id: Uuid, entity_type: String },
    BusinessRuleViolation { rule: String, message: String },
}

/// Field validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldValidationResult {
    pub field_name: String,
    pub valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Entity validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityValidationResult {
    pub entity_id: Uuid,
    pub entity_type: String,
    pub valid: bool,
    pub field_results: Vec<FieldValidationResult>,
    pub constraint_violations: Vec<String>,
}

/// Assignment validation context
#[derive(Debug, Clone)]
pub struct ValidationContext {
    pub domain_config: crate::models::DomainConfig,
    pub existing_assignments: Vec<crate::models::Assignment>,
    pub historical_data: HashMap<Uuid, Vec<String>>,
    pub metadata: HashMap<String, serde_json::Value>,
}

impl ValidationError {
    pub fn severity(&self) -> crate::models::ValidationSeverity {
        match self {
            ValidationError::FieldRequired { .. } => crate::models::ValidationSeverity::Error,
            ValidationError::FieldInvalid { .. } => crate::models::ValidationSeverity::Error,
            ValidationError::ConstraintViolation { .. } => crate::models::ValidationSeverity::Warning,
            ValidationError::EntityNotFound { .. } => crate::models::ValidationSeverity::Critical,
            ValidationError::BusinessRuleViolation { .. } => crate::models::ValidationSeverity::Error,
        }
    }

    pub fn message(&self) -> String {
        match self {
            ValidationError::FieldRequired { field, entity_type } => {
                format!("Required field '{}' is missing for entity type '{}'", field, entity_type)
            }
            ValidationError::FieldInvalid { field, value, reason } => {
                format!("Field '{}' has invalid value '{}': {}", field, value, reason)
            }
            ValidationError::ConstraintViolation { constraint, message } => {
                format!("Constraint '{}' violated: {}", constraint, message)
            }
            ValidationError::EntityNotFound { entity_id, entity_type } => {
                format!("Entity '{}' of type '{}' not found", entity_id, entity_type)
            }
            ValidationError::BusinessRuleViolation { rule, message } => {
                format!("Business rule '{}' violated: {}", rule, message)
            }
        }
    }
}
