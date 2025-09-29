# VividShift Troubleshooting Guide

## Overview
Comprehensive troubleshooting guide for common issues, error resolution, and diagnostic procedures for VividShift Generic Assignment Engine.

**Target Audience:** Developers, system administrators, support engineers

## Quick Diagnostics

### Health Check Commands
```bash
# Application health
curl http://localhost:8080/health
# Expected: {"service":"vividshift-backend","status":"healthy"}

# Database connectivity
cargo run --bin db_cli status

# Full system check
curl http://localhost:8080/ready
# Expected: {"status":"ready","database":"connected","cache":"connected"}
```

### Log Analysis
```bash
# View application logs
docker-compose logs -f app

# Filter for errors
docker-compose logs app | grep -i error

# Database logs
docker-compose logs db

# Real-time monitoring
docker-compose logs -f --tail=100
```

## Common Issues

### Application Startup Problems

#### Service Won't Start
**Symptoms:**
- Container exits immediately
- "Connection refused" errors
- Application logs show startup failures

**Diagnosis:**
```bash
# Check container status
docker-compose ps

# View startup logs
docker-compose logs app

# Check configuration
docker-compose exec app env | grep VIVIDSHIFT_
```

**Solutions:**
1. **Invalid Configuration:**
   ```bash
   # Validate environment variables
   echo $VIVIDSHIFT_DATABASE_URL
   echo $VIVIDSHIFT_AUTH_JWT_SECRET
   
   # Check configuration file syntax
   cargo run --bin config_validator
   ```

2. **Database Connection Issues:**
   ```bash
   # Test database connectivity
   psql $VIVIDSHIFT_DATABASE_URL -c "SELECT 1;"
   
   # Check database is running
   docker-compose ps db
   ```

3. **Port Conflicts:**
   ```bash
   # Check if port is in use
   netstat -tulpn | grep :8080
   
   # Use different port
   export VIVIDSHIFT_SERVER_PORT=8081
   ```

#### Migration Failures
**Symptoms:**
- Application starts but database operations fail
- "relation does not exist" errors
- Migration timeout errors

**Diagnosis:**
```bash
# Check migration status
cargo run --bin db_cli migrate --status

# Validate database schema
cargo run --bin db_cli validate
```

**Solutions:**
```bash
# Run pending migrations
cargo run --bin db_cli migrate

# Reset database (development only)
cargo run --bin db_cli migrate --reset

# Manual migration rollback
cargo run --bin db_cli migrate --rollback
```

### Authentication Issues

#### JWT Token Problems
**Symptoms:**
- "Invalid token" errors
- Authentication failures
- Token expiration issues

**Diagnosis:**
```bash
# Check JWT secret configuration
echo $VIVIDSHIFT_AUTH_JWT_SECRET

# Test token generation
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password123"}'
```

**Solutions:**
1. **Missing JWT Secret:**
   ```bash
   # Generate secure secret
   export VIVIDSHIFT_AUTH_JWT_SECRET=$(openssl rand -hex 32)
   ```

2. **Token Expiration:**
   ```bash
   # Increase token lifetime
   export VIVIDSHIFT_AUTH_JWT_EXPIRATION=86400  # 24 hours
   ```

3. **Invalid Credentials:**
   ```bash
   # Reset user password
   cargo run --bin db_cli reset-password --username=admin
   ```

#### Session Management Issues
**Symptoms:**
- Users logged out unexpectedly
- Session persistence problems
- Multiple login requirements

**Solutions:**
```bash
# Check session configuration
cargo run --bin db_cli status --sessions

# Clean expired sessions
cargo run --bin db_cli cleanup --sessions

# Verify session storage
psql $VIVIDSHIFT_DATABASE_URL -c "SELECT COUNT(*) FROM user_sessions;"
```

### Database Connection Problems

#### Connection Pool Exhaustion
**Symptoms:**
- "Connection pool timeout" errors
- Slow response times
- Database connection failures

**Diagnosis:**
```bash
# Check active connections
psql $VIVIDSHIFT_DATABASE_URL -c "
  SELECT count(*) as active_connections 
  FROM pg_stat_activity 
  WHERE datname = 'vividshift_prod';"

# Monitor connection pool
cargo run --bin db_cli status --connections
```

**Solutions:**
1. **Increase Pool Size:**
   ```toml
   # backend/config/prod.toml
   [database]
   max_connections = 20
   min_connections = 5
   ```

2. **Optimize Queries:**
   ```bash
   # Find slow queries
   psql $VIVIDSHIFT_DATABASE_URL -c "
     SELECT query, mean_time, calls 
     FROM pg_stat_statements 
     ORDER BY mean_time DESC 
     LIMIT 10;"
   ```

