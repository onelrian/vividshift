use anyhow::Result;
use sqlx::PgPool;
use tracing::{info, warn};
use bcrypt::{hash, DEFAULT_COST};

use crate::database::repositories::RepositoryManager;
use crate::database::models::{CreateUser, CreateParticipant, CreateAssignmentTarget};

/// Database seeding manager for development and testing
pub struct SeedManager {
    repo_manager: RepositoryManager,
    pool: PgPool,
}

impl SeedManager {
    pub fn new(repo_manager: RepositoryManager, pool: PgPool) -> Self {
        Self { repo_manager, pool }
    }

    /// Run all seeding operations
    pub async fn seed_all(&self, force: bool) -> Result<()> {
        info!("🌱 Starting database seeding...");

        // Check if already seeded (unless force is true)
        if !force && self.is_already_seeded().await? {
            info!("Database already seeded. Use force=true to re-seed.");
            return Ok(());
        }

        // Seed in dependency order
        self.seed_users().await?;
        self.seed_domains().await?;
        self.seed_participants().await?;
        self.seed_assignment_targets().await?;
        self.seed_rule_configurations().await?;

        info!("✅ Database seeding completed successfully");
        Ok(())
    }

    /// Check if database has been seeded
    async fn is_already_seeded(&self) -> Result<bool> {
        let user_count = self.repo_manager.users.count().await?;
        Ok(user_count > 0)
    }

    /// Seed default users
    async fn seed_users(&self) -> Result<()> {
        info!("Seeding users...");

        let users = vec![
            CreateUser {
                username: "admin".to_string(),
                email: "admin@vividshift.local".to_string(),
                password_hash: hash("admin123", DEFAULT_COST)?,
                role: Some("admin".to_string()),
            },
            CreateUser {
                username: "user".to_string(),
                email: "user@vividshift.local".to_string(),
                password_hash: hash("user123", DEFAULT_COST)?,
                role: Some("user".to_string()),
            },
            CreateUser {
                username: "viewer".to_string(),
                email: "viewer@vividshift.local".to_string(),
                password_hash: hash("viewer123", DEFAULT_COST)?,
                role: Some("viewer".to_string()),
            },
        ];

        for user in users {
            match self.repo_manager.users.find_by_username(&user.username).await? {
                Some(_) => {
                    info!("User '{}' already exists, skipping", user.username);
                }
                None => {
                    let created_user = self.repo_manager.users.create(&user).await?;
                    info!("Created user: {} ({})", created_user.username, created_user.email);
                }
            }
        }

        Ok(())
    }

    /// Seed default domains
    async fn seed_domains(&self) -> Result<()> {
        info!("Seeding domains...");

        let domain_config = serde_json::json!({
            "assignment_strategies": ["balanced_rotation", "random_assignment"],
            "validation_rules": ["capacity_check", "availability_check"],
            "max_assignments_per_participant": 3,
            "allow_self_assignment": false
        });

        let business_rules = serde_json::json!([
            {
                "name": "max_consecutive_assignments",
                "description": "Limit consecutive assignments for fairness",
                "type": "constraint",
                "parameters": { "max_consecutive": 2 }
            },
            {
                "name": "skill_matching",
                "description": "Match participants to tasks based on skills",
                "type": "preference",
                "parameters": { "weight": 0.8 }
            }
        ]);

        // Get admin user for created_by
        let admin_user = self.repo_manager.users.find_by_username("admin").await?;
        let admin_id = admin_user.map(|u| u.id);

        match self.repo_manager.domains.find_domain_by_name("work_groups").await? {
            Some(_) => {
                info!("Domain 'work_groups' already exists, skipping");
            }
            None => {
                let domain = self.repo_manager.domains.create_domain(
                    "work_groups",
                    "Work Group Assignment System",
                    Some("Default domain for work group assignments and task distribution"),
                    &domain_config,
                    &business_rules,
                    admin_id
                ).await?;
                info!("Created domain: {} ({})", domain.name, domain.display_name);
            }
        }

        Ok(())
    }

