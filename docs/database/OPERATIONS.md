# Database Operations Guide

## Overview
Comprehensive guide for database maintenance, monitoring, and troubleshooting operations for VividShift's PostgreSQL database.

**Target Audience:** Database administrators, DevOps engineers, system operators

## Daily Operations

### Health Monitoring

#### Database Status Check
```bash
# Using VividShift CLI
cargo run --bin db_cli status

# Detailed status with metrics
cargo run --bin db_cli status --detailed

# Check specific components
cargo run --bin db_cli status --component=connections
cargo run --bin db_cli status --component=performance
```

#### Connection Pool Monitoring
```sql
-- Current connection status
SELECT 
    pid,
    usename,
    application_name,
    client_addr,
    state,
    query_start,
    query
FROM pg_stat_activity 
WHERE datname = 'vividshift_prod';

-- Connection pool statistics
SELECT 
    numbackends as active_connections,
    xact_commit as transactions_committed,
    xact_rollback as transactions_rolled_back,
    blks_read as blocks_read,
    blks_hit as blocks_hit_cache
FROM pg_stat_database 
WHERE datname = 'vividshift_prod';
```

#### Performance Metrics
```sql
-- Query performance overview
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    rows
FROM pg_stat_statements 
ORDER BY mean_time DESC 
LIMIT 10;

-- Table size and usage
SELECT 
    schemaname,
    tablename,
    pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size,
    n_tup_ins as inserts,
    n_tup_upd as updates,
    n_tup_del as deletes
FROM pg_stat_user_tables 
ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;
```

### Backup Operations

#### Automated Daily Backup
```bash
# Run automated backup
./scripts/backup.sh

# Backup with custom settings
./scripts/backup.sh -d postgresql://user:pass@host/db --compress --verify

# Schema-only backup
./scripts/backup.sh --schema-only

# Data-only backup
./scripts/backup.sh --data-only
```

#### Backup Verification
```bash
# List available backups
./scripts/restore.sh --list

# Verify backup integrity
./scripts/restore.sh --verify backup_file.sql.gz

# Test restore (dry run)
./scripts/restore.sh --dry-run backup_file.sql.gz
```

#### Backup Rotation
```bash
# Configure backup retention (in backup script)
RETENTION_DAYS=30
BACKUP_DIR="/var/backups/vividshift"

# Manual cleanup of old backups
find $BACKUP_DIR -name "*.sql.gz" -mtime +$RETENTION_DAYS -delete
find $BACKUP_DIR -name "*.meta" -mtime +$RETENTION_DAYS -delete
```

### Maintenance Tasks

#### Database Cleanup
```bash
# Clean expired sessions
cargo run --bin db_cli cleanup --sessions

# Clean old assignment history (older than 1 year)
cargo run --bin db_cli cleanup --history --older-than=365d

# Vacuum and analyze tables
cargo run --bin db_cli maintenance --vacuum --analyze
```

#### Index Maintenance
```sql
-- Rebuild fragmented indexes
REINDEX INDEX CONCURRENTLY idx_participants_skills_gin;

-- Update table statistics
ANALYZE participants;
ANALYZE assignments;
ANALYZE assignment_details;

-- Check for unused indexes
SELECT 
    schemaname,
    tablename,
    indexname,
    idx_scan
FROM pg_stat_user_indexes 
WHERE idx_scan = 0;
```

## Migration Management

### Running Migrations

#### Standard Migration Process
```bash
# Check migration status
cargo run --bin db_cli migrate --status

# Run pending migrations
cargo run --bin db_cli migrate

# Run specific migration
cargo run --bin db_cli migrate --target=20240101_120000

# Dry run (show what would be executed)
cargo run --bin db_cli migrate --dry-run
```

#### Rollback Operations
```bash
# Rollback last migration
cargo run --bin db_cli migrate --rollback

# Rollback to specific version
cargo run --bin db_cli migrate --rollback --target=20240101_120000

# Show rollback plan
cargo run --bin db_cli migrate --rollback --dry-run
```

### Migration Validation
```bash
# Validate schema integrity
cargo run --bin db_cli validate

# Check foreign key constraints
cargo run --bin db_cli validate --constraints

# Verify index consistency
cargo run --bin db_cli validate --indexes

# Full validation report
cargo run --bin db_cli validate --full-report
```

## Data Seeding

### Development Data Seeding
```bash
# Seed with default development data
cargo run --bin db_cli seed

# Force re-seeding (clears existing data)
cargo run --bin db_cli seed --force

# Seed specific data sets
cargo run --bin db_cli seed --users --participants
cargo run --bin db_cli seed --assignments --targets

# Custom seed file
cargo run --bin db_cli seed --file=custom_seed.json
```