3. **Connection Leaks:**
   ```bash
   # Check for long-running queries
   psql $VIVIDSHIFT_DATABASE_URL -c "
     SELECT pid, now() - query_start as duration, query 
     FROM pg_stat_activity 
     WHERE state = 'active' 
     ORDER BY duration DESC;"
   ```

#### Database Performance Issues
**Symptoms:**
- Slow query responses
- High CPU usage
- Memory consumption

**Diagnosis:**
```bash
# Check database performance
psql $VIVIDSHIFT_DATABASE_URL -c "
  SELECT schemaname, tablename, 
         pg_size_pretty(pg_total_relation_size(schemaname||'.'||tablename)) as size
  FROM pg_tables 
  WHERE schemaname = 'public' 
  ORDER BY pg_total_relation_size(schemaname||'.'||tablename) DESC;"

# Analyze query performance
EXPLAIN ANALYZE SELECT * FROM participants WHERE skills ? 'cleaning';
```

**Solutions:**
1. **Index Optimization:**
   ```sql
   -- Check index usage
   SELECT indexrelname, idx_scan, idx_tup_read 
   FROM pg_stat_user_indexes 
   ORDER BY idx_scan DESC;
   
   -- Create missing indexes
   CREATE INDEX CONCURRENTLY idx_participants_name ON participants(name);
   ```

2. **Query Optimization:**
   ```bash
   # Update table statistics
   psql $VIVIDSHIFT_DATABASE_URL -c "ANALYZE participants;"
   
   # Vacuum tables
   psql $VIVIDSHIFT_DATABASE_URL -c "VACUUM ANALYZE;"
   ```

### API and Network Issues

#### HTTP Request Failures
**Symptoms:**
- 500 Internal Server Error
- Connection timeouts
- Slow response times

**Diagnosis:**
```bash
# Test API endpoints
curl -v http://localhost:8080/health

# Check application metrics
curl http://localhost:8080/metrics | grep http_requests

# Monitor response times
time curl http://localhost:8080/api/work-groups/history
```

**Solutions:**
1. **Increase Timeouts:**
   ```toml
   # backend/config/prod.toml
   [server]
   request_timeout = 60
   keep_alive_timeout = 75
   ```

2. **Load Balancer Configuration:**
   ```nginx
   # /etc/nginx/sites-available/vividshift
   location / {
       proxy_connect_timeout 10s;
       proxy_send_timeout 60s;
       proxy_read_timeout 60s;
   }
   ```

#### Rate Limiting Issues
**Symptoms:**
- 429 Too Many Requests errors
- Blocked API access
- Authentication failures

**Solutions:**
```bash
# Check rate limit configuration
grep -r "rate_limit" backend/config/

# Adjust rate limits
export VIVIDSHIFT_RATE_LIMIT_REQUESTS_PER_MINUTE=120

# Whitelist specific IPs (if using nginx)
# Add to nginx config: allow 192.168.1.100;
```

### Assignment Generation Problems

#### Strategy Execution Failures
**Symptoms:**
- Assignment generation timeouts
- "No valid assignments found" errors
- Strategy selection failures

**Diagnosis:**
```bash
# Test assignment generation
curl -X POST http://localhost:8080/api/assignments/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"strategy": "balanced_rotation"}'

# Check available strategies
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/rules/strategies
```

**Solutions:**
1. **Insufficient Data:**
   ```bash
   # Check participants and targets
   curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/entities/participant
   
   curl -H "Authorization: Bearer $TOKEN" \
     http://localhost:8080/api/entities/assignment_target
   ```

2. **Strategy Configuration:**
   ```bash
   # Validate strategy parameters
   cargo run --bin rule_engine_test --strategy=balanced_rotation
   
   # Use fallback strategy
   curl -X POST http://localhost:8080/api/assignments/generate \
     -H "Authorization: Bearer $TOKEN" \
     -d '{"strategy": "random_assignment"}'
   ```

#### Validation Failures
**Symptoms:**
- Assignments rejected by validators
- Capacity constraint violations
- Skill matching failures

**Solutions:**
```bash
# Check validation rules
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/rules/validators

# Disable strict validation (temporarily)
curl -X POST http://localhost:8080/api/assignments/generate \
  -H "Authorization: Bearer $TOKEN" \
  -d '{"strategy": "random_assignment", "validation_mode": "permissive"}'
```

## Performance Troubleshooting

### Memory Issues
**Symptoms:**
- Out of memory errors
- Application crashes
- Slow performance

**Diagnosis:**
```bash
# Check memory usage
docker stats vividshift_app

# Monitor Rust memory allocation
RUST_LOG=debug cargo run 2>&1 | grep -i memory

# Check for memory leaks
valgrind --tool=memcheck cargo run
```