    /// Seed sample participants
    async fn seed_participants(&self) -> Result<()> {
        info!("Seeding participants...");

        let admin_user = self.repo_manager.users.find_by_username("admin").await?;
        let admin_id = admin_user.map(|u| u.id);

        let participants = vec![
            CreateParticipant {
                name: "Alice Johnson".to_string(),
                email: Some("alice@example.com".to_string()),
                phone: Some("+1-555-0101".to_string()),
                skills: Some(serde_json::json!(["cleaning", "organizing", "leadership"])),
                availability: Some(serde_json::json!({
                    "monday": true,
                    "tuesday": true,
                    "wednesday": false,
                    "thursday": true,
                    "friday": true,
                    "weekend": false
                })),
                preferences: Some(serde_json::json!({
                    "preferred_tasks": ["parlor", "frontyard"],
                    "avoid_tasks": ["toilet_cleaning"]
                })),
                metadata: Some(serde_json::json!({
                    "experience_level": "senior",
                    "notes": "Team leader with 3+ years experience"
                })),
            },
            CreateParticipant {
                name: "Bob Smith".to_string(),
                email: Some("bob@example.com".to_string()),
                phone: Some("+1-555-0102".to_string()),
                skills: Some(serde_json::json!(["maintenance", "repairs", "heavy_lifting"])),
                availability: Some(serde_json::json!({
                    "monday": true,
                    "tuesday": false,
                    "wednesday": true,
                    "thursday": true,
                    "friday": true,
                    "weekend": true
                })),
                preferences: Some(serde_json::json!({
                    "preferred_tasks": ["backyard", "tank", "bin"],
                    "max_assignments_per_day": 2
                })),
                metadata: Some(serde_json::json!({
                    "experience_level": "intermediate",
                    "notes": "Good with physical tasks and equipment"
                })),
            },
            CreateParticipant {
                name: "Carol Davis".to_string(),
                email: Some("carol@example.com".to_string()),
                phone: Some("+1-555-0103".to_string()),
                skills: Some(serde_json::json!(["detail_oriented", "sanitization", "quality_control"])),
                availability: Some(serde_json::json!({
                    "monday": false,
                    "tuesday": true,
                    "wednesday": true,
                    "thursday": false,
                    "friday": true,
                    "weekend": false
                })),
                preferences: Some(serde_json::json!({
                    "preferred_tasks": ["toilet_a", "toilet_b"],
                    "preferred_time": "morning"
                })),
                metadata: Some(serde_json::json!({
                    "experience_level": "expert",
                    "notes": "Specialist in sanitation and hygiene tasks"
                })),
            },
            CreateParticipant {
                name: "David Wilson".to_string(),
                email: Some("david@example.com".to_string()),
                phone: Some("+1-555-0104".to_string()),
                skills: Some(serde_json::json!(["flexible", "quick_learner", "team_player"])),
                availability: Some(serde_json::json!({
                    "monday": true,
                    "tuesday": true,
                    "wednesday": true,
                    "thursday": true,
                    "friday": false,
                    "weekend": true
                })),
                preferences: Some(serde_json::json!({
                    "willing_to_learn": true,
                    "no_preferences": true
                })),
                metadata: Some(serde_json::json!({
                    "experience_level": "beginner",
                    "notes": "New team member, eager to help anywhere needed"
                })),
            },
        ];

        for participant in participants {
            match self.repo_manager.participants.find_by_name(&participant.name).await? {
                Some(_) => {
                    info!("Participant '{}' already exists, skipping", participant.name);
                }
                None => {
                    let created = self.repo_manager.participants.create(&participant, admin_id).await?;
                    info!("Created participant: {} ({})", created.name, created.email.unwrap_or_default());
                }
            }
        }

        Ok(())
    }

