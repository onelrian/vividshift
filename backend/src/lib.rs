pub mod api;
pub mod auth;
pub mod config;
pub mod database;
pub mod engines;
pub mod models;
pub mod services;

// Re-export commonly used types
pub use models::{GenericEntity, Assignment, AssignmentResult};
pub use services::{RuleEngine, EntityManager};
pub use config::AppConfig;
pub use database::{DatabaseManager, RepositoryManager};