**Solutions:**
```bash
# Increase container memory
# docker-compose.yml
services:
  app:
    mem_limit: 2g
    
# Optimize database connections
export VIVIDSHIFT_DATABASE_MAX_CONNECTIONS=10
```

### CPU Performance
**Symptoms:**
- High CPU usage
- Slow request processing
- Thread contention

**Diagnosis:**
```bash
# Monitor CPU usage
top -p $(pgrep vividshift)

# Check thread usage
ps -eLf | grep vividshift

# Profile application
cargo run --release --bin profiler
```

**Solutions:**
```bash
# Optimize thread pool
export VIVIDSHIFT_WORKER_THREADS=4

# Enable CPU optimizations
cargo build --release --target-cpu=native
```

## Monitoring and Alerting

### Log Analysis Techniques
```bash
# Error pattern analysis
grep -E "(ERROR|FATAL|PANIC)" /var/log/vividshift/app.log

# Performance monitoring
grep "slow_query" /var/log/vividshift/app.log | tail -20

# Authentication monitoring
grep "auth" /var/log/vividshift/app.log | grep -v "success"
```

### Metrics Investigation
```bash
# Check Prometheus metrics
curl http://localhost:9090/api/v1/query?query=http_requests_total

# Database metrics
curl http://localhost:9090/api/v1/query?query=database_connections_active

# System metrics
curl http://localhost:9090/api/v1/query?query=process_resident_memory_bytes
```

## Recovery Procedures

### Application Recovery
```bash
# Graceful restart
docker-compose restart app

# Force restart with cleanup
docker-compose down
docker-compose up -d --force-recreate app

# Rollback to previous version
docker-compose down
git checkout previous-stable-tag
docker-compose up -d --build
```

### Database Recovery
```bash
# Restore from backup
./scripts/restore.sh --latest

# Repair corrupted data
cargo run --bin db_cli repair --table=participants

# Reset to clean state (development only)
cargo run --bin db_cli reset --confirm
```

### Emergency Procedures
```bash
# Enable maintenance mode
curl -X POST http://localhost:8080/admin/maintenance/enable

# Disable problematic features
export VIVIDSHIFT_FEATURE_ASSIGNMENTS_ENABLED=false

# Emergency user creation
cargo run --bin emergency_admin --username=emergency --password=temp123
```

## Preventive Measures

### Health Monitoring Setup
```bash
# Automated health checks
#!/bin/bash
# /etc/cron.d/vividshift-health
*/5 * * * * root /usr/local/bin/vividshift-health-check.sh

# Health check script
#!/bin/bash
if ! curl -f http://localhost:8080/health > /dev/null 2>&1; then
    echo "VividShift health check failed" | mail -s "Alert" admin@company.com
    systemctl restart vividshift
fi
```

### Backup Verification
```bash
# Automated backup testing
#!/bin/bash
# Test backup integrity daily
BACKUP_FILE=$(ls -t /var/backups/vividshift/*.sql.gz | head -1)
gunzip -t "$BACKUP_FILE" || echo "Backup corruption detected!"
```

### Performance Baselines
```bash
# Establish performance baselines
curl -w "@curl-format.txt" -o /dev/null -s http://localhost:8080/health

# Monitor key metrics
echo "response_time:$(curl -w '%{time_total}' -o /dev/null -s http://localhost:8080/health)"
```

## Getting Help

### Debug Information Collection
```bash
# Collect system information
#!/bin/bash
echo "=== VividShift Debug Information ===" > debug-info.txt
echo "Date: $(date)" >> debug-info.txt
echo "Version: $(git describe --tags)" >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Configuration ===" >> debug-info.txt
env | grep VIVIDSHIFT_ >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Service Status ===" >> debug-info.txt
docker-compose ps >> debug-info.txt
echo "" >> debug-info.txt

echo "=== Recent Logs ===" >> debug-info.txt
docker-compose logs --tail=50 app >> debug-info.txt
```

### Support Channels
- **GitHub Issues:** Report bugs with debug information
- **Documentation:** Check relevant guides in `/docs`
- **Community:** Discord/Slack channels for community support
- **Professional Support:** Contact for enterprise support options

## References
- [Configuration Guide](CONFIGURATION.md) - Configuration troubleshooting
- [Database Operations](database/OPERATIONS.md) - Database-specific issues
- [Security Guide](SECURITY.md) - Security-related problems
- [Deployment Guide](DEPLOYMENT.md) - Deployment troubleshooting
- [API Reference](API_REFERENCE.md) - API error codes and solutions