    /// Seed assignment targets (work areas)
    async fn seed_assignment_targets(&self) -> Result<()> {
        info!("Seeding assignment targets...");

        let admin_user = self.repo_manager.users.find_by_username("admin").await?;
        let admin_id = admin_user.map(|u| u.id);

        let targets = vec![
            CreateAssignmentTarget {
                name: "Parlor".to_string(),
                description: Some("Main living area cleaning and maintenance".to_string()),
                required_count: 5,
                required_skills: Some(serde_json::json!(["cleaning", "organizing"])),
                constraints: Some(serde_json::json!({
                    "min_experience": "intermediate",
                    "requires_leadership": true
                })),
                metadata: Some(serde_json::json!({
                    "priority": "high",
                    "estimated_duration": "2 hours",
                    "difficulty": "medium"
                })),
            },
            CreateAssignmentTarget {
                name: "Frontyard".to_string(),
                description: Some("Outdoor front area maintenance and landscaping".to_string()),
                required_count: 3,
                required_skills: Some(serde_json::json!(["outdoor_work", "landscaping"])),
                constraints: Some(serde_json::json!({
                    "weather_dependent": true,
                    "requires_tools": ["rake", "shovel"]
                })),
                metadata: Some(serde_json::json!({
                    "priority": "medium",
                    "estimated_duration": "1.5 hours",
                    "difficulty": "easy"
                })),
            },
            CreateAssignmentTarget {
                name: "Backyard".to_string(),
                description: Some("Rear outdoor area cleaning and maintenance".to_string()),
                required_count: 1,
                required_skills: Some(serde_json::json!(["maintenance", "heavy_lifting"])),
                constraints: Some(serde_json::json!({
                    "requires_strength": true,
                    "safety_equipment_required": true
                })),
                metadata: Some(serde_json::json!({
                    "priority": "medium",
                    "estimated_duration": "1 hour",
                    "difficulty": "medium"
                })),
            },
            CreateAssignmentTarget {
                name: "Tank".to_string(),
                description: Some("Water tank cleaning and maintenance".to_string()),
                required_count: 2,
                required_skills: Some(serde_json::json!(["maintenance", "technical"])),
                constraints: Some(serde_json::json!({
                    "requires_certification": false,
                    "safety_critical": true
                })),
                metadata: Some(serde_json::json!({
                    "priority": "high",
                    "estimated_duration": "3 hours",
                    "difficulty": "hard"
                })),
            },
            CreateAssignmentTarget {
                name: "Toilet A".to_string(),
                description: Some("Primary bathroom cleaning and sanitization".to_string()),
                required_count: 2,
                required_skills: Some(serde_json::json!(["sanitization", "detail_oriented"])),
                constraints: Some(serde_json::json!({
                    "hygiene_critical": true,
                    "requires_supplies": ["disinfectant", "gloves"]
                })),
                metadata: Some(serde_json::json!({
                    "priority": "high",
                    "estimated_duration": "1 hour",
                    "difficulty": "medium"
                })),
            },
            CreateAssignmentTarget {
                name: "Toilet B".to_string(),
                description: Some("Secondary bathroom cleaning and sanitization".to_string()),
                required_count: 4,
                required_skills: Some(serde_json::json!(["sanitization", "detail_oriented"])),
                constraints: Some(serde_json::json!({
                    "hygiene_critical": true,
                    "requires_supplies": ["disinfectant", "gloves"]
                })),
                metadata: Some(serde_json::json!({
                    "priority": "high",
                    "estimated_duration": "1 hour",
                    "difficulty": "medium"
                })),
            },
            CreateAssignmentTarget {
                name: "Bin".to_string(),
                description: Some("Waste management and bin cleaning".to_string()),
                required_count: 1,
                required_skills: Some(serde_json::json!(["waste_management", "heavy_lifting"])),
                constraints: Some(serde_json::json!({
                    "requires_protective_gear": true,
                    "outdoor_task": true
                })),
                metadata: Some(serde_json::json!({
                    "priority": "medium",
                    "estimated_duration": "0.5 hours",
                    "difficulty": "easy"
                })),
            },
        ];

        for target in targets {
            match self.repo_manager.assignments.find_target_by_name(&target.name).await? {
                Some(_) => {
                    info!("Assignment target '{}' already exists, skipping", target.name);
                }
                None => {
                    let created = self.repo_manager.assignments.create_target(&target, admin_id).await?;
                    info!("Created assignment target: {} (requires {} people)", created.name, created.required_count);
                }
            }
        }

        Ok(())
    }

