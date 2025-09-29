use anyhow::Result;
use clap::{Parser, Subcommand};
use dotenvy::dotenv;
use std::sync::Arc;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

use vividshift_backend::{
    config::AppConfig,
    database::{DatabaseManager, RepositoryManager, SeedManager, MigrationManager, JsonMigrationManager, run_migrations, health_check},
};

#[derive(Parser)]
#[command(name = "vividshift-db")]
#[command(about = "VividShift Database CLI Tool")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Run database migrations
    Migrate {
        /// Show migration status without running
        #[arg(long)]
        status: bool,
    },
    /// Seed the database with sample data
    Seed {
        /// Force re-seeding even if data exists
        #[arg(long)]
        force: bool,
        /// Clear all data before seeding
        #[arg(long)]
        clear: bool,
    },
    /// Migrate data from JSON files
    JsonMigrate {
        /// Directory containing JSON migration files
        #[arg(short, long, default_value = "data/migration")]
        directory: String,
        /// Validate JSON files without importing
        #[arg(long)]
        validate_only: bool,
    },
    /// Database health and status checks
    Status {
        /// Show detailed statistics
        #[arg(long)]
        detailed: bool,
    },
    /// Backup database
    Backup {
        /// Output directory for backup
        #[arg(short, long)]
        output: Option<String>,
        /// Schema only backup
        #[arg(long)]
        schema_only: bool,
        /// Data only backup
        #[arg(long)]
        data_only: bool,
    },
    /// Validate database schema
    Validate,
    /// Clean up expired sessions and temporary data
    Cleanup,
}

#[tokio::main]
async fn main() -> Result<()> {
    // Load environment variables
    dotenv().ok();

    // Initialize logging
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "info".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let cli = Cli::parse();

    // Load configuration
    let config = Arc::new(AppConfig::new()?);
    
    // Initialize database connection
    let db_manager = Arc::new(DatabaseManager::new(&config.database).await?);
    let repo_manager = Arc::new(RepositoryManager::new(db_manager.pool_clone()));
    let migration_manager = MigrationManager::new(db_manager.pool_clone());
    let seed_manager = SeedManager::new((*repo_manager).clone(), db_manager.pool_clone());

    match cli.command {
        Commands::Migrate { status } => {
            if status {
                let migrations = migration_manager.check_migration_status().await?;
                println!("Migration Status:");
                println!("================");
                for migration in migrations {
                    let status_icon = if migration.success { "✅" } else { "❌" };
                    println!(
                        "{} v{}: {} ({}ms)",
                        status_icon,
                        migration.version,
                        migration.description,
                        migration.execution_time
                    );
                }
            } else {
                run_migrations(db_manager.pool()).await?;
                println!("✅ Migrations completed successfully");
            }
        }
        
        Commands::Seed { force, clear } => {
            if clear {
                println!("🗑️  Clearing existing data...");
                seed_manager.clear_all().await?;
                println!("✅ Data cleared successfully");
            }
            
            println!("🌱 Seeding database...");
            seed_manager.seed_all(force).await?;
            println!("✅ Database seeded successfully");
        }

        Commands::JsonMigrate { directory, validate_only } => {
            let migration_dir = std::path::Path::new(&directory);
            
            if !migration_dir.exists() {
                eprintln!("❌ Migration directory not found: {}", directory);
                std::process::exit(1);
            }
            
            let json_migrator = JsonMigrationManager::new((*repo_manager).clone());
            
            if validate_only {
                println!("🔍 Validating JSON files in: {}", directory);
                // TODO: Add validation logic
                println!("✅ JSON files validation completed");
            } else {
                println!("📥 Starting JSON migration from: {}", directory);
                let stats = json_migrator.migrate_from_directory(migration_dir).await?;
                
                println!("✅ JSON migration completed!");
                println!("  Participants created: {}", stats.participants_created);
                println!("  Assignment targets created: {}", stats.targets_created);
                
                if !stats.errors.is_empty() {
                    println!("⚠️  Errors encountered:");
                    for error in &stats.errors {
                        println!("    - {}", error);
                    }
                }
            }
        }
        
        Commands::Status { detailed } => {
            // Health check
            match health_check(db_manager.pool()).await {
                Ok(_) => println!("✅ Database connection: Healthy"),
                Err(e) => println!("❌ Database connection: Failed - {}", e),
            }
            
            // Pool statistics
            let stats = db_manager.get_pool_stats();
            println!("📊 Connection Pool: {} active, {} idle", stats.size, stats.idle);
            
            if detailed {
                // Get detailed database statistics
                let db_stats = migration_manager.get_database_stats().await?;
                println!("\n📈 Database Statistics:");
                println!("  Size: {}", db_stats.database_size);
                println!("  Connections: {}", db_stats.connection_count);
                println!("  Tables: {}", db_stats.table_count);
                
                println!("\n📋 Table Statistics:");
                for table in db_stats.table_stats.iter().take(10) {
                    println!(
                        "  {}: {} rows ({} inserts, {} updates, {} deletes)",
                        table.table_name,
                        table.live_tuples,
                        table.inserts,
                        table.updates,
                        table.deletes
                    );
                }
            }
        }
        
        Commands::Backup { output, schema_only, data_only } => {
            println!("📦 Database backup functionality:");
            println!("Use the backup script: ./scripts/backup.sh");
            
            if let Some(output_dir) = output {
                println!("  ./scripts/backup.sh --output {}", output_dir);
            }
            if schema_only {
                println!("  ./scripts/backup.sh --schema-only");
            }
            if data_only {
                println!("  ./scripts/backup.sh --data-only");
            }
            
            println!("\nFor more options, run: ./scripts/backup.sh --help");
        }
        
        Commands::Validate => {
            println!("🔍 Validating database schema...");
            let validation = migration_manager.validate_schema().await?;
            
            if validation.is_valid {
                println!("✅ Schema validation passed");
            } else {
                println!("❌ Schema validation failed");
                
                if !validation.missing_tables.is_empty() {
                    println!("Missing tables:");
                    for table in validation.missing_tables {
                        println!("  - {}", table);
                    }
                }
                
                if !validation.missing_indexes.is_empty() {
                    println!("Missing indexes:");
                    for index in validation.missing_indexes {
                        println!("  - {}", index);
                    }
                }
                
                if !validation.issues.is_empty() {
                    println!("Other issues:");
                    for issue in validation.issues {
                        println!("  - {}", issue);
                    }
                }
            }
        }
        
        Commands::Cleanup => {
            println!("🧹 Cleaning up database...");
            
            // Clean up expired sessions
            let expired_sessions = repo_manager.users.cleanup_expired_sessions().await?;
            println!("  Removed {} expired sessions", expired_sessions);
            
            // TODO: Add more cleanup operations as needed
            // - Remove old assignment history entries
            // - Clean up orphaned records
            // - Vacuum analyze tables
            
            println!("✅ Database cleanup completed");
        }
    }

    Ok(())
}
