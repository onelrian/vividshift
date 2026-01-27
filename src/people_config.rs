//! People Configuration Module
//!
//! This module provides type-safe loading and validation of people configuration
//! from TOML files. It replaces the legacy file_a.txt and file_b.txt approach
//! with a structured, validated configuration system.
//!
//! # Overview
//!
//! The configuration is loaded from `config/people.toml` and provides:
//! - Group definitions with metadata and constraints
//! - Individual person records with group assignments
//! - Validation of data integrity
//! - Filtering and querying capabilities
//!
//! # Usage
//!
//! ```no_run
//! use work_group_generator::people_config::PeopleConfiguration;
//!
//! let config = PeopleConfiguration::load()?;
//! let group_a_people = config.get_people_by_group("A");
//! let active_people = config.get_active_people();
//! ```
//!
//! # Error Handling
//!
//! All operations return `Result` types with descriptive errors using `thiserror`.
//! No panics or unwraps are used in production code paths.

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet};
use std::fs;
use std::path::Path;
use thiserror::Error;
use tracing::{debug, info, warn};

/// Errors that can occur when loading or validating people configuration
#[derive(Error, Debug)]
pub enum ConfigError {
    /// Configuration file could not be read
    #[error("Failed to read configuration file: {0}")]
    FileRead(#[from] std::io::Error),

    /// Configuration file contains invalid TOML
    #[error("Failed to parse TOML configuration: {0}")]
    TomlParse(#[from] toml::de::Error),

    /// Configuration failed validation checks
    #[error("Configuration validation failed: {0}")]
    Validation(#[from] ValidationError),

    /// Configuration file not found at expected location
    #[error("Configuration file not found at path: {0}")]
    NotFound(String),
}

/// Validation errors for people configuration
#[derive(Error, Debug)]
pub enum ValidationError {
    /// Duplicate person names found
    #[error("Duplicate person names found: {0:?}")]
    DuplicateNames(Vec<String>),

    /// Person references non-existent group
    #[error("Person '{person}' references undefined group '{group}'")]
    UndefinedGroup { person: String, group: String },

    /// Group has no active members
    #[error("Group '{0}' has no active members")]
    NoActiveMembers(String),

    /// No people defined in configuration
    #[error("Configuration must contain at least one person")]
    EmptyConfiguration,
}

/// Configuration for a single group
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct GroupConfig {
    /// Human-readable description of the group
    pub description: String,

    /// List of constraint identifiers that apply to this group
    /// Example: ["cannot_perform_toilet_b"]
    #[serde(default)]
    pub constraints: Vec<String>,
}

/// Configuration for a single person
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct PersonConfig {
    /// Person's full name (must be unique)
    pub name: String,

    /// Group identifier (must reference a defined group)
    pub group: String,

    /// Whether the person is currently active
    #[serde(default = "default_active")]
    pub active: bool,
}

fn default_active() -> bool {
    true
}

/// Root configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeopleConfiguration {
    /// Map of group identifiers to group configurations
    pub groups: HashMap<String, GroupConfig>,

    /// List of all people
    #[serde(rename = "person")]
    pub people: Vec<PersonConfig>,
}

#[allow(dead_code)]
impl PeopleConfiguration {
    /// Default path to the people configuration file
    pub const DEFAULT_CONFIG_PATH: &'static str = "config/people.toml";

    /// Load people configuration from the default path
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if:
    /// - File cannot be read
    /// - TOML parsing fails
    /// - Validation fails
    ///
    /// # Example
    ///
    /// ```no_run
    /// let config = PeopleConfiguration::load()?;
    /// ```
    pub fn load() -> Result<Self, ConfigError> {
        Self::load_from_path(Self::DEFAULT_CONFIG_PATH)
    }

    /// Load people configuration from a specific path
    ///
    /// # Arguments
    ///
    /// * `path` - Path to the TOML configuration file
    ///
    /// # Errors
    ///
    /// Returns `ConfigError` if file operations or validation fails
    pub fn load_from_path<P: AsRef<Path>>(path: P) -> Result<Self, ConfigError> {
        let path = path.as_ref();
        info!("Loading people configuration from: {}", path.display());

        // Check if file exists
        if !path.exists() {
            warn!("Configuration file not found: {}", path.display());
            return Err(ConfigError::NotFound(path.display().to_string()));
        }

        // Read file contents
        let content = fs::read_to_string(path).map_err(|e| {
            warn!("Failed to read configuration file: {}", e);
            ConfigError::FileRead(e)
        })?;

        debug!("Configuration file read successfully, parsing TOML...");

        // Parse TOML
        let config: Self = toml::from_str(&content).map_err(|e| {
            warn!("Failed to parse TOML: {}", e);
            ConfigError::TomlParse(e)
        })?;

        info!(
            "Parsed configuration: {} groups, {} people",
            config.groups.len(),
            config.people.len()
        );

        // Validate configuration
        config.validate()?;

        info!("Configuration loaded and validated successfully");
        Ok(config)
    }

    /// Validate the configuration for consistency and correctness
    ///
    /// Checks:
    /// - At least one person exists
    /// - No duplicate names
    /// - All group references are valid
    /// - Each group has at least one active member
    ///
    /// # Errors
    ///
    /// Returns `ValidationError` if any validation check fails
    pub fn validate(&self) -> Result<(), ValidationError> {
        debug!("Validating people configuration...");

        // Check for empty configuration
        if self.people.is_empty() {
            return Err(ValidationError::EmptyConfiguration);
        }

        // Check for duplicate names
        let mut seen_names = HashSet::new();
        let mut duplicates = Vec::new();

        for person in &self.people {
            if !seen_names.insert(person.name.clone()) {
                duplicates.push(person.name.clone());
            }
        }

        if !duplicates.is_empty() {
            return Err(ValidationError::DuplicateNames(duplicates));
        }

        // Check all group references are valid
        for person in &self.people {
            if !self.groups.contains_key(&person.group) {
                return Err(ValidationError::UndefinedGroup {
                    person: person.name.clone(),
                    group: person.group.clone(),
                });
            }
        }

        // Check each group has at least one active member
        for group_id in self.groups.keys() {
            let active_count = self
                .people
                .iter()
                .filter(|p| p.group == *group_id && p.active)
                .count();

            if active_count == 0 {
                return Err(ValidationError::NoActiveMembers(group_id.clone()));
            }
        }

        debug!("Validation passed");
        Ok(())
    }

    /// Get all people belonging to a specific group
    ///
    /// # Arguments
    ///
    /// * `group` - Group identifier to filter by
    ///
    /// # Returns
    ///
    /// Vector of references to people in the specified group
    pub fn get_people_by_group(&self, group: &str) -> Vec<&PersonConfig> {
        self.people.iter().filter(|p| p.group == group).collect()
    }

    /// Get all active people (across all groups)
    ///
    /// # Returns
    ///
    /// Vector of references to active people
    pub fn get_active_people(&self) -> Vec<&PersonConfig> {
        self.people.iter().filter(|p| p.active).collect()
    }

    /// Get all active people in a specific group
    ///
    /// # Arguments
    ///
    /// * `group` - Group identifier to filter by
    ///
    /// # Returns
    ///
    /// Vector of references to active people in the specified group
    pub fn get_active_people_by_group(&self, group: &str) -> Vec<&PersonConfig> {
        self.people
            .iter()
            .filter(|p| p.group == group && p.active)
            .collect()
    }

    /// Get configuration for a specific group
    ///
    /// # Arguments
    ///
    /// * `group` - Group identifier
    ///
    /// # Returns
    ///
    /// Option containing the group configuration if it exists
    pub fn get_group(&self, group: &str) -> Option<&GroupConfig> {
        self.groups.get(group)
    }

    /// Get all group identifiers
    ///
    /// # Returns
    ///
    /// Iterator over group identifier strings
    pub fn get_group_ids(&self) -> impl Iterator<Item = &String> {
        self.groups.keys()
    }

    /// Get total count of people (active and inactive)
    pub fn total_people(&self) -> usize {
        self.people.len()
    }

    /// Get count of active people
    pub fn active_people_count(&self) -> usize {
        self.people.iter().filter(|p| p.active).count()
    }

    /// Check if a person with the given name exists
    ///
    /// # Arguments
    ///
    /// * `name` - Name to search for
    ///
    /// # Returns
    ///
    /// True if a person with this name exists (case-sensitive)
    pub fn has_person(&self, name: &str) -> bool {
        self.people.iter().any(|p| p.name == name)
    }

    /// Find a person by name
    ///
    /// # Arguments
    ///
    /// * `name` - Name to search for
    ///
    /// # Returns
    ///
    /// Option containing the person configuration if found
    pub fn find_person(&self, name: &str) -> Option<&PersonConfig> {
        self.people.iter().find(|p| p.name == name)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_group_config_serialization() {
        let group = GroupConfig {
            description: "Test group".to_string(),
            constraints: vec!["test_constraint".to_string()],
        };

        let toml = toml::to_string(&group).unwrap();
        let deserialized: GroupConfig = toml::from_str(&toml).unwrap();

        assert_eq!(group, deserialized);
    }

    #[test]
    fn test_person_config_default_active() {
        let toml = r#"
            name = "Test Person"
            group = "A"
        "#;

        let person: PersonConfig = toml::from_str(toml).unwrap();
        assert!(person.active, "Active should default to true");
    }

    #[test]
    fn test_validation_empty_config() {
        let config = PeopleConfiguration {
            groups: HashMap::new(),
            people: Vec::new(),
        };

        let result = config.validate();
        assert!(
            matches!(result, Err(ValidationError::EmptyConfiguration)),
            "Should reject empty configuration"
        );
    }

    #[test]
    fn test_validation_duplicate_names() {
        let mut groups = HashMap::new();
        groups.insert(
            "A".to_string(),
            GroupConfig {
                description: "Group A".to_string(),
                constraints: vec![],
            },
        );

        let config = PeopleConfiguration {
            groups,
            people: vec![
                PersonConfig {
                    name: "John".to_string(),
                    group: "A".to_string(),
                    active: true,
                },
                PersonConfig {
                    name: "John".to_string(), // Duplicate!
                    group: "A".to_string(),
                    active: true,
                },
            ],
        };

        let result = config.validate();
        assert!(
            matches!(result, Err(ValidationError::DuplicateNames(_))),
            "Should reject duplicate names"
        );
    }

    #[test]
    fn test_validation_undefined_group() {
        let config = PeopleConfiguration {
            groups: HashMap::new(), // No groups defined
            people: vec![PersonConfig {
                name: "John".to_string(),
                group: "A".to_string(), // References undefined group
                active: true,
            }],
        };

        let result = config.validate();
        assert!(
            matches!(result, Err(ValidationError::UndefinedGroup { .. })),
            "Should reject undefined group reference"
        );
    }

    #[test]
    fn test_get_people_by_group() {
        let mut groups = HashMap::new();
        groups.insert(
            "A".to_string(),
            GroupConfig {
                description: "Group A".to_string(),
                constraints: vec![],
            },
        );
        groups.insert(
            "B".to_string(),
            GroupConfig {
                description: "Group B".to_string(),
                constraints: vec![],
            },
        );

        let config = PeopleConfiguration {
            groups,
            people: vec![
                PersonConfig {
                    name: "Alice".to_string(),
                    group: "A".to_string(),
                    active: true,
                },
                PersonConfig {
                    name: "Bob".to_string(),
                    group: "B".to_string(),
                    active: true,
                },
                PersonConfig {
                    name: "Charlie".to_string(),
                    group: "A".to_string(),
                    active: true,
                },
            ],
        };

        let group_a = config.get_people_by_group("A");
        assert_eq!(group_a.len(), 2);
        assert!(group_a.iter().any(|p| p.name == "Alice"));
        assert!(group_a.iter().any(|p| p.name == "Charlie"));
    }

    #[test]
    fn test_get_active_people() {
        let mut groups = HashMap::new();
        groups.insert(
            "A".to_string(),
            GroupConfig {
                description: "Group A".to_string(),
                constraints: vec![],
            },
        );

        let config = PeopleConfiguration {
            groups,
            people: vec![
                PersonConfig {
                    name: "Active".to_string(),
                    group: "A".to_string(),
                    active: true,
                },
                PersonConfig {
                    name: "Inactive".to_string(),
                    group: "A".to_string(),
                    active: false,
                },
            ],
        };

        let active = config.get_active_people();
        assert_eq!(active.len(), 1);
        assert_eq!(active[0].name, "Active");
    }
}
