use anyhow::Result;
use std::sync::Arc;
use testcontainers::{clients::Cli, images::postgres::Postgres, Container};
use tokio::sync::OnceCell;
use uuid::Uuid;

use vividshift_backend::{
    config::DatabaseConfig,
    database::{
        DatabaseManager, RepositoryManager, SeedManager, MigrationManager,
        models::{CreateUser, CreateParticipant, CreateAssignmentTarget, CreateAssignment},
        run_migrations, health_check,
    },
};

static DOCKER: OnceCell<Cli> = OnceCell::const_new();

async fn get_docker() -> &'static Cli {
    DOCKER.get_or_init(|| async { Cli::default() }).await
}

struct TestDatabase {
    _container: Container<'static, Postgres>,
    db_manager: Arc<DatabaseManager>,
    repo_manager: Arc<RepositoryManager>,
}

impl TestDatabase {
    async fn new() -> Result<Self> {
        let docker = get_docker().await;
        let postgres_image = Postgres::default();
        let container = docker.run(postgres_image);
        
        let port = container.get_host_port_ipv4(5432);
        let database_url = format!("postgresql://postgres:postgres@localhost:{}/postgres", port);
        
        let config = DatabaseConfig {
            url: database_url,
            max_connections: 5,
            min_connections: 1,
            connect_timeout: 30,
        };
        
        let db_manager = Arc::new(DatabaseManager::new(&config).await?);
        
        // Run migrations
        run_migrations(db_manager.pool()).await?;
        
        // Verify health
        health_check(db_manager.pool()).await?;
        
        let repo_manager = Arc::new(RepositoryManager::new(db_manager.pool_clone()));
        
        Ok(Self {
            _container: container,
            db_manager,
            repo_manager,
        })
    }
}

#[tokio::test]
async fn test_database_connection_and_health() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Test health check
    health_check(test_db.db_manager.pool()).await?;
    
    // Test pool statistics
    let stats = test_db.db_manager.get_pool_stats();
    assert!(stats.size > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_migration_system() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    let migration_manager = MigrationManager::new(test_db.db_manager.pool_clone());
    
    // Check migration status
    let migrations = migration_manager.check_migration_status().await?;
    assert!(!migrations.is_empty());
    assert!(migrations.iter().all(|m| m.success));
    
    // Validate schema
    let validation = migration_manager.validate_schema().await?;
    assert!(validation.is_valid, "Schema validation failed: {:?}", validation);
    
    Ok(())
}

#[tokio::test]
async fn test_user_repository_crud() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Create user
    let create_user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        role: Some("user".to_string()),
    };
    
    let user = test_db.repo_manager.users.create(&create_user).await?;
    assert_eq!(user.username, "testuser");
    assert_eq!(user.email, "test@example.com");
    assert_eq!(user.role, "user");
    assert!(user.is_active);
    
    // Find by ID
    let found_user = test_db.repo_manager.users.find_by_id(user.id).await?;
    assert!(found_user.is_some());
    assert_eq!(found_user.unwrap().username, "testuser");
    
    // Find by username
    let found_user = test_db.repo_manager.users.find_by_username("testuser").await?;
    assert!(found_user.is_some());
    
    // Find by email
    let found_user = test_db.repo_manager.users.find_by_email("test@example.com").await?;
    assert!(found_user.is_some());
    
    // Update last login
    test_db.repo_manager.users.update_last_login(user.id).await?;
    
    // Count users
    let count = test_db.repo_manager.users.count().await?;
    assert!(count > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_participant_repository_crud() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Create a user first for created_by
    let create_user = CreateUser {
        username: "admin".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        role: Some("admin".to_string()),
    };
    let user = test_db.repo_manager.users.create(&create_user).await?;
    
    // Create participant
    let create_participant = CreateParticipant {
        name: "John Doe".to_string(),
        email: Some("john@example.com".to_string()),
        phone: Some("+1-555-0123".to_string()),
        skills: Some(serde_json::json!(["rust", "postgresql"])),
        availability: Some(serde_json::json!({"monday": true, "tuesday": false})),
        preferences: Some(serde_json::json!({"preferred_time": "morning"})),
        metadata: Some(serde_json::json!({"experience": "senior"})),
    };
    
    let participant = test_db.repo_manager.participants.create(&create_participant, Some(user.id)).await?;
    assert_eq!(participant.name, "John Doe");
    assert_eq!(participant.email, Some("john@example.com".to_string()));
    assert!(participant.is_active);
    
    // Find by ID
    let found = test_db.repo_manager.participants.find_by_id(participant.id).await?;
    assert!(found.is_some());
    
    // Find by name
    let found = test_db.repo_manager.participants.find_by_name("John Doe").await?;
    assert!(found.is_some());
    
    // Find by email
    let found = test_db.repo_manager.participants.find_by_email("john@example.com").await?;
    assert!(found.is_some());
    
    // Find all active
    let active = test_db.repo_manager.participants.find_all_active().await?;
    assert!(!active.is_empty());
    
    // Find by skill
    let skilled = test_db.repo_manager.participants.find_by_skill("rust").await?;
    assert!(!skilled.is_empty());
    
    // Count active
    let count = test_db.repo_manager.participants.count_active().await?;
    assert!(count > 0);
    
    Ok(())
}

