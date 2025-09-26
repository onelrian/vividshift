use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use uuid::Uuid;

/// Domain configuration that defines the business context
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainConfig {
    pub name: String,
    pub display_name: String,
    pub description: String,
    pub version: String,
    pub entities: HashMap<String, super::EntityDefinition>,
    pub default_data: Option<HashMap<String, serde_json::Value>>,
    pub business_rules: Vec<BusinessRule>,
}

/// Business rule definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BusinessRule {
    pub name: String,
    pub description: String,
    pub rule_type: BusinessRuleType,
    pub condition: String,
    pub action: String,
    pub priority: i32,
    pub enabled: bool,
}

/// Types of business rules
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum BusinessRuleType {
    Validation,
    Assignment,
    Constraint,
    Transformation,
}

/// Work group specific domain models (backward compatibility)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkGroupDomain {
    pub assignment_targets: Vec<AssignmentTarget>,
    pub participants: Vec<Participant>,
    pub assignment_rules: AssignmentRules,
}

/// Assignment target (e.g., work location, task)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentTarget {
    pub id: Uuid,
    pub name: String,
    pub required_count: u32,
    pub priority: Option<u32>,
    pub constraints: Vec<String>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Participant (e.g., person, team member)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Participant {
    pub id: Uuid,
    pub name: String,
    pub group: Option<String>,
    pub skills: Vec<String>,
    pub availability: bool,
    pub preferences: HashMap<String, serde_json::Value>,
    pub metadata: HashMap<String, serde_json::Value>,
}

/// Assignment rules configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentRules {
    pub strategy: String,
    pub max_attempts: u32,
    pub rotation_weight: f64,
    pub balance_weight: f64,
    pub constraints: Vec<AssignmentRuleConstraint>,
}

/// Assignment rule constraint
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AssignmentRuleConstraint {
    pub name: String,
    pub constraint_type: String,
    pub parameters: HashMap<String, serde_json::Value>,
    pub enabled: bool,
}

impl From<AssignmentTarget> for super::GenericEntity {
    fn from(target: AssignmentTarget) -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), serde_json::json!(target.name));
        attributes.insert("required_count".to_string(), serde_json::json!(target.required_count));
        
        if let Some(priority) = target.priority {
            attributes.insert("priority".to_string(), serde_json::json!(priority));
        }
        
        attributes.insert("constraints".to_string(), serde_json::json!(target.constraints));
        
        // Merge metadata
        for (key, value) in target.metadata {
            attributes.insert(key, value);
        }

        let mut entity = super::GenericEntity::new("assignment_target".to_string(), attributes);
        entity.id = target.id;
        entity
    }
}

impl From<Participant> for super::GenericEntity {
    fn from(participant: Participant) -> Self {
        let mut attributes = HashMap::new();
        attributes.insert("name".to_string(), serde_json::json!(participant.name));
        
        if let Some(group) = participant.group {
            attributes.insert("group".to_string(), serde_json::json!(group));
        }
        
        attributes.insert("skills".to_string(), serde_json::json!(participant.skills));
        attributes.insert("availability".to_string(), serde_json::json!(participant.availability));
        
        // Merge preferences and metadata
        for (key, value) in participant.preferences {
            attributes.insert(format!("pref_{}", key), value);
        }
        
        for (key, value) in participant.metadata {
            attributes.insert(key, value);
        }

        let mut entity = super::GenericEntity::new("participant".to_string(), attributes);
        entity.id = participant.id;
        entity
    }
}

impl TryFrom<super::GenericEntity> for AssignmentTarget {
    type Error = serde_json::Error;

    fn try_from(entity: super::GenericEntity) -> Result<Self, Self::Error> {
        Ok(AssignmentTarget {
            id: entity.id,
            name: entity.get_attribute("name").unwrap_or_default(),
            required_count: entity.get_attribute("required_count").unwrap_or(1),
            priority: entity.get_attribute("priority"),
            constraints: entity.get_attribute("constraints").unwrap_or_default(),
            metadata: entity.attributes,
        })
    }
}

impl TryFrom<super::GenericEntity> for Participant {
    type Error = serde_json::Error;

    fn try_from(entity: super::GenericEntity) -> Result<Self, Self::Error> {
        let mut preferences = HashMap::new();
        let mut metadata = HashMap::new();

        // Separate preferences from other attributes
        for (key, value) in entity.attributes {
            if key.starts_with("pref_") {
                preferences.insert(key.strip_prefix("pref_").unwrap().to_string(), value);
            } else if !["name", "group", "skills", "availability"].contains(&key.as_str()) {
                metadata.insert(key, value);
            }
        }

        Ok(Participant {
            id: entity.id,
            name: entity.get_attribute("name").unwrap_or_default(),
            group: entity.get_attribute("group"),
            skills: entity.get_attribute("skills").unwrap_or_default(),
            availability: entity.get_attribute("availability").unwrap_or(true),
            preferences,
            metadata,
        })
    }
}
