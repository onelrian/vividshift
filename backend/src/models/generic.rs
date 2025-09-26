use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;
use validator::Validate;

/// Generic entity that can represent any domain object
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenericEntity {
    pub id: Uuid,
    pub entity_type: String,
    pub attributes: HashMap<String, serde_json::Value>,
    pub metadata: EntityMetadata,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

/// Metadata for entity management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityMetadata {
    pub version: String,
    pub created_by: Option<Uuid>,
    pub updated_by: Option<Uuid>,
    pub tags: Vec<String>,
    pub status: EntityStatus,
}

/// Entity lifecycle status
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EntityStatus {
    Active,
    Inactive,
    Archived,
    Draft,
}

/// Definition of an entity type with its schema
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityDefinition {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub fields: HashMap<String, FieldDefinition>,
    pub relationships: HashMap<String, RelationshipDefinition>,
    pub constraints: Vec<EntityConstraint>,
}

/// Field definition with validation rules
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldDefinition {
    pub field_type: FieldType,
    pub required: bool,
    pub default_value: Option<serde_json::Value>,
    pub constraints: Option<FieldConstraints>,
    pub display_name: String,
    pub description: Option<String>,
}

/// Supported field types
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum FieldType {
    String,
    Integer,
    Float,
    Boolean,
    Array,
    Object,
    DateTime,
    Uuid,
}

/// Field-level constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FieldConstraints {
    pub min: Option<i64>,
    pub max: Option<i64>,
    pub min_length: Option<usize>,
    pub max_length: Option<usize>,
    pub pattern: Option<String>,
    pub allowed_values: Option<Vec<serde_json::Value>>,
    pub unique: Option<bool>,
}

/// Relationship between entities
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RelationshipDefinition {
    pub target_entity: String,
    pub relationship_type: RelationshipType,
    pub cardinality: Cardinality,
    pub required: bool,
}

/// Types of relationships
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum RelationshipType {
    OneToOne,
    OneToMany,
    ManyToMany,
    Hierarchical,
}

/// Relationship cardinality
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum Cardinality {
    ZeroOrOne,
    ExactlyOne,
    ZeroOrMany,
    OneOrMany,
}

/// Entity-level constraints
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EntityConstraint {
    pub name: String,
    pub constraint_type: ConstraintType,
    pub fields: Vec<String>,
    pub rule: String,
    pub error_message: String,
}

/// Types of entity constraints
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ConstraintType {
    Unique,
    Check,
    Custom,
}

/// Assignment request structure
#[derive(Debug, Deserialize, Validate)]
pub struct AssignmentRequest {
    pub strategy: String,
    pub participants: Vec<Uuid>,
    pub targets: Vec<Uuid>,
    pub parameters: Option<HashMap<String, serde_json::Value>>,
    pub constraints: Option<Vec<AssignmentConstraint>>,
}

/// Assignment constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentConstraint {
    pub name: String,
    pub constraint_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
}

/// Assignment result
#[derive(Debug, Serialize)]
pub struct AssignmentResult {
    pub id: Uuid,
    pub strategy_used: String,
    pub assignments: Vec<Assignment>,
    pub metadata: AssignmentMetadata,
    pub created_at: DateTime<Utc>,
}

/// Individual assignment
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Assignment {
    pub participant_id: Uuid,
    pub target_id: Uuid,
    pub confidence: f64,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Assignment execution metadata
#[derive(Debug, Serialize)]
pub struct AssignmentMetadata {
    pub attempts: u32,
    pub execution_time_ms: u64,
    pub strategy_parameters: HashMap<String, serde_json::Value>,
    pub validation_results: Vec<ValidationResult>,
}

/// Validation result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationResult {
    pub rule_name: String,
    pub passed: bool,
    pub message: Option<String>,
    pub severity: ValidationSeverity,
}

/// Validation severity levels
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum ValidationSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

impl GenericEntity {
    pub fn new(entity_type: String, attributes: HashMap<String, serde_json::Value>) -> Self {
        let now = Utc::now();
        Self {
            id: Uuid::new_v4(),
            entity_type,
            attributes,
            metadata: EntityMetadata {
                version: "1.0".to_string(),
                created_by: None,
                updated_by: None,
                tags: Vec::new(),
                status: EntityStatus::Active,
            },
            created_at: now,
            updated_at: now,
        }
    }

    pub fn get_attribute<T>(&self, key: &str) -> Option<T>
    where
        T: for<'de> Deserialize<'de>,
    {
        self.attributes
            .get(key)
            .and_then(|v| serde_json::from_value(v.clone()).ok())
    }

    pub fn set_attribute<T>(&mut self, key: String, value: T) -> Result<(), serde_json::Error>
    where
        T: Serialize,
    {
        let json_value = serde_json::to_value(value)?;
        self.attributes.insert(key, json_value);
        self.updated_at = Utc::now();
        Ok(())
    }
}

impl Default for EntityStatus {
    fn default() -> Self {
        EntityStatus::Active
    }
}