### Production Data Import
```bash
# Import from JSON file
cargo run --bin db_cli import --file=production_data.json --validate

# Import with transformation
cargo run --bin db_cli import --file=legacy_data.csv --transform=legacy_to_vividshift

# Bulk import with batching
cargo run --bin db_cli import --file=large_dataset.json --batch-size=1000
```

## Performance Tuning

### Query Optimization

#### Slow Query Analysis
```sql
-- Enable slow query logging
ALTER SYSTEM SET log_min_duration_statement = 1000; -- 1 second
SELECT pg_reload_conf();

-- Analyze slow queries
SELECT 
    query,
    calls,
    total_time,
    mean_time,
    (total_time/calls) as avg_time_ms
FROM pg_stat_statements 
WHERE mean_time > 100
ORDER BY total_time DESC;
```

#### Index Usage Optimization
```sql
-- Find missing indexes (high seq_scan, low idx_scan)
SELECT 
    schemaname,
    tablename,
    seq_scan,
    seq_tup_read,
    idx_scan,
    idx_tup_fetch,
    seq_tup_read / seq_scan as avg_seq_read
FROM pg_stat_user_tables 
WHERE seq_scan > 0
ORDER BY seq_tup_read DESC;

-- Analyze query plans for optimization
EXPLAIN (ANALYZE, BUFFERS) 
SELECT * FROM participants 
WHERE skills ? 'cleaning' 
  AND is_active = true;
```

### Connection Pool Tuning
```bash
# Monitor connection pool metrics
cargo run --bin db_cli status --connections

# Adjust pool settings in configuration
# backend/config/prod.toml
[database]
max_connections = 50      # Increase for high load
min_connections = 10      # Maintain minimum connections
connect_timeout = 10      # Reduce timeout for faster failover
idle_timeout = 600        # Close idle connections
max_lifetime = 1800       # Rotate connections regularly
```

### Memory and Storage Optimization
```sql
-- Check database size and growth
SELECT 
    pg_size_pretty(pg_database_size('vividshift_prod')) as db_size,
    pg_size_pretty(pg_total_relation_size('participants')) as participants_size,
    pg_size_pretty(pg_total_relation_size('assignments')) as assignments_size;

-- Analyze table bloat
SELECT 
    tablename,
    pg_size_pretty(pg_total_relation_size(tablename)) as size,
    n_dead_tup,
    n_live_tup,
    round(n_dead_tup * 100.0 / (n_live_tup + n_dead_tup), 2) as bloat_percent
FROM pg_stat_user_tables 
WHERE n_live_tup > 0
ORDER BY n_dead_tup DESC;
```

## Disaster Recovery

### Backup and Restore Procedures

#### Full Database Restore
```bash
# Stop application services
docker-compose stop app

# Restore from latest backup
./scripts/restore.sh --latest --drop-database

# Restore from specific backup
./scripts/restore.sh backup_20240101_120000.sql.gz --drop-database

# Verify restore
cargo run --bin db_cli validate --full-report

# Restart services
docker-compose start app
```

#### Point-in-Time Recovery
```bash
# Restore to specific timestamp
./scripts/restore.sh backup_file.sql.gz --point-in-time="2024-01-01 12:00:00"

# Restore with transaction log replay
./scripts/restore.sh backup_file.sql.gz --replay-logs --target-time="2024-01-01 12:00:00"
```

#### Partial Data Recovery
```bash
# Restore specific tables
./scripts/restore.sh backup_file.sql.gz --tables="participants,assignments"

# Restore with data filtering
./scripts/restore.sh backup_file.sql.gz --where="created_at >= '2024-01-01'"
```

### High Availability Setup

#### Master-Slave Replication
```sql
-- On master server
ALTER SYSTEM SET wal_level = replica;
ALTER SYSTEM SET max_wal_senders = 3;
ALTER SYSTEM SET wal_keep_segments = 64;
SELECT pg_reload_conf();

-- Create replication user
CREATE USER replicator REPLICATION LOGIN ENCRYPTED PASSWORD 'password';
```

#### Failover Procedures
```bash
# Check replication status
SELECT 
    client_addr,
    state,
    sent_lsn,
    write_lsn,
    flush_lsn,
    replay_lsn
FROM pg_stat_replication;

# Promote slave to master (in case of failover)
pg_promote /var/lib/postgresql/data

# Update application configuration to new master
export VIVIDSHIFT_DATABASE_URL="postgresql://user:pass@new-master:5432/vividshift"
```

## Troubleshooting

### Common Issues

