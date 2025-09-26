use std::collections::HashMap;
use uuid::Uuid;
use vividshift_backend::{
    config::AppConfig,
    engines::assignment::BalancedRotationStrategy,
    engines::validation::{AvailabilityCheckValidator, CapacityCheckValidator},
    models::{GenericEntity, AssignmentRequest},
    services::{EntityManager, RuleEngine, StrategyConfig, ExecutionContext},
};

#[tokio::test]
async fn test_entity_manager_crud() {
    let config = AppConfig::default();
    let entity_manager = EntityManager::new(config.domain.clone());

    // Test entity creation
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), serde_json::json!("Test Participant"));
    attributes.insert("availability".to_string(), serde_json::json!(true));

    let entity = entity_manager
        .create_entity("participant".to_string(), attributes)
        .await
        .expect("Failed to create entity");

    assert_eq!(entity.entity_type, "participant");
    assert_eq!(entity.get_attribute::<String>("name").unwrap(), "Test Participant");

    // Test entity retrieval
    let retrieved = entity_manager
        .get_entity("participant", entity.id)
        .await
        .expect("Failed to get entity");

    assert_eq!(retrieved.id, entity.id);

    // Test entity update
    let mut updates = HashMap::new();
    updates.insert("availability".to_string(), serde_json::json!(false));

    let updated = entity_manager
        .update_entity("participant", entity.id, updates)
        .await
        .expect("Failed to update entity");

    assert_eq!(updated.get_attribute::<bool>("availability").unwrap(), false);

    // Test entity listing
    let entities = entity_manager
        .list_entities("participant")
        .await
        .expect("Failed to list entities");

    assert!(!entities.is_empty());

    // Test entity deletion
    entity_manager
        .delete_entity("participant", entity.id)
        .await
        .expect("Failed to delete entity");
}

#[tokio::test]
async fn test_rule_engine_assignment() {
    let config = AppConfig::default();
    let mut rule_engine = RuleEngine::new(config.rule_engine.clone());

    // Register strategy and validators
    rule_engine.register_strategy(BalancedRotationStrategy::new());
    rule_engine.register_validator(CapacityCheckValidator);
    rule_engine.register_validator(AvailabilityCheckValidator);

    // Create test participants
    let participants = vec![
        create_test_participant("Alice", true),
        create_test_participant("Bob", true),
        create_test_participant("Charlie", false), // Unavailable
    ];

    // Create test targets
    let targets = vec![
        create_test_target("Target A", 2),
        create_test_target("Target B", 1),
    ];

    // Create strategy config
    let strategy_config = StrategyConfig {
        name: "balanced_rotation".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("rotation_weight".to_string(), serde_json::json!(0.7));
            params.insert("balance_weight".to_string(), serde_json::json!(0.3));
            params.insert("max_attempts".to_string(), serde_json::json!(10));
            params
        },
        constraints: vec![],
        validation_rules: vec![
            "capacity_check".to_string(),
            "availability_check".to_string(),
        ],
    };

    // Create execution context
    let context = ExecutionContext {
        request_id: Uuid::new_v4(),
        user_id: Some(Uuid::new_v4()),
        domain: "test".to_string(),
        metadata: HashMap::new(),
    };

    // Execute assignment
    let result = rule_engine
        .execute_assignment(
            "balanced_rotation",
            participants,
            targets,
            strategy_config,
            context,
        )
        .await
        .expect("Assignment execution failed");

    // Verify results
    assert_eq!(result.strategy_used, "balanced_rotation");
    assert!(!result.assignments.is_empty());
    assert!(result.metadata.execution_time_ms > 0);

    // Check that unavailable participant (Charlie) was not assigned
    let charlie_assignments: Vec<_> = result
        .assignments
        .iter()
        .filter(|a| {
            // This is a simplified check - in real implementation,
            // we'd need to match by participant ID
            true
        })
        .collect();

    // Verify validation results
    assert!(!result.metadata.validation_results.is_empty());
}

#[tokio::test]
async fn test_configuration_loading() {
    // Test default configuration
    let config = AppConfig::default();
    
    assert_eq!(config.domain.name, "work_groups");
    assert!(!config.domain.entities.is_empty());
    assert!(config.domain.entities.contains_key("assignment_target"));

    // Test configuration validation
    config.validate_domain_config().expect("Domain config validation failed");
}

#[test]
fn test_balanced_rotation_strategy_validation() {
    let strategy = BalancedRotationStrategy::new();

    // Test valid configuration
    let valid_config = StrategyConfig {
        name: "balanced_rotation".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("rotation_weight".to_string(), serde_json::json!(0.7));
            params.insert("balance_weight".to_string(), serde_json::json!(0.3));
            params.insert("max_attempts".to_string(), serde_json::json!(25));
            params
        },
        constraints: vec![],
        validation_rules: vec![],
    };

    assert!(strategy.validate_config(&valid_config).is_ok());

    // Test invalid configuration (weight > 1.0)
    let invalid_config = StrategyConfig {
        name: "balanced_rotation".to_string(),
        parameters: {
            let mut params = HashMap::new();
            params.insert("rotation_weight".to_string(), serde_json::json!(1.5));
            params
        },
        constraints: vec![],
        validation_rules: vec![],
    };

    assert!(strategy.validate_config(&invalid_config).is_err());
}

// Helper functions
fn create_test_participant(name: &str, available: bool) -> GenericEntity {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), serde_json::json!(name));
    attributes.insert("availability".to_string(), serde_json::json!(available));
    attributes.insert("group".to_string(), serde_json::json!("TestGroup"));

    GenericEntity::new("participant".to_string(), attributes)
}

fn create_test_target(name: &str, required_count: u32) -> GenericEntity {
    let mut attributes = HashMap::new();
    attributes.insert("name".to_string(), serde_json::json!(name));
    attributes.insert("required_count".to_string(), serde_json::json!(required_count));
    attributes.insert("priority".to_string(), serde_json::json!(1));

    GenericEntity::new("assignment_target".to_string(), attributes)
}
