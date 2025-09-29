# Database System Overview

## Overview
VividShift uses PostgreSQL as its primary database with SQLx for type-safe database operations. The system supports authentication, domain-agnostic entity management, and comprehensive assignment tracking.

**Target Audience:** Backend developers, database administrators

## Architecture

```mermaid
graph TB
    App[Application] --> Pool[Connection Pool]
    Pool --> PG[(PostgreSQL)]
    
    subgraph "Repository Layer"
        UserRepo[UserRepository]
        ParticipantRepo[ParticipantRepository]
        AssignmentRepo[AssignmentRepository]
        DomainRepo[DomainRepository]
    end
    
    App --> UserRepo
    App --> ParticipantRepo
    App --> AssignmentRepo
    App --> DomainRepo
    
    UserRepo --> Pool
    ParticipantRepo --> Pool
    AssignmentRepo --> Pool
    DomainRepo --> Pool
    
    subgraph "Database Features"
        JSONB[JSONB Support]
        UUID[UUID Primary Keys]
        Indexes[Strategic Indexing]
        Triggers[Automatic Timestamps]
    end
    
    PG --> JSONB
    PG --> UUID
    PG --> Indexes
    PG --> Triggers
```

## Key Features

### Repository Pattern
Type-safe database operations with compile-time query validation:

```rust
// Example usage
let user = repo_manager.users.find_by_username("admin").await?;
let participants = repo_manager.participants.find_all_active().await?;
let assignment = repo_manager.assignments.create_assignment(&request, user_id).await?;
```

### Connection Management
- **Connection Pooling:** Configurable min/max connections
- **Health Monitoring:** Automatic connection validation
- **Timeout Handling:** Configurable acquire and idle timeouts
- **SSL/TLS Support:** Production-ready security

### Schema Design
- **UUID Primary Keys:** Distributed system compatibility
- **JSONB Columns:** Flexible schema for skills, preferences, metadata
- **Audit Trails:** Complete change tracking with timestamps
- **Referential Integrity:** Proper foreign key relationships

## Quick Start

### Database Setup
```bash
# Start PostgreSQL with Docker
docker-compose up -d db

# Run migrations
cargo run --bin db_cli migrate

# Seed development data
cargo run --bin db_cli seed
```

### Connection Configuration
```bash
# Environment variables
export VIVIDSHIFT_DATABASE_URL="postgresql://postgres:password@localhost:5432/vividshift_dev"
export VIVIDSHIFT_DATABASE_MAX_CONNECTIONS=10
export VIVIDSHIFT_DATABASE_MIN_CONNECTIONS=1
```

## Documentation Structure
- [Schema Design](SCHEMA.md) - Table definitions and relationships
- [Operations Guide](OPERATIONS.md) - Maintenance and troubleshooting
- [Migration Guide](MIGRATIONS.md) - Schema versioning procedures

## Common Operations

### Health Check
```bash
cargo run --bin db_cli status
```

### Backup
```bash
./scripts/backup.sh
```

### Query Performance
```sql
-- Check index usage
EXPLAIN ANALYZE SELECT * FROM participants WHERE skills ? 'cleaning';
```

## Performance Characteristics
- **Connection Pool:** 10 max connections (configurable)
- **Query Performance:** Sub-millisecond indexed lookups
- **JSONB Operations:** Optimized with GIN indexes
- **Bulk Operations:** Batched transactions for efficiency

## Integration with VividShift

### Repository Manager
The database layer integrates seamlessly with the application through the repository manager:

```rust
pub struct RepositoryManager {
    pub users: UserRepository,
    pub participants: ParticipantRepository,
    pub assignments: AssignmentRepository,
    pub domains: DomainRepository,
}
```

### Connection Pool Configuration
Connection pooling is configured through the application's configuration system:

```toml
[database]
url = "postgresql://postgres:password@localhost:5432/vividshift_dev"
max_connections = 10
min_connections = 1
connect_timeout = 30
```

### Health Monitoring
Database health is monitored through built-in health checks:

- Connection validation
- Query performance tracking
- Pool utilization monitoring
- Automatic reconnection on failure

## Security Features

### Authentication Integration
- User credential storage with bcrypt hashing
- Session management with JWT tokens
- Role-based access control
- Session revocation support

### Data Protection
- Parameterized queries prevent SQL injection
- SSL/TLS connection encryption
- Audit trails for all data modifications
- Soft delete preservation

## Backup and Recovery

### Automated Backup System
- Daily automated backups with compression
- Backup verification and integrity checks
- Configurable retention policies
- Point-in-time recovery capability

### Recovery Procedures
```bash
# Full database restore
./scripts/restore.sh --latest --drop-database

# Validate backup integrity
./scripts/restore.sh backup_file.sql.gz --validate
```

## Performance Optimization

### Indexing Strategy
- B-tree indexes for standard lookups
- GIN indexes for JSONB operations
- Composite indexes for complex queries
- Automatic index maintenance

### Query Optimization
- Connection pooling reduces overhead
- Prepared statements for repeated queries
- Batch operations for bulk data
- Query performance monitoring

## Troubleshooting

### Common Issues
- **Connection timeouts:** Check pool configuration
- **Slow queries:** Review indexing and query patterns
- **Migration failures:** Validate schema changes
- **Pool exhaustion:** Monitor connection usage

### Diagnostic Tools
```bash
# Database status
cargo run --bin db_cli status --detailed

# Migration status
cargo run --bin db_cli migrate --status

# Schema validation
cargo run --bin db_cli validate
```

## References
- [PostgreSQL Documentation](https://www.postgresql.org/docs/)
- [SQLx Documentation](https://docs.rs/sqlx/)
- [Configuration Reference](../CONFIGURATION.md)
- [API Integration](../API_REFERENCE.md)