#### Connection Problems
```bash
# Check if PostgreSQL is running
systemctl status postgresql
docker-compose ps db

# Test connection
psql $VIVIDSHIFT_DATABASE_URL -c "SELECT 1;"

# Check connection limits
SELECT 
    max_conn,
    used,
    res_for_super,
    max_conn - used - res_for_super as available
FROM (
    SELECT count(*) used FROM pg_stat_activity
) t1,
(
    SELECT setting::int res_for_super FROM pg_settings WHERE name = 'superuser_reserved_connections'
) t2,
(
    SELECT setting::int max_conn FROM pg_settings WHERE name = 'max_connections'
) t3;
```

#### Performance Issues
```bash
# Check for blocking queries
SELECT 
    blocked_locks.pid AS blocked_pid,
    blocked_activity.usename AS blocked_user,
    blocking_locks.pid AS blocking_pid,
    blocking_activity.usename AS blocking_user,
    blocked_activity.query AS blocked_statement,
    blocking_activity.query AS blocking_statement
FROM pg_catalog.pg_locks blocked_locks
JOIN pg_catalog.pg_stat_activity blocked_activity ON blocked_activity.pid = blocked_locks.pid
JOIN pg_catalog.pg_locks blocking_locks ON blocking_locks.locktype = blocked_locks.locktype
JOIN pg_catalog.pg_stat_activity blocking_activity ON blocking_activity.pid = blocking_locks.pid
WHERE NOT blocked_locks.granted;

# Kill blocking query (if necessary)
SELECT pg_terminate_backend(pid) FROM pg_stat_activity WHERE pid = <blocking_pid>;
```

#### Data Integrity Issues
```bash
# Check for constraint violations
cargo run --bin db_cli validate --constraints

# Verify foreign key integrity
SELECT 
    conname,
    conrelid::regclass,
    confrelid::regclass
FROM pg_constraint 
WHERE contype = 'f'
  AND NOT EXISTS (
    SELECT 1 FROM information_schema.table_constraints 
    WHERE constraint_name = conname
  );

# Fix orphaned records
DELETE FROM assignment_details 
WHERE participant_id NOT IN (SELECT id FROM participants);
```

### Monitoring and Alerting

#### Key Metrics to Monitor
```sql
-- Database size growth
SELECT 
    pg_size_pretty(pg_database_size(current_database())) as current_size,
    pg_size_pretty(pg_database_size(current_database()) - lag(pg_database_size(current_database())) OVER (ORDER BY now())) as growth;

-- Connection pool utilization
SELECT 
    count(*) as active_connections,
    count(*) * 100.0 / (SELECT setting::int FROM pg_settings WHERE name = 'max_connections') as utilization_percent
FROM pg_stat_activity 
WHERE state = 'active';

-- Query performance degradation
SELECT 
    query,
    calls,
    mean_time,
    lag(mean_time) OVER (PARTITION BY query ORDER BY now()) as previous_mean_time
FROM pg_stat_statements 
WHERE calls > 100;
```

#### Alerting Thresholds
- **Connection utilization** > 80%
- **Query response time** > 1 second average
- **Database size growth** > 10% per day
- **Failed connections** > 1% of total
- **Replication lag** > 1 minute

### Log Analysis
```bash
# PostgreSQL log analysis
tail -f /var/log/postgresql/postgresql-*.log | grep ERROR

# Application database logs
docker-compose logs -f app | grep "database\|sql\|connection"

# Slow query log analysis
grep "duration:" /var/log/postgresql/postgresql-*.log | sort -k12 -nr | head -10
```

## Security Operations

### Access Control Management
```sql
-- Review user permissions
SELECT 
    r.rolname,
    r.rolsuper,
    r.rolinherit,
    r.rolcreaterole,
    r.rolcreatedb,
    r.rolcanlogin,
    r.rolconnlimit,
    r.rolvaliduntil
FROM pg_roles r
WHERE r.rolcanlogin = true;

-- Audit database access
SELECT 
    usename,
    datname,
    client_addr,
    application_name,
    state,
    query_start
FROM pg_stat_activity 
WHERE datname = 'vividshift_prod';
```

### Security Hardening
```bash
# Enable SSL connections
ALTER SYSTEM SET ssl = on;
ALTER SYSTEM SET ssl_cert_file = 'server.crt';
ALTER SYSTEM SET ssl_key_file = 'server.key';

# Configure authentication
# Edit pg_hba.conf
hostssl all all 0.0.0.0/0 md5

# Reload configuration
SELECT pg_reload_conf();
```

## References
- [Database Schema](SCHEMA.md) - Table structures and relationships
- [Migration Guide](MIGRATIONS.md) - Schema versioning procedures
- [Configuration Reference](../CONFIGURATION.md) - Database configuration options
- [PostgreSQL Documentation](https://www.postgresql.org/docs/) - Official PostgreSQL docs
