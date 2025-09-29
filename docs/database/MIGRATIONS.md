# Database Migration Guide

## Overview
Comprehensive guide for managing database schema changes in VividShift using SQLx migrations. This document covers migration creation, execution, rollback procedures, and best practices.

**Target Audience:** Backend developers, database administrators, DevOps engineers

## Migration System Architecture

VividShift uses SQLx's migration system with the following structure:

```
migrations/
├── 20250926152953_initial_schema.sql
├── 20250926152953_initial_schema.down.sql
├── 20250927100000_add_user_profiles.sql
├── 20250927100000_add_user_profiles.down.sql
├── 20250928143000_add_assignment_history.sql
├── 20250928143000_add_assignment_history.down.sql
└── ...
```

### Migration Naming Convention
- **Format:** `YYYYMMDD_HHMMSS_description.sql`
- **Rollback:** `YYYYMMDD_HHMMSS_description.down.sql`
- **Example:** `20250926152953_initial_schema.sql`

## Migration Management

### Creating New Migrations

#### Using SQLx CLI
```bash
# Install sqlx-cli if not already installed
cargo install sqlx-cli --no-default-features --features postgres

# Create a new migration
sqlx migrate add create_new_table

# This creates:
# migrations/TIMESTAMP_create_new_table.sql
# migrations/TIMESTAMP_create_new_table.down.sql (manually created)
```

#### Using VividShift CLI
```bash
# Create migration with VividShift CLI
cargo run --bin db_cli migrate --create "add_new_feature"

# Create migration with template
cargo run --bin db_cli migrate --create "add_new_table" --template=table

# Create data migration
cargo run --bin db_cli migrate --create "seed_initial_data" --template=data
```

### Migration Templates

#### Table Creation Template
```sql
-- migrations/TIMESTAMP_create_example_table.sql
CREATE TABLE example_table (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    name VARCHAR(200) NOT NULL,
    description TEXT,
    metadata JSONB DEFAULT '{}',
    is_active BOOLEAN NOT NULL DEFAULT true,
    created_at TIMESTAMPTZ NOT NULL DEFAULT NOW(),
    updated_at TIMESTAMPTZ NOT NULL DEFAULT NOW()
);

-- Indexes
CREATE INDEX idx_example_table_name ON example_table(name);
CREATE INDEX idx_example_table_is_active ON example_table(is_active);
CREATE INDEX idx_example_table_metadata_gin ON example_table USING gin(metadata);

-- Triggers
CREATE TRIGGER update_example_table_updated_at 
    BEFORE UPDATE ON example_table 
    FOR EACH ROW EXECUTE FUNCTION update_updated_at_column();

-- Comments
COMMENT ON TABLE example_table IS 'Example table for demonstration';
COMMENT ON COLUMN example_table.metadata IS 'Flexible metadata storage';
```

#### Rollback Template
```sql
-- migrations/TIMESTAMP_create_example_table.down.sql
-- Drop triggers first
DROP TRIGGER IF EXISTS update_example_table_updated_at ON example_table;

-- Drop indexes
DROP INDEX IF EXISTS idx_example_table_metadata_gin;
DROP INDEX IF EXISTS idx_example_table_is_active;
DROP INDEX IF EXISTS idx_example_table_name;

-- Drop table
DROP TABLE IF EXISTS example_table;
```

### Running Migrations

#### Standard Migration Execution
```bash
# Check current migration status
sqlx migrate info

# Run all pending migrations
sqlx migrate run

# Using VividShift CLI
cargo run --bin db_cli migrate

# Check migration status
cargo run --bin db_cli migrate --status
```

#### Environment-Specific Migrations
```bash
# Development environment
ENVIRONMENT=dev cargo run --bin db_cli migrate

# Staging environment
ENVIRONMENT=staging cargo run --bin db_cli migrate

# Production environment
ENVIRONMENT=prod cargo run --bin db_cli migrate
```

#### Dry Run and Validation
```bash
# Show what migrations would be executed
cargo run --bin db_cli migrate --dry-run

# Validate migrations before execution
cargo run --bin db_cli migrate --validate

# Run with detailed logging
RUST_LOG=debug cargo run --bin db_cli migrate
```

### Rollback Procedures

