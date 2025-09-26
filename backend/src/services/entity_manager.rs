use anyhow::{anyhow, Result};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

use crate::models::{
    DomainConfig, EntityDefinition, GenericEntity, EntityStatus, ValidationError,
    EntityValidationResult, FieldValidationResult,
};

/// Generic entity manager for CRUD operations
pub struct EntityManager {
    entities: Arc<RwLock<HashMap<String, HashMap<Uuid, GenericEntity>>>>,
    domain_config: DomainConfig,
}

impl EntityManager {
    pub fn new(domain_config: DomainConfig) -> Self {
        Self {
            entities: Arc::new(RwLock::new(HashMap::new())),
            domain_config,
        }
    }

    /// Create a new entity
    pub async fn create_entity(
        &self,
        entity_type: String,
        attributes: HashMap<String, serde_json::Value>,
    ) -> Result<GenericEntity> {
        // Validate entity type exists
        let entity_def = self
            .domain_config
            .entities
            .get(&entity_type)
            .ok_or_else(|| anyhow!("Entity type '{}' not defined", entity_type))?;

        // Validate attributes
        self.validate_entity_attributes(&entity_type, &attributes, entity_def)?;

        // Create entity
        let entity = GenericEntity::new(entity_type.clone(), attributes);

        // Store entity
        let mut entities = self.entities.write().await;
        let type_entities = entities.entry(entity_type).or_insert_with(HashMap::new);
        type_entities.insert(entity.id, entity.clone());

        tracing::info!("Created entity {} of type {}", entity.id, entity.entity_type);
        Ok(entity)
    }

    /// Get entity by ID and type
    pub async fn get_entity(&self, entity_type: &str, entity_id: Uuid) -> Result<GenericEntity> {
        let entities = self.entities.read().await;
        let type_entities = entities
            .get(entity_type)
            .ok_or_else(|| anyhow!("No entities of type '{}'", entity_type))?;

        type_entities
            .get(&entity_id)
            .cloned()
            .ok_or_else(|| anyhow!("Entity {} not found", entity_id))
    }

    /// List entities by type
    pub async fn list_entities(&self, entity_type: &str) -> Result<Vec<GenericEntity>> {
        let entities = self.entities.read().await;
        let type_entities = entities.get(entity_type).unwrap_or(&HashMap::new());
        
        Ok(type_entities
            .values()
            .filter(|e| e.metadata.status == EntityStatus::Active)
            .cloned()
            .collect())
    }

    /// Update entity
    pub async fn update_entity(
        &self,
        entity_type: &str,
        entity_id: Uuid,
        updates: HashMap<String, serde_json::Value>,
    ) -> Result<GenericEntity> {
        // Validate entity type exists
        let entity_def = self
            .domain_config
            .entities
            .get(entity_type)
            .ok_or_else(|| anyhow!("Entity type '{}' not defined", entity_type))?;

        let mut entities = self.entities.write().await;
        let type_entities = entities
            .get_mut(entity_type)
            .ok_or_else(|| anyhow!("No entities of type '{}'", entity_type))?;

        let entity = type_entities
            .get_mut(&entity_id)
            .ok_or_else(|| anyhow!("Entity {} not found", entity_id))?;

        // Validate updates
        self.validate_entity_attributes(entity_type, &updates, entity_def)?;

        // Apply updates
        for (key, value) in updates {
            entity.attributes.insert(key, value);
        }
        entity.updated_at = chrono::Utc::now();

        tracing::info!("Updated entity {} of type {}", entity_id, entity_type);
        Ok(entity.clone())
    }

    /// Delete entity (soft delete)
    pub async fn delete_entity(&self, entity_type: &str, entity_id: Uuid) -> Result<()> {
        let mut entities = self.entities.write().await;
        let type_entities = entities
            .get_mut(entity_type)
            .ok_or_else(|| anyhow!("No entities of type '{}'", entity_type))?;

        let entity = type_entities
            .get_mut(&entity_id)
            .ok_or_else(|| anyhow!("Entity {} not found", entity_id))?;

        entity.metadata.status = EntityStatus::Archived;
        entity.updated_at = chrono::Utc::now();

        tracing::info!("Deleted entity {} of type {}", entity_id, entity_type);
        Ok(())
    }

