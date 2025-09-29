# VividShift Development Guide

## Overview
Comprehensive development guide for contributing to VividShift Generic Assignment Engine, covering local setup, development workflow, testing, and contribution guidelines.

**Target Audience:** Developers, contributors, maintainers

## Development Environment Setup

### Prerequisites
- **Rust:** 1.70+ with cargo
- **PostgreSQL:** 15+ for database development
- **Docker & Docker Compose:** For containerized development
- **Git:** For version control
- **IDE:** VS Code with rust-analyzer recommended

### Local Development Setup
```bash
# Clone repository
git clone https://github.com/onelrian/vividshift.git
cd vividshift

# Install Rust toolchain
rustup update stable
rustup component add clippy rustfmt

# Install development dependencies
cargo install sqlx-cli --no-default-features --features postgres
cargo install cargo-watch
cargo install cargo-audit

# Setup environment
cp .env.example .env.dev
# Edit .env.dev with development settings
```

### Database Setup
```bash
# Start PostgreSQL with Docker
docker-compose up -d db

# Run migrations
sqlx migrate run

# Seed development data
cargo run --bin db_cli seed
```

### Running the Application
```bash
# Development mode with hot reload
cargo watch -x run

# Standard development run
cargo run

# Release mode
cargo run --release

# With specific environment
ENVIRONMENT=dev cargo run
```

## Project Structure

### Directory Layout
```
backend/
├── src/
│   ├── api/                 # HTTP API endpoints
│   ├── auth/               # Authentication & authorization
│   ├── database/           # Database layer & repositories
│   ├── engines/            # Assignment & validation engines
│   ├── models/             # Data models & types
│   ├── services/           # Business logic services
│   ├── config.rs           # Configuration management
│   ├── main.rs             # Application entry point
│   └── lib.rs              # Library root
├── config/                 # Configuration files
├── migrations/             # Database migrations
├── tests/                  # Integration tests
├── Cargo.toml             # Rust dependencies
└── Dockerfile             # Container configuration
```

### Key Modules

#### API Layer (`src/api/`)
```rust
// HTTP endpoints and routing
pub mod auth;      // Authentication endpoints
pub mod health;    // Health check endpoints
pub mod entities;  // Entity management
pub mod assignments; // Assignment generation
```

#### Database Layer (`src/database/`)
```rust
// Repository pattern implementation
pub mod connection;    // Connection management
pub mod repositories;  // Data access layer
pub mod models;       // Database models
pub mod migrations;   // Schema management
```

#### Business Logic (`src/engines/`)
```rust
// Assignment and validation engines
pub mod assignment;   // Assignment strategies
pub mod validation;   // Validation rules
pub mod distribution; // Distribution algorithms
```

## Development Workflow

### Code Style and Standards
```bash
# Format code
cargo fmt

# Lint code
cargo clippy -- -D warnings

# Check for security vulnerabilities
cargo audit

# Run all checks
cargo fmt && cargo clippy -- -D warnings && cargo test
```

### Testing Strategy

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_assignment_generation() {
        let participants = vec![/* test data */];
        let targets = vec![/* test data */];
        
        let result = generate_assignments(&participants, &targets);
        
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), expected_count);
    }
}
```

#### Integration Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test database::

# Run with output
cargo test -- --nocapture

# Test with database
DATABASE_URL=postgresql://localhost/vividshift_test cargo test
```

#### Database Tests
```rust
#[sqlx::test]
async fn test_user_repository(pool: PgPool) -> sqlx::Result<()> {
    let repo = UserRepository::new(pool);
    
    let user = CreateUser {
        username: "testuser".to_string(),
        email: "test@example.com".to_string(),
        password: "password123".to_string(),
    };
    
    let created = repo.create(&user).await?;
    assert_eq!(created.username, "testuser");
    
    Ok(())
}
```

### Performance Testing
```bash
# Benchmark tests
cargo bench

# Load testing with custom tool
cargo run --bin load_test --concurrent=10 --requests=1000

# Memory profiling
cargo run --bin memory_profiler
```

## Contributing Guidelines