#### Single Migration Rollback
```bash
# Rollback the last migration
sqlx migrate revert

# Using VividShift CLI
cargo run --bin db_cli migrate --rollback

# Rollback to specific version
cargo run --bin db_cli migrate --rollback --target=20250926152953
```

#### Rollback Planning
```bash
# Show rollback plan
cargo run --bin db_cli migrate --rollback --dry-run

# Validate rollback scripts
cargo run --bin db_cli migrate --rollback --validate

# Force rollback (use with caution)
cargo run --bin db_cli migrate --rollback --force
```

## Migration Best Practices

### Schema Changes

#### Adding Columns
```sql
-- Safe: Add nullable column
ALTER TABLE participants ADD COLUMN phone VARCHAR(20);

-- Safe: Add column with default
ALTER TABLE participants ADD COLUMN status VARCHAR(20) NOT NULL DEFAULT 'active';

-- Risky: Add non-nullable column without default (requires data migration)
-- Better approach:
-- 1. Add nullable column
-- 2. Populate data
-- 3. Add NOT NULL constraint in separate migration
```

#### Modifying Columns
```sql
-- Safe: Increase column size
ALTER TABLE participants ALTER COLUMN name TYPE VARCHAR(300);

-- Risky: Decrease column size (data loss possible)
-- Better approach: validate data first
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM participants WHERE length(name) > 200) THEN
        RAISE EXCEPTION 'Cannot reduce column size: data would be truncated';
    END IF;
END $$;
ALTER TABLE participants ALTER COLUMN name TYPE VARCHAR(200);

-- Safe: Make column nullable
ALTER TABLE participants ALTER COLUMN phone DROP NOT NULL;

-- Risky: Make column non-nullable
-- Better approach: ensure no NULL values first
UPDATE participants SET phone = '' WHERE phone IS NULL;
ALTER TABLE participants ALTER COLUMN phone SET NOT NULL;
```

#### Index Management
```sql
-- Safe: Create index concurrently (doesn't block writes)
CREATE INDEX CONCURRENTLY idx_participants_email ON participants(email);

-- Rollback for concurrent index
DROP INDEX CONCURRENTLY IF EXISTS idx_participants_email;

-- Complex index with conditions
CREATE INDEX CONCURRENTLY idx_participants_active_skills 
ON participants USING gin(skills) 
WHERE is_active = true;
```

### Data Migrations

#### Safe Data Migration Pattern
```sql
-- Migration: Update data in batches
DO $$
DECLARE
    batch_size INTEGER := 1000;
    processed INTEGER := 0;
    total INTEGER;
BEGIN
    SELECT COUNT(*) INTO total FROM participants WHERE old_field IS NOT NULL;
    
    WHILE processed < total LOOP
        UPDATE participants 
        SET new_field = transform_old_field(old_field)
        WHERE id IN (
            SELECT id FROM participants 
            WHERE old_field IS NOT NULL 
              AND new_field IS NULL
            LIMIT batch_size
        );
        
        processed := processed + batch_size;
        
        -- Log progress
        RAISE NOTICE 'Processed % of % records', processed, total;
        
        -- Allow other operations
        COMMIT;
    END LOOP;
END $$;
```

#### Data Validation
```sql
-- Validate data consistency after migration
DO $$
DECLARE
    inconsistent_count INTEGER;
BEGIN
    SELECT COUNT(*) INTO inconsistent_count
    FROM participants 
    WHERE old_field IS NOT NULL 
      AND new_field IS NULL;
    
    IF inconsistent_count > 0 THEN
        RAISE EXCEPTION 'Data migration incomplete: % records not migrated', inconsistent_count;
    END IF;
    
    RAISE NOTICE 'Data migration validation passed';
END $$;
```

### Production Migration Strategy

#### Pre-Migration Checklist
1. **Backup Database**
   ```bash
   ./scripts/backup.sh --verify
   ```

2. **Test Migration on Staging**
   ```bash
   # Copy production data to staging
   ./scripts/restore.sh production_backup.sql.gz --target=staging
   
   # Run migration on staging
   ENVIRONMENT=staging cargo run --bin db_cli migrate
   ```

3. **Validate Migration Scripts**
   ```bash
   cargo run --bin db_cli migrate --validate --dry-run
   ```

4. **Plan Rollback Strategy**
   ```bash
   cargo run --bin db_cli migrate --rollback --dry-run
   ```

