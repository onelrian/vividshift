use config::{Config, ConfigError, Environment, File};
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, env};

use crate::models::{DomainConfig, EntityDefinition};
use crate::services::rule_engine::RuleEngineConfig;

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub database: DatabaseConfig,
    pub auth: AuthConfig,
    pub logging: LoggingConfig,
    pub domain: DomainConfig,
    pub rule_engine: RuleEngineConfig,
    pub engines: EngineConfig,
    // Backward compatibility
    pub work_assignments: Option<WorkAssignmentsConfig>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct ServerConfig {
    pub host: String,
    pub port: u16,
    pub environment: String,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct DatabaseConfig {
    pub url: String,
    pub max_connections: u32,
    pub min_connections: u32,
    pub connect_timeout: u64,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct AuthConfig {
    pub jwt_secret: String,
    pub jwt_expiration: i64,
    pub bcrypt_cost: u32,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct LoggingConfig {
    pub level: String,
    pub file_enabled: bool,
    pub file_path: String,
    pub json_format: bool,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct WorkAssignmentsConfig {
    pub max_attempts: u32,
    pub assignments: std::collections::HashMap<String, usize>,
}

#[derive(Debug, Deserialize, Serialize, Clone)]
pub struct EngineConfig {
    pub assignment_engines: Vec<String>,
    pub validation_engines: Vec<String>,
    pub distribution_algorithms: Vec<String>,
    pub default_strategy: String,
}

impl AppConfig {
    pub fn new() -> Result<Self, ConfigError> {
        let environment = env::var("ENVIRONMENT").unwrap_or_else(|_| "dev".to_string());
        
        let config = Config::builder()
            // Start with default configuration
            .add_source(File::with_name("backend/config/default").required(false))
            // Add environment-specific configuration
            .add_source(File::with_name(&format!("backend/config/{}", environment)).required(false))
            // Add local configuration (for development overrides)
            .add_source(File::with_name("backend/config/local").required(false))
            // Add environment variables with prefix "VIVIDSHIFT_"
            .add_source(Environment::with_prefix("VIVIDSHIFT").separator("_"))
            .build()?;

        config.try_deserialize()
    }

    pub fn is_development(&self) -> bool {
        self.server.environment == "dev" || self.server.environment == "development"
    }

    pub fn is_production(&self) -> bool {
        self.server.environment == "prod" || self.server.environment == "production"
    }

    pub fn is_staging(&self) -> bool {
        self.server.environment == "staging"
    }

    pub fn load_domain_config(&self, domain_name: &str) -> Result<DomainConfig, ConfigError> {
        let domain_path = format!("backend/config/domain/{}.toml", domain_name);
        let config = Config::builder()
            .add_source(File::with_name(&domain_path))
            .build()?;
        config.try_deserialize()
    }

    pub fn validate_domain_config(&self) -> Result<(), String> {
        // Validate entity definitions
        for (name, entity) in &self.domain.entities {
            if entity.name.is_empty() {
                return Err(format!("Entity '{}' has empty name", name));
            }
            
            if entity.fields.is_empty() {
                return Err(format!("Entity '{}' has no fields defined", name));
            }
        }
        Ok(())
    }
}

impl Default for AppConfig {
    fn default() -> Self {
        // Create default domain config for work groups
        let mut entities = HashMap::new();
        
        // Default assignment target entity
        let mut target_fields = HashMap::new();
        target_fields.insert("name".to_string(), crate::models::FieldDefinition {
            field_type: crate::models::FieldType::String,
            required: true,
            default_value: None,
            constraints: Some(crate::models::FieldConstraints {
                min: None,
                max: None,
                min_length: Some(1),
                max_length: Some(100),
                pattern: None,
                allowed_values: None,
                unique: Some(true),
            }),
            display_name: "Name".to_string(),
            description: Some("The name of the assignment target".to_string()),
        });
        
        target_fields.insert("required_count".to_string(), crate::models::FieldDefinition {
            field_type: crate::models::FieldType::Integer,
            required: true,
            default_value: Some(serde_json::json!(1)),
            constraints: Some(crate::models::FieldConstraints {
                min: Some(1),
                max: Some(100),
                min_length: None,
                max_length: None,
                pattern: None,
                allowed_values: None,
                unique: None,
            }),
            display_name: "Required Count".to_string(),
            description: Some("Number of participants required for this target".to_string()),
        });

        entities.insert("assignment_target".to_string(), EntityDefinition {
            name: "assignment_target".to_string(),
            display_name: "Assignment Target".to_string(),
            description: "A target that requires participant assignments".to_string(),
            version: "1.0".to_string(),
            fields: target_fields,
            relationships: HashMap::new(),
            constraints: Vec::new(),
        });

        // Legacy work assignments for backward compatibility
        let mut work_assignments = std::collections::HashMap::new();
        work_assignments.insert("Parlor".to_string(), 5);
        work_assignments.insert("Frontyard".to_string(), 3);
        work_assignments.insert("Backyard".to_string(), 1);
        work_assignments.insert("Tank".to_string(), 2);
        work_assignments.insert("Toilet B".to_string(), 4);
        work_assignments.insert("Toilet A".to_string(), 2);
        work_assignments.insert("Bin".to_string(), 1);

        Self {
            server: ServerConfig {
                host: "127.0.0.1".to_string(),
                port: 8080,
                environment: "dev".to_string(),
            },
            database: DatabaseConfig {
                url: "postgresql://localhost:5432/vividshift_dev".to_string(),
                max_connections: 10,
                min_connections: 1,
                connect_timeout: 30,
            },
            auth: AuthConfig {
                jwt_secret: "dev-secret-change-in-production".to_string(),
                jwt_expiration: 86400, // 24 hours
                bcrypt_cost: 12,
            },
            logging: LoggingConfig {
                level: "info".to_string(),
                file_enabled: false,
                file_path: "logs/app.log".to_string(),
                json_format: false,
            },
            domain: DomainConfig {
                name: "work_groups".to_string(),
                display_name: "Work Group Assignment System".to_string(),
                description: "Default work group assignment domain".to_string(),
                version: "1.0".to_string(),
                entities,
                default_data: None,
                business_rules: Vec::new(),
            },
            rule_engine: RuleEngineConfig::default(),
            engines: EngineConfig {
                assignment_engines: vec![
                    "balanced_rotation".to_string(),
                    "random_assignment".to_string(),
                ],
                validation_engines: vec![
                    "capacity_check".to_string(),
                    "availability_check".to_string(),
                ],
                distribution_algorithms: vec![
                    "round_robin".to_string(),
                    "weighted_random".to_string(),
                ],
                default_strategy: "balanced_rotation".to_string(),
            },
            work_assignments: Some(WorkAssignmentsConfig {
                max_attempts: 50,
                assignments: work_assignments,
            }),
        }
    }
}