### Git Workflow
```bash
# Create feature branch
git checkout -b feature/new-assignment-strategy

# Make changes and commit
git add .
git commit -m "feat: add skill-based assignment strategy"

# Push and create PR
git push origin feature/new-assignment-strategy
```

### Commit Message Format
```
type(scope): description

feat(engines): add new balanced rotation strategy
fix(auth): resolve JWT token expiration issue
docs(api): update endpoint documentation
test(database): add repository integration tests
```

### Pull Request Process
1. **Create Feature Branch:** Branch from `main`
2. **Implement Changes:** Follow coding standards
3. **Add Tests:** Ensure adequate test coverage
4. **Update Documentation:** Update relevant docs
5. **Submit PR:** Include description and testing notes
6. **Code Review:** Address reviewer feedback
7. **Merge:** Squash and merge after approval

## Adding New Features

### Creating Assignment Strategies
```rust
use async_trait::async_trait;
use crate::services::rule_engine::{AssignmentStrategy, StrategyConfig};

pub struct CustomStrategy;

#[async_trait]
impl AssignmentStrategy for CustomStrategy {
    async fn execute(
        &self,
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &StrategyConfig,
    ) -> Result<Vec<Assignment>, EngineError> {
        // Implementation here
        todo!()
    }
    
    fn name(&self) -> &str {
        "custom_strategy"
    }
    
    fn description(&self) -> &str {
        "Custom assignment strategy implementation"
    }
    
    fn validate_config(&self, config: &StrategyConfig) -> Result<(), EngineError> {
        // Validate strategy parameters
        Ok(())
    }
}

// Register in main.rs
rule_engine.register_strategy(CustomStrategy);
```

### Adding Validation Rules
```rust
use async_trait::async_trait;
use crate::services::rule_engine::{ValidationRule, ValidationResult};

pub struct CustomValidator;

#[async_trait]
impl ValidationRule for CustomValidator {
    async fn validate(
        &self,
        assignments: &[Assignment],
        participants: &[GenericEntity],
        targets: &[GenericEntity],
        config: &HashMap<String, serde_json::Value>,
    ) -> Result<ValidationResult, EngineError> {
        // Validation logic here
        Ok(ValidationResult::success())
    }
    
    fn name(&self) -> &str {
        "custom_validator"
    }
    
    fn severity(&self) -> ValidationSeverity {
        ValidationSeverity::Warning
    }
}
```

### Adding API Endpoints
```rust
use axum::{Json, extract::State};
use crate::api::AppState;

pub async fn new_endpoint(
    State(state): State<AppState>,
    Json(payload): Json<RequestPayload>,
) -> Result<Json<ResponsePayload>, ApiError> {
    // Endpoint implementation
    let result = state.service.process_request(payload).await?;
    Ok(Json(result))
}

// Add to router in api/mod.rs
pub fn create_router() -> Router<AppState> {
    Router::new()
        .route("/new-endpoint", post(new_endpoint))
        // ... other routes
}
```

## Database Development

### Creating Migrations
```bash
# Create new migration
sqlx migrate add create_new_table

# Edit migration file
# migrations/TIMESTAMP_create_new_table.sql
CREATE TABLE new_table (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(200) NOT NULL,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

# Create rollback migration
# migrations/TIMESTAMP_create_new_table.down.sql
DROP TABLE IF EXISTS new_table;

# Run migration
sqlx migrate run
```

### Repository Development
```rust
use sqlx::PgPool;
use uuid::Uuid;

pub struct NewTableRepository {
    pool: PgPool,
}

impl NewTableRepository {
    pub fn new(pool: PgPool) -> Self {
        Self { pool }
    }
    
    pub async fn create(&self, data: &CreateData) -> Result<Entity, sqlx::Error> {
        let entity = sqlx::query_as!(
            Entity,
            "INSERT INTO new_table (name) VALUES ($1) RETURNING *",
            data.name
        )
        .fetch_one(&self.pool)
        .await?;
        
        Ok(entity)
    }
    
    pub async fn find_by_id(&self, id: Uuid) -> Result<Option<Entity>, sqlx::Error> {
        let entity = sqlx::query_as!(
            Entity,
            "SELECT * FROM new_table WHERE id = $1",
            id
        )
        .fetch_optional(&self.pool)
        .await?;
        
        Ok(entity)
    }
}
```