#### Migration Execution Process
```bash
# 1. Enable maintenance mode (if applicable)
curl -X POST http://localhost:8080/admin/maintenance/enable

# 2. Stop application services
docker-compose stop app

# 3. Create pre-migration backup
./scripts/backup.sh --tag="pre-migration-$(date +%Y%m%d_%H%M%S)"

# 4. Run migrations
ENVIRONMENT=prod cargo run --bin db_cli migrate

# 5. Validate migration success
cargo run --bin db_cli validate --full-report

# 6. Start application services
docker-compose start app

# 7. Disable maintenance mode
curl -X POST http://localhost:8080/admin/maintenance/disable

# 8. Monitor application health
curl http://localhost:8080/health
```

## Advanced Migration Scenarios

### Schema Refactoring

#### Table Renaming
```sql
-- Step 1: Create new table with desired structure
CREATE TABLE new_participants (
    id UUID PRIMARY KEY DEFAULT uuid_generate_v4(),
    full_name VARCHAR(200) NOT NULL,  -- renamed from 'name'
    email_address VARCHAR(255),       -- renamed from 'email'
    -- ... other columns
);

-- Step 2: Migrate data
INSERT INTO new_participants (id, full_name, email_address, ...)
SELECT id, name, email, ... FROM participants;

-- Step 3: Create views for backward compatibility (if needed)
CREATE VIEW participants AS SELECT 
    id,
    full_name as name,
    email_address as email,
    ...
FROM new_participants;

-- Step 4: In subsequent migration, drop old table and rename new table
-- (This allows for zero-downtime deployment)
```

#### Column Type Changes
```sql
-- Safe approach for changing column types
-- Step 1: Add new column
ALTER TABLE participants ADD COLUMN phone_new VARCHAR(20);

-- Step 2: Migrate data with validation
UPDATE participants 
SET phone_new = CASE 
    WHEN phone ~ '^[0-9+\-\s()]+$' THEN phone
    ELSE NULL
END;

-- Step 3: Validate migration
DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM participants WHERE phone IS NOT NULL AND phone_new IS NULL) THEN
        RAISE EXCEPTION 'Phone number migration failed for some records';
    END IF;
END $$;

-- Step 4: Drop old column and rename new column (in separate migration)
ALTER TABLE participants DROP COLUMN phone;
ALTER TABLE participants RENAME COLUMN phone_new TO phone;
```

### Complex Relationship Changes

#### Foreign Key Modifications
```sql
-- Adding foreign key constraint safely
-- Step 1: Add column without constraint
ALTER TABLE assignments ADD COLUMN created_by_new UUID;

-- Step 2: Populate with valid references
UPDATE assignments 
SET created_by_new = (
    SELECT id FROM users 
    WHERE username = 'system' 
    LIMIT 1
) 
WHERE created_by IS NULL;

UPDATE assignments 
SET created_by_new = created_by 
WHERE created_by IS NOT NULL 
  AND EXISTS (SELECT 1 FROM users WHERE id = assignments.created_by);

-- Step 3: Add foreign key constraint
ALTER TABLE assignments 
ADD CONSTRAINT fk_assignments_created_by_new 
FOREIGN KEY (created_by_new) REFERENCES users(id);

-- Step 4: Drop old column and rename (in separate migration)
ALTER TABLE assignments DROP COLUMN created_by;
ALTER TABLE assignments RENAME COLUMN created_by_new TO created_by;
```

## Migration Testing

### Automated Testing
```rust
// tests/migrations_test.rs
#[tokio::test]
async fn test_migration_up_and_down() {
    let pool = setup_test_database().await;
    
    // Run all migrations
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // Verify schema
    let table_count: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM information_schema.tables WHERE table_schema = 'public'"
    )
    .fetch_one(&pool)
    .await
    .unwrap();
    
    assert!(table_count > 0);
    
    // Test rollback (if supported)
    // Note: SQLx doesn't support automatic rollback
    // This would need custom implementation
}

#[tokio::test]
async fn test_data_migration_integrity() {
    let pool = setup_test_database_with_data().await;
    
    // Count records before migration
    let count_before: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM participants")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    // Run migration
    sqlx::migrate!("./migrations").run(&pool).await.unwrap();
    
    // Count records after migration
    let count_after: i64 = sqlx::query_scalar("SELECT COUNT(*) FROM participants")
        .fetch_one(&pool)
        .await
        .unwrap();
    
    assert_eq!(count_before, count_after);
}
```

