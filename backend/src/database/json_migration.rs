use anyhow::Result;
use serde::Deserialize;
use serde_json::Value;
use std::path::Path;
use tracing::{info, error};
use uuid::Uuid;

use crate::database::repositories::RepositoryManager;
use crate::database::models::{CreateParticipant, CreateAssignmentTarget};

/// JSON migration manager
pub struct JsonMigrationManager {
    repo_manager: RepositoryManager,
}

#[derive(Debug, Deserialize)]
pub struct JsonParticipant {
    pub name: String,
    pub email: Option<String>,
    pub phone: Option<String>,
    pub skills: Option<Vec<String>>,
    pub availability: Option<Value>,
    pub preferences: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Deserialize)]
pub struct JsonAssignmentTarget {
    pub name: String,
    pub description: Option<String>,
    pub required_count: Option<i32>,
    pub required_skills: Option<Vec<String>>,
    pub constraints: Option<Value>,
    pub metadata: Option<Value>,
}

#[derive(Debug, Default)]
pub struct MigrationStats {
    pub participants_created: usize,
    pub targets_created: usize,
    pub errors: Vec<String>,
}

impl JsonMigrationManager {
    pub fn new(repo_manager: RepositoryManager) -> Self {
        Self { repo_manager }
    }

    pub async fn migrate_from_directory(&self, migration_dir: &Path) -> Result<MigrationStats> {
        info!("Starting JSON migration from: {}", migration_dir.display());
        let mut stats = MigrationStats::default();

        // Get admin user
        let admin_user = self.repo_manager.users.find_by_username("admin").await?;
        let admin_id = admin_user.map(|u| u.id);

        // Migrate participants
        let participants_file = migration_dir.join("participants.json");
        if participants_file.exists() {
            self.migrate_participants(&participants_file, admin_id, &mut stats).await?;
        }

        // Migrate assignment targets
        let targets_file = migration_dir.join("assignment_targets.json");
        if targets_file.exists() {
            self.migrate_assignment_targets(&targets_file, admin_id, &mut stats).await?;
        }

        Ok(stats)
    }

    async fn migrate_participants(&self, file_path: &Path, admin_id: Option<Uuid>, stats: &mut MigrationStats) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let json_participants: Vec<JsonParticipant> = serde_json::from_str(&content)?;

        for json_participant in json_participants {
            let create_participant = CreateParticipant {
                name: json_participant.name.clone(),
                email: json_participant.email,
                phone: json_participant.phone,
                skills: json_participant.skills.map(|skills| {
                    serde_json::Value::Array(skills.into_iter().map(serde_json::Value::String).collect())
                }),
                availability: json_participant.availability,
                preferences: json_participant.preferences,
                metadata: json_participant.metadata,
            };

            match self.repo_manager.participants.create(&create_participant, admin_id).await {
                Ok(created) => {
                    info!("Created participant: {}", created.name);
                    stats.participants_created += 1;
                }
                Err(e) => {
                    error!("Failed to create participant {}: {}", json_participant.name, e);
                    stats.errors.push(format!("Participant {}: {}", json_participant.name, e));
                }
            }
        }

        Ok(())
    }

    async fn migrate_assignment_targets(&self, file_path: &Path, admin_id: Option<Uuid>, stats: &mut MigrationStats) -> Result<()> {
        let content = std::fs::read_to_string(file_path)?;
        let json_targets: Vec<JsonAssignmentTarget> = serde_json::from_str(&content)?;

        for json_target in json_targets {
            let create_target = CreateAssignmentTarget {
                name: json_target.name.clone(),
                description: json_target.description,
                required_count: json_target.required_count.unwrap_or(1),
                required_skills: json_target.required_skills.map(|skills| {
                    serde_json::Value::Array(skills.into_iter().map(serde_json::Value::String).collect())
                }),
                constraints: json_target.constraints,
                metadata: json_target.metadata,
            };

            match self.repo_manager.assignments.create_target(&create_target, admin_id).await {
                Ok(created) => {
                    info!("Created assignment target: {}", created.name);
                    stats.targets_created += 1;
                }
                Err(e) => {
                    error!("Failed to create target {}: {}", json_target.name, e);
                    stats.errors.push(format!("Target {}: {}", json_target.name, e));
                }
            }
        }

        Ok(())
    }
}