## Configuration Development

### Adding Configuration Options
```rust
// src/config.rs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NewFeatureConfig {
    pub enabled: bool,
    pub parameter: String,
    pub timeout_seconds: u64,
}

impl Default for NewFeatureConfig {
    fn default() -> Self {
        Self {
            enabled: false,
            parameter: "default_value".to_string(),
            timeout_seconds: 30,
        }
    }
}

// Add to AppConfig
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AppConfig {
    // ... existing fields
    pub new_feature: NewFeatureConfig,
}
```

### Environment Variable Support
```rust
// Support environment variables with VIVIDSHIFT_ prefix
pub fn load_config() -> Result<AppConfig, ConfigError> {
    let mut config = Config::builder()
        .add_source(File::with_name("config/default"))
        .add_source(File::with_name(&format!("config/{}", env)).required(false))
        .add_source(Environment::with_prefix("VIVIDSHIFT").separator("_"))
        .build()?;
    
    config.try_deserialize()
}
```

## Testing and Quality Assurance

### Test Coverage
```bash
# Install coverage tool
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html --output-dir coverage/

# View coverage
open coverage/tarpaulin-report.html
```

### Performance Benchmarks
```rust
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_assignment_generation(c: &mut Criterion) {
    let participants = create_test_participants(100);
    let targets = create_test_targets(20);
    
    c.bench_function("assignment_generation", |b| {
        b.iter(|| {
            generate_assignments(
                black_box(&participants),
                black_box(&targets)
            )
        })
    });
}

criterion_group!(benches, benchmark_assignment_generation);
criterion_main!(benches);
```

### Load Testing
```rust
// tests/load_test.rs
use tokio::time::{sleep, Duration};
use reqwest::Client;

#[tokio::test]
async fn load_test_assignment_generation() {
    let client = Client::new();
    let mut handles = vec![];
    
    for _ in 0..10 {
        let client = client.clone();
        let handle = tokio::spawn(async move {
            for _ in 0..100 {
                let response = client
                    .post("http://localhost:8080/api/assignments/generate")
                    .json(&test_payload())
                    .send()
                    .await?;
                
                assert!(response.status().is_success());
                sleep(Duration::from_millis(10)).await;
            }
            Ok::<(), Box<dyn std::error::Error>>(())
        });
        handles.push(handle);
    }
    
    for handle in handles {
        handle.await??;
    }
}
```

## Debugging and Profiling

### Debug Configuration
```bash
# Enable debug logging
export RUST_LOG=debug,sqlx=info

# Enable backtraces
export RUST_BACKTRACE=1

# Full backtraces
export RUST_BACKTRACE=full
```

### Performance Profiling
```bash
# CPU profiling with perf
perf record --call-graph=dwarf cargo run --release
perf report

# Memory profiling with valgrind
valgrind --tool=massif cargo run
ms_print massif.out.*
```

### Database Query Analysis
```sql
-- Enable query logging
ALTER SYSTEM SET log_statement = 'all';
ALTER SYSTEM SET log_min_duration_statement = 100;
SELECT pg_reload_conf();

-- Analyze slow queries
SELECT query, mean_time, calls 
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;
```

## Release Process

### Version Management
```bash
# Update version in Cargo.toml
# Create release branch
git checkout -b release/v1.2.0

# Update CHANGELOG.md
# Tag release
git tag -a v1.2.0 -m "Release version 1.2.0"

# Push tag
git push origin v1.2.0
```

### Build and Deployment
```bash
# Build release binary
cargo build --release

# Create Docker image
docker build -t vividshift:v1.2.0 .

# Run integration tests
cargo test --release

# Deploy to staging
./scripts/deploy-staging.sh v1.2.0
```

## References
- [Architecture Guide](ARCHITECTURE.md) - System design overview
- [API Reference](API_REFERENCE.md) - API development guidelines
- [Database Schema](database/SCHEMA.md) - Database development
- [Configuration Guide](CONFIGURATION.md) - Configuration management
- [Rust Documentation](https://doc.rust-lang.org/) - Rust language reference