#[tokio::test]
async fn test_assignment_repository_crud() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Create user
    let create_user = CreateUser {
        username: "admin".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        role: Some("admin".to_string()),
    };
    let user = test_db.repo_manager.users.create(&create_user).await?;
    
    // Create assignment target
    let create_target = CreateAssignmentTarget {
        name: "Test Task".to_string(),
        description: Some("A test task for validation".to_string()),
        required_count: 2,
        required_skills: Some(serde_json::json!(["testing"])),
        constraints: Some(serde_json::json!({"priority": "high"})),
        metadata: Some(serde_json::json!({"category": "development"})),
    };
    
    let target = test_db.repo_manager.assignments.create_target(&create_target, Some(user.id)).await?;
    assert_eq!(target.name, "Test Task");
    assert_eq!(target.required_count, 2);
    
    // Create participant
    let create_participant = CreateParticipant {
        name: "Test Participant".to_string(),
        email: Some("participant@example.com".to_string()),
        phone: None,
        skills: Some(serde_json::json!(["testing"])),
        availability: Some(serde_json::json!({})),
        preferences: Some(serde_json::json!({})),
        metadata: Some(serde_json::json!({})),
    };
    let participant = test_db.repo_manager.participants.create(&create_participant, Some(user.id)).await?;
    
    // Create assignment
    let create_assignment = CreateAssignment {
        name: Some("Test Assignment".to_string()),
        description: Some("A test assignment".to_string()),
        assignment_date: None,
        strategy_used: "balanced_rotation".to_string(),
        configuration: Some(serde_json::json!({"test": true})),
        metadata: Some(serde_json::json!({"version": "1.0"})),
    };
    
    let assignment = test_db.repo_manager.assignments.create_assignment(&create_assignment, Some(user.id)).await?;
    assert_eq!(assignment.strategy_used, "balanced_rotation");
    
    // Add assignment detail
    let detail = test_db.repo_manager.assignments.add_assignment_detail(
        assignment.id,
        participant.id,
        target.id,
        Some(1),
        None
    ).await?;
    assert_eq!(detail.assignment_id, assignment.id);
    assert_eq!(detail.participant_id, participant.id);
    assert_eq!(detail.target_id, target.id);
    
    // Find assignment details
    let details = test_db.repo_manager.assignments.find_assignment_details(assignment.id).await?;
    assert_eq!(details.len(), 1);
    
    // Add history entry
    let changes = serde_json::json!({"action": "created", "details": "Initial assignment"});
    let history = test_db.repo_manager.assignments.add_history_entry(
        assignment.id,
        "create",
        &changes,
        Some(user.id)
    ).await?;
    assert_eq!(history.action, "create");
    
    Ok(())
}

#[tokio::test]
async fn test_seeding_system() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    let seed_manager = SeedManager::new((*test_db.repo_manager).clone(), test_db.db_manager.pool_clone());
    
    // Run seeding
    seed_manager.seed_all(false).await?;
    
    // Verify seeded data
    let users = test_db.repo_manager.users.find_all_active().await?;
    assert!(users.len() >= 3); // admin, user, viewer
    
    let participants = test_db.repo_manager.participants.find_all_active().await?;
    assert!(participants.len() >= 4); // Alice, Bob, Carol, David
    
    let targets = test_db.repo_manager.assignments.find_all_active_targets().await?;
    assert!(targets.len() >= 7); // Parlor, Frontyard, etc.
    
    // Verify specific seeded users
    let admin = test_db.repo_manager.users.find_by_username("admin").await?;
    assert!(admin.is_some());
    assert_eq!(admin.unwrap().role, "admin");
    
    Ok(())
}

