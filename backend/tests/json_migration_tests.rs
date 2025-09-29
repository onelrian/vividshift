use anyhow::Result;
use std::path::Path;
use tempfile::TempDir;
use tokio_test;

use vividshift_backend::{
    config::AppConfig,
    database::{DatabaseManager, RepositoryManager, JsonMigrationManager, SeedManager, run_migrations},
};

/// Test JSON migration functionality
#[tokio::test]
async fn test_json_migration_participants() -> Result<()> {
    // Setup test database
    let config = AppConfig::from_env().expect("Failed to load config");
    let db_manager = DatabaseManager::new(&config.database).await?;
    let repo_manager = RepositoryManager::new(db_manager.pool().clone());
    
    // Run migrations
    run_migrations(db_manager.pool()).await?;
    
    // Seed admin user first
    let seed_manager = SeedManager::new(repo_manager.clone(), db_manager.pool().clone());
    seed_manager.seed_users().await?;
    
    // Create temporary JSON file
    let temp_dir = TempDir::new()?;
    let participants_file = temp_dir.path().join("participants.json");
    
    let json_content = r#"[
        {
            "name": "Test User 1",
            "email": "test1@example.com",
            "phone": "+1-555-0001",
            "skills": ["cleaning", "organizing"],
            "availability": {
                "monday": ["09:00", "17:00"],
                "tuesday": ["09:00", "17:00"]
            },
            "preferences": {
                "max_assignments_per_week": 3
            },
            "metadata": {
                "department": "test",
                "employee_id": "TEST001"
            }
        },
        {
            "name": "Test User 2",
            "email": "test2@example.com",
            "skills": ["maintenance", "technical"],
            "availability": {
                "wednesday": ["10:00", "18:00"]
            },
            "preferences": {
                "max_assignments_per_week": 2
            }
        }
    ]"#;
    
    std::fs::write(&participants_file, json_content)?;
    
    // Run JSON migration
    let json_migrator = JsonMigrationManager::new(repo_manager.clone());
    let stats = json_migrator.migrate_from_directory(temp_dir.path()).await?;
    
    // Verify results
    assert_eq!(stats.participants_created, 2);
    assert_eq!(stats.targets_created, 0);
    assert!(stats.errors.is_empty());
    
    // Verify participants were created in database
    let participant1 = repo_manager.participants.find_by_name("Test User 1").await?;
    assert!(participant1.is_some());
    let p1 = participant1.unwrap();
    assert_eq!(p1.email, Some("test1@example.com".to_string()));
    assert_eq!(p1.phone, Some("+1-555-0001".to_string()));
    
    let participant2 = repo_manager.participants.find_by_name("Test User 2").await?;
    assert!(participant2.is_some());
    let p2 = participant2.unwrap();
    assert_eq!(p2.email, Some("test2@example.com".to_string()));
    
    Ok(())
}

#[tokio::test]
async fn test_json_migration_assignment_targets() -> Result<()> {
    // Setup test database
    let config = AppConfig::from_env().expect("Failed to load config");
    let db_manager = DatabaseManager::new(&config.database).await?;
    let repo_manager = RepositoryManager::new(db_manager.pool().clone());
    
    // Run migrations
    run_migrations(db_manager.pool()).await?;
    
    // Seed admin user first
    let seed_manager = SeedManager::new(repo_manager.clone(), db_manager.pool().clone());
    seed_manager.seed_users().await?;
    
    // Create temporary JSON file
    let temp_dir = TempDir::new()?;
    let targets_file = temp_dir.path().join("assignment_targets.json");
    
    let json_content = r#"[
        {
            "name": "Test Room Cleaning",
            "description": "Clean the test room thoroughly",
            "required_count": 2,
            "required_skills": ["cleaning", "organizing"],
            "constraints": {
                "time_estimate_minutes": 45,
                "difficulty_level": "easy"
            },
            "metadata": {
                "location": "Test Floor, Room T01",
                "equipment": ["vacuum", "cleaning_supplies"]
            }
        },
        {
            "name": "Test Equipment Maintenance",
            "description": "Maintain test equipment",
            "required_count": 1,
            "required_skills": ["technical", "maintenance"],
            "constraints": {
                "time_estimate_minutes": 120,
                "difficulty_level": "hard"
            }
        }
    ]"#;
    
    std::fs::write(&targets_file, json_content)?;
    
    // Run JSON migration
    let json_migrator = JsonMigrationManager::new(repo_manager.clone());
    let stats = json_migrator.migrate_from_directory(temp_dir.path()).await?;
    
    // Verify results
    assert_eq!(stats.participants_created, 0);
    assert_eq!(stats.targets_created, 2);
    assert!(stats.errors.is_empty());
    
    // Verify targets were created in database
    let target1 = repo_manager.assignments.find_target_by_name("Test Room Cleaning").await?;
    assert!(target1.is_some());
    let t1 = target1.unwrap();
    assert_eq!(t1.required_count, 2);
    assert_eq!(t1.description, Some("Clean the test room thoroughly".to_string()));
    
    let target2 = repo_manager.assignments.find_target_by_name("Test Equipment Maintenance").await?;
    assert!(target2.is_some());
    let t2 = target2.unwrap();
    assert_eq!(t2.required_count, 1);
    
    Ok(())
}

#[tokio::test]
async fn test_json_migration_malformed_data() -> Result<()> {
    // Setup test database
    let config = AppConfig::from_env().expect("Failed to load config");
    let db_manager = DatabaseManager::new(&config.database).await?;
    let repo_manager = RepositoryManager::new(db_manager.pool().clone());
    
    // Run migrations
    run_migrations(db_manager.pool()).await?;
    
    // Seed admin user first
    let seed_manager = SeedManager::new(repo_manager.clone(), db_manager.pool().clone());
    seed_manager.seed_users().await?;
    
    // Create temporary JSON file with malformed data
    let temp_dir = TempDir::new()?;
    let participants_file = temp_dir.path().join("participants.json");
    
    // Invalid JSON structure
    let json_content = r#"[
        {
            "name": "Valid User",
            "email": "valid@example.com"
        },
        {
            "invalid_field": "This should cause an error"
        }
    ]"#;
    
    std::fs::write(&participants_file, json_content)?;
    
    // Run JSON migration
    let json_migrator = JsonMigrationManager::new(repo_manager.clone());
    let result = json_migrator.migrate_from_directory(temp_dir.path()).await;
    
    // Should handle malformed data gracefully
    match result {
        Ok(stats) => {
            // Should have some errors
            assert!(!stats.errors.is_empty() || stats.participants_created < 2);
        }
        Err(_) => {
            // Or fail entirely, which is also acceptable
        }
    }
    
    Ok(())
}

#[tokio::test]
async fn test_json_migration_missing_files() -> Result<()> {
    // Setup test database
    let config = AppConfig::from_env().expect("Failed to load config");
    let db_manager = DatabaseManager::new(&config.database).await?;
    let repo_manager = RepositoryManager::new(db_manager.pool().clone());
    
    // Create empty temporary directory
    let temp_dir = TempDir::new()?;
    
    // Run JSON migration on empty directory
    let json_migrator = JsonMigrationManager::new(repo_manager.clone());
    let stats = json_migrator.migrate_from_directory(temp_dir.path()).await?;
    
    // Should complete successfully with no changes
    assert_eq!(stats.participants_created, 0);
    assert_eq!(stats.targets_created, 0);
    assert!(stats.errors.is_empty());
    
    Ok(())
}