    /// Seed rule configurations
    async fn seed_rule_configurations(&self) -> Result<()> {
        info!("Seeding rule configurations...");

        let admin_user = self.repo_manager.users.find_by_username("admin").await?;
        let admin_id = admin_user.map(|u| u.id);

        let domain = self.repo_manager.domains.find_domain_by_name("work_groups").await?;
        let domain_id = domain.map(|d| d.id);

        let configs = vec![
            (
                "balanced_rotation_default",
                "Default Balanced Rotation Strategy",
                "balanced_rotation",
                serde_json::json!({
                    "rotation_period": "weekly",
                    "balance_factor": 0.8,
                    "consider_preferences": true,
                    "avoid_consecutive": true,
                    "max_assignments_per_person": 3
                })
            ),
            (
                "skill_based_assignment",
                "Skill-Based Assignment Strategy",
                "skill_based",
                serde_json::json!({
                    "skill_weight": 0.9,
                    "experience_weight": 0.7,
                    "availability_weight": 0.8,
                    "preference_weight": 0.6,
                    "strict_skill_matching": false
                })
            ),
            (
                "capacity_validation",
                "Capacity Check Validation",
                "capacity_check",
                serde_json::json!({
                    "enforce_max_capacity": true,
                    "allow_overallocation": false,
                    "buffer_percentage": 0.1
                })
            ),
            (
                "availability_validation",
                "Availability Check Validation",
                "availability_check",
                serde_json::json!({
                    "strict_availability": true,
                    "allow_partial_availability": false,
                    "check_time_conflicts": true
                })
            ),
        ];

        for (name, description, strategy_type, config) in configs {
            match self.repo_manager.domains.find_rule_config_by_name(name).await? {
                Some(_) => {
                    info!("Rule configuration '{}' already exists, skipping", name);
                }
                None => {
                    let created = self.repo_manager.domains.create_rule_config(
                        name,
                        Some(description),
                        domain_id,
                        strategy_type,
                        &config,
                        admin_id
                    ).await?;
                    info!("Created rule configuration: {} ({})", created.name, created.strategy_type);
                }
            }
        }

        Ok(())
    }

    /// Clear all seeded data (for testing)
    pub async fn clear_all(&self) -> Result<()> {
        warn!("🗑️  Clearing all seeded data...");

        // Clear in reverse dependency order
        sqlx::query("DELETE FROM assignment_history").execute(&self.pool).await?;
        sqlx::query("DELETE FROM assignment_details").execute(&self.pool).await?;
        sqlx::query("DELETE FROM assignments").execute(&self.pool).await?;
        sqlx::query("DELETE FROM rule_configurations").execute(&self.pool).await?;
        sqlx::query("DELETE FROM assignment_targets").execute(&self.pool).await?;
        sqlx::query("DELETE FROM participants").execute(&self.pool).await?;
        sqlx::query("DELETE FROM entities").execute(&self.pool).await?;
        sqlx::query("DELETE FROM entity_definitions").execute(&self.pool).await?;
        sqlx::query("DELETE FROM domains").execute(&self.pool).await?;
        sqlx::query("DELETE FROM user_profiles").execute(&self.pool).await?;
        sqlx::query("DELETE FROM user_sessions").execute(&self.pool).await?;
        sqlx::query("DELETE FROM users").execute(&self.pool).await?;

        info!("✅ All seeded data cleared");
        Ok(())
    }
}