### Manual Testing Procedures
```bash
# 1. Create test database
createdb vividshift_migration_test

# 2. Run migrations
DATABASE_URL=postgresql://localhost/vividshift_migration_test sqlx migrate run

# 3. Seed test data
cargo run --bin db_cli seed --database-url=postgresql://localhost/vividshift_migration_test

# 4. Test application functionality
VIVIDSHIFT_DATABASE_URL=postgresql://localhost/vividshift_migration_test cargo test

# 5. Clean up
dropdb vividshift_migration_test
```

## Troubleshooting Migrations

### Common Issues

#### Migration Failures
```bash
# Check migration status
cargo run --bin db_cli migrate --status

# View failed migration details
sqlx migrate info

# Fix and retry migration
# Edit migration file to fix issues
cargo run --bin db_cli migrate
```

#### Rollback Issues
```bash
# Validate rollback script
cargo run --bin db_cli migrate --rollback --validate

# Manual rollback if automated rollback fails
psql $DATABASE_URL -f migrations/TIMESTAMP_migration.down.sql

# Update migration tracking
DELETE FROM _sqlx_migrations WHERE version = 'TIMESTAMP';
```

#### Data Consistency Issues
```sql
-- Check for orphaned records
SELECT 'assignment_details' as table_name, COUNT(*) as orphaned_count
FROM assignment_details ad
LEFT JOIN assignments a ON ad.assignment_id = a.id
WHERE a.id IS NULL

UNION ALL

SELECT 'assignment_details' as table_name, COUNT(*) as orphaned_count
FROM assignment_details ad
LEFT JOIN participants p ON ad.participant_id = p.id
WHERE p.id IS NULL;

-- Fix orphaned records
DELETE FROM assignment_details 
WHERE assignment_id NOT IN (SELECT id FROM assignments);
```

### Recovery Procedures

#### Partial Migration Failure
```bash
# 1. Identify failed migration
cargo run --bin db_cli migrate --status

# 2. Manual cleanup of partial changes
psql $DATABASE_URL -c "
    -- Rollback partial changes
    DROP TABLE IF EXISTS partially_created_table;
    DROP INDEX IF EXISTS partially_created_index;
"

# 3. Update migration tracking
psql $DATABASE_URL -c "
    DELETE FROM _sqlx_migrations 
    WHERE version = 'failed_migration_timestamp';
"

# 4. Fix migration script and retry
cargo run --bin db_cli migrate
```

#### Complete Database Recovery
```bash
# 1. Stop application
docker-compose stop app

# 2. Restore from backup
./scripts/restore.sh --latest --drop-database

# 3. Re-run migrations
cargo run --bin db_cli migrate

# 4. Validate database state
cargo run --bin db_cli validate --full-report

# 5. Restart application
docker-compose start app
```

## Migration Monitoring

### Performance Monitoring
```sql
-- Monitor long-running migrations
SELECT 
    pid,
    now() - pg_stat_activity.query_start AS duration,
    query 
FROM pg_stat_activity 
WHERE (now() - pg_stat_activity.query_start) > interval '5 minutes';

-- Check lock conflicts during migration
SELECT 
    blocked_locks.pid AS blocked_pid,
    blocked_activity.usename AS blocked_user,
    blocking_locks.pid AS blocking_pid,
    blocking_activity.usename AS blocking_user,
    blocked_activity.query AS blocked_statement
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
WHERE NOT blocked_locks.granted;
```

### Migration Logging
```bash
# Enable detailed migration logging
RUST_LOG=sqlx=debug,db_cli=debug cargo run --bin db_cli migrate

# Log migration execution time
time cargo run --bin db_cli migrate

# Monitor database during migration
watch -n 1 'psql $DATABASE_URL -c "SELECT count(*) FROM pg_stat_activity;"'
```

## References
- [Database Schema](SCHEMA.md) - Table structures and relationships
- [Database Operations](OPERATIONS.md) - Maintenance and monitoring
- [SQLx Documentation](https://docs.rs/sqlx/) - SQLx migration system
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) - PostgreSQL DDL reference