#[tokio::test]
async fn test_connection_pool_behavior() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Test concurrent connections
    let mut handles = vec![];
    
    for i in 0..10 {
        let repo = test_db.repo_manager.clone();
        let handle = tokio::spawn(async move {
            let create_user = CreateUser {
                username: format!("user_{}", i),
                email: format!("user_{}@example.com", i),
                password_hash: "hashed_password".to_string(),
                role: Some("user".to_string()),
            };
            repo.users.create(&create_user).await
        });
        handles.push(handle);
    }
    
    // Wait for all operations to complete
    for handle in handles {
        let result = handle.await?;
        assert!(result.is_ok());
    }
    
    // Verify all users were created
    let count = test_db.repo_manager.users.count().await?;
    assert!(count >= 10);
    
    Ok(())
}

#[tokio::test]
async fn test_transaction_rollback() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Start a transaction
    let mut tx = test_db.db_manager.pool().begin().await?;
    
    // Create a user within the transaction
    let user = sqlx::query!(
        "INSERT INTO users (username, email, password_hash, role) VALUES ($1, $2, $3, $4) RETURNING id",
        "tx_user",
        "tx@example.com",
        "hashed",
        "user"
    )
    .fetch_one(&mut *tx)
    .await?;
    
    // Verify user exists within transaction
    let found = sqlx::query!(
        "SELECT id FROM users WHERE id = $1",
        user.id
    )
    .fetch_optional(&mut *tx)
    .await?;
    assert!(found.is_some());
    
    // Rollback transaction
    tx.rollback().await?;
    
    // Verify user doesn't exist after rollback
    let found = test_db.repo_manager.users.find_by_id(user.id).await?;
    assert!(found.is_none());
    
    Ok(())
}

#[tokio::test]
async fn test_jsonb_queries() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    
    // Create user for created_by
    let create_user = CreateUser {
        username: "admin".to_string(),
        email: "admin@example.com".to_string(),
        password_hash: "hashed_password".to_string(),
        role: Some("admin".to_string()),
    };
    let user = test_db.repo_manager.users.create(&create_user).await?;
    
    // Create participant with specific skills
    let create_participant = CreateParticipant {
        name: "Skilled Worker".to_string(),
        email: Some("skilled@example.com".to_string()),
        phone: None,
        skills: Some(serde_json::json!(["rust", "postgresql", "docker"])),
        availability: Some(serde_json::json!({"monday": true, "tuesday": true})),
        preferences: Some(serde_json::json!({"shift": "morning"})),
        metadata: Some(serde_json::json!({"certification": "senior"})),
    };
    
    let participant = test_db.repo_manager.participants.create(&create_participant, Some(user.id)).await?;
    
    // Test skill-based queries
    let rust_developers = test_db.repo_manager.participants.find_by_skill("rust").await?;
    assert!(!rust_developers.is_empty());
    assert!(rust_developers.iter().any(|p| p.id == participant.id));
    
    let postgresql_users = test_db.repo_manager.participants.find_by_skill("postgresql").await?;
    assert!(!postgresql_users.is_empty());
    
    // Test availability queries
    let available_monday = test_db.repo_manager.participants.find_available("monday").await?;
    assert!(!available_monday.is_empty());
    
    Ok(())
}

#[tokio::test]
async fn test_database_statistics() -> Result<()> {
    let test_db = TestDatabase::new().await?;
    let migration_manager = MigrationManager::new(test_db.db_manager.pool_clone());
    
    // Get database statistics
    let stats = migration_manager.get_database_stats().await?;
    
    assert!(!stats.database_size.is_empty());
    assert!(stats.connection_count > 0);
    assert!(stats.table_count > 0);
    assert!(!stats.table_stats.is_empty());
    
    // Verify we have expected tables
    let table_names: Vec<String> = stats.table_stats.iter()
        .map(|t| t.table_name.clone())
        .collect();
    
    assert!(table_names.contains(&"users".to_string()));
    assert!(table_names.contains(&"participants".to_string()));
    assert!(table_names.contains(&"assignments".to_string()));
    
    Ok(())
}