    /// Validate entity attributes against schema
    fn validate_entity_attributes(
        &self,
        entity_type: &str,
        attributes: &HashMap<String, serde_json::Value>,
        entity_def: &EntityDefinition,
    ) -> Result<EntityValidationResult> {
        let mut field_results = Vec::new();
        let mut constraint_violations = Vec::new();
        let mut overall_valid = true;

        // Validate each field
        for (field_name, field_def) in &entity_def.fields {
            let mut field_valid = true;
            let mut errors = Vec::new();
            let mut warnings = Vec::new();

            // Check if required field is present
            if field_def.required && !attributes.contains_key(field_name) {
                errors.push(format!("Required field '{}' is missing", field_name));
                field_valid = false;
                overall_valid = false;
            }

            // Validate field value if present
            if let Some(value) = attributes.get(field_name) {
                if let Err(validation_errors) = self.validate_field_value(field_def, value) {
                    for error in validation_errors {
                        errors.push(error.message());
                        field_valid = false;
                        overall_valid = false;
                    }
                }
            }

            field_results.push(FieldValidationResult {
                field_name: field_name.clone(),
                valid: field_valid,
                errors,
                warnings,
            });
        }

        // Check for unknown fields
        for field_name in attributes.keys() {
            if !entity_def.fields.contains_key(field_name) {
                constraint_violations.push(format!("Unknown field '{}'", field_name));
            }
        }

        Ok(EntityValidationResult {
            entity_id: Uuid::new_v4(), // Placeholder for new entities
            entity_type: entity_type.to_string(),
            valid: overall_valid && constraint_violations.is_empty(),
            field_results,
            constraint_violations,
        })
    }

    /// Validate individual field value
    fn validate_field_value(
        &self,
        field_def: &crate::models::FieldDefinition,
        value: &serde_json::Value,
    ) -> Result<(), Vec<ValidationError>> {
        let mut errors = Vec::new();

        // Type validation
        match (&field_def.field_type, value) {
            (crate::models::FieldType::String, serde_json::Value::String(s)) => {
                if let Some(constraints) = &field_def.constraints {
                    if let Some(min_len) = constraints.min_length {
                        if s.len() < min_len {
                            errors.push(ValidationError::FieldInvalid {
                                field: "length".to_string(),
                                value: s.clone(),
                                reason: format!("Must be at least {} characters", min_len),
                            });
                        }
                    }
                    if let Some(max_len) = constraints.max_length {
                        if s.len() > max_len {
                            errors.push(ValidationError::FieldInvalid {
                                field: "length".to_string(),
                                value: s.clone(),
                                reason: format!("Must be at most {} characters", max_len),
                            });
                        }
                    }
                }
            }
            (crate::models::FieldType::Integer, serde_json::Value::Number(n)) => {
                if let Some(i) = n.as_i64() {
                    if let Some(constraints) = &field_def.constraints {
                        if let Some(min) = constraints.min {
                            if i < min {
                                errors.push(ValidationError::FieldInvalid {
                                    field: "value".to_string(),
                                    value: i.to_string(),
                                    reason: format!("Must be at least {}", min),
                                });
                            }
                        }
                        if let Some(max) = constraints.max {
                            if i > max {
                                errors.push(ValidationError::FieldInvalid {
                                    field: "value".to_string(),
                                    value: i.to_string(),
                                    reason: format!("Must be at most {}", max),
                                });
                            }
                        }
                    }
                }
            }
            (expected_type, actual_value) => {
                errors.push(ValidationError::FieldInvalid {
                    field: "type".to_string(),
                    value: format!("{:?}", actual_value),
                    reason: format!("Expected {:?}", expected_type),
                });
            }
        }

        if errors.is_empty() {
            Ok(())
        } else {
            Err(errors)
        }
    }

    /// Load default data from domain configuration
    pub async fn load_default_data(&self) -> Result<()> {
        if let Some(default_data) = &self.domain_config.default_data {
            for (entity_type, data) in default_data {
                if let Some(entities_data) = data.as_object() {
                    for (key, entity_data) in entities_data {
                        if let Some(attributes) = entity_data.as_object() {
                            let mut attr_map = HashMap::new();
                            for (attr_key, attr_value) in attributes {
                                attr_map.insert(attr_key.clone(), attr_value.clone());
                            }
                            
                            match self.create_entity(entity_type.clone(), attr_map).await {
                                Ok(_) => tracing::info!("Loaded default entity: {} -> {}", entity_type, key),
                                Err(e) => tracing::error!("Failed to load default entity {}: {}", key, e),
                            }
                        }
                    }
                }
            }
        }
        Ok(())
    }
}
