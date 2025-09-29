# VividShift Deployment Guide

## Overview
Comprehensive deployment guide for VividShift Generic Assignment Engine covering development, staging, and production environments.

**Target Audience:** DevOps engineers, system administrators, deployment engineers

## Prerequisites

### System Requirements
- **CPU:** 2 cores minimum, 4 cores recommended
- **Memory:** 4GB RAM minimum, 8GB recommended for production
- **Storage:** 20GB minimum, SSD recommended
- **Network:** Stable internet connection for dependencies

### Required Software
- Docker 20.10+ and Docker Compose 2.0+
- Git for source code management
- curl for API testing
- jq for JSON processing (optional but recommended)

### Optional Tools
- nginx or similar reverse proxy for production
- SSL certificates for HTTPS
- Monitoring tools (Prometheus, Grafana)

## Quick Start Deployment

### 1. Repository Setup
```bash
# Clone repository
git clone https://github.com/onelrian/vividshift.git
cd vividshift

# Switch to main branch (or desired release)
git checkout main

# Copy environment template
cp .env.example .env
```

### 2. Environment Configuration
Edit `.env` file with your specific values:

```bash
# Core configuration
ENVIRONMENT=prod
VIVIDSHIFT_SERVER_HOST=0.0.0.0
VIVIDSHIFT_SERVER_PORT=8080

# Database (update with your credentials)
VIVIDSHIFT_DATABASE_URL=postgresql://postgres:secure_password@localhost:5432/vividshift_prod

# Security (generate secure values)
VIVIDSHIFT_AUTH_JWT_SECRET=your-secure-jwt-secret-here

# Logging
VIVIDSHIFT_LOGGING_LEVEL=info
VIVIDSHIFT_LOGGING_JSON_FORMAT=true
```

### 3. Service Deployment
```bash
# Start all services
docker-compose up -d --build

# Verify services are running
docker-compose ps

# Check logs for any issues
docker-compose logs -f
```

### 4. Database Initialization
```bash
# Run database migrations
docker-compose exec app cargo run --bin db_cli migrate

# Seed initial data (optional for production)
docker-compose exec app cargo run --bin db_cli seed
```

### 5. Deployment Verification
```bash
# Health check
curl http://localhost:8080/health
# Expected: {"service":"vividshift-backend","status":"healthy"}

# Readiness check
curl http://localhost:8080/ready
# Expected: {"status":"ready","database":"connected"}

# API functionality test
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","email":"admin@example.com","password":"password123"}'
```

## Environment-Specific Deployments

### Development Environment

#### Configuration
```bash
# .env for development
ENVIRONMENT=dev
VIVIDSHIFT_SERVER_HOST=127.0.0.1
VIVIDSHIFT_DATABASE_URL=postgresql://postgres:password@localhost:5432/vividshift_dev
VIVIDSHIFT_LOGGING_LEVEL=debug
VIVIDSHIFT_LOGGING_FILE_ENABLED=true
```

#### Development Setup
```bash
# Start development services
docker-compose -f docker-compose.dev.yml up -d

# Run application locally (optional)
cd backend
cargo run

# Hot reload for development
cargo watch -x run
```

### Staging Environment

#### Configuration
```bash
# .env for staging
ENVIRONMENT=staging
VIVIDSHIFT_SERVER_HOST=0.0.0.0
VIVIDSHIFT_DATABASE_URL=postgresql://postgres:staging_password@staging-db:5432/vividshift_staging
VIVIDSHIFT_LOGGING_LEVEL=info
VIVIDSHIFT_LOGGING_JSON_FORMAT=true
```

#### Staging Deployment
```bash
# Deploy to staging
docker-compose -f docker-compose.staging.yml up -d --build

# Run integration tests
docker-compose exec app cargo test --release

# Performance testing
docker-compose exec app cargo run --bin load_test
```

### Production Environment

#### Security Configuration
```bash
# Generate secure JWT secret
VIVIDSHIFT_AUTH_JWT_SECRET=$(openssl rand -hex 32)

# Use strong database password
VIVIDSHIFT_DATABASE_URL=postgresql://vividshift_user:$(openssl rand -base64 32)@prod-db:5432/vividshift_prod

# Enable security features
VIVIDSHIFT_AUTH_BCRYPT_COST=14
VIVIDSHIFT_LOGGING_LEVEL=warn
```

#### Production Deployment Process
```bash
# 1. Backup existing data (if upgrading)
./scripts/backup.sh --tag="pre-deployment-$(date +%Y%m%d_%H%M%S)"

# 2. Deploy new version
docker-compose -f docker-compose.prod.yml pull
docker-compose -f docker-compose.prod.yml up -d --no-deps app

# 3. Run database migrations
docker-compose exec app cargo run --bin db_cli migrate

# 4. Verify deployment
curl -f http://localhost:8080/health || exit 1

# 5. Run smoke tests
./scripts/smoke_tests.sh
```

## Container Configuration

### Docker Compose Services

#### Application Service
```yaml
# docker-compose.yml
services:
  app:
    build:
      context: ./backend
      dockerfile: Dockerfile
    ports:
      - "8080:8080"
    environment:
      - ENVIRONMENT=${ENVIRONMENT}
      - VIVIDSHIFT_DATABASE_URL=${VIVIDSHIFT_DATABASE_URL}
      - VIVIDSHIFT_AUTH_JWT_SECRET=${VIVIDSHIFT_AUTH_JWT_SECRET}
    depends_on:
      - db
      - redis
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "curl", "-f", "http://localhost:8080/health"]
      interval: 30s
      timeout: 10s
      retries: 3
```

#### Database Service
```yaml
  db:
    image: postgres:15
    environment:
      - POSTGRES_DB=vividshift_prod
      - POSTGRES_USER=postgres
      - POSTGRES_PASSWORD=${DB_PASSWORD}
    volumes:
      - postgres_data:/var/lib/postgresql/data
      - ./scripts/init-db.sql:/docker-entrypoint-initdb.d/init.sql
    ports:
      - "5432:5432"
    restart: unless-stopped
    healthcheck:
      test: ["CMD-SHELL", "pg_isready -U postgres"]
      interval: 10s
      timeout: 5s
      retries: 5
```

#### Redis Service
```yaml
  redis:
    image: redis:7-alpine
    ports:
      - "6379:6379"
    volumes:
      - redis_data:/data
    restart: unless-stopped
    healthcheck:
      test: ["CMD", "redis-cli", "ping"]
      interval: 10s
      timeout: 5s
      retries: 3
```

### Production Docker Configuration

#### Multi-stage Dockerfile
```dockerfile
# backend/Dockerfile.prod
FROM rust:1.70 as builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src

# Build optimized binary
RUN cargo build --release

FROM debian:bookworm-slim

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    libpq5 \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false vividshift

WORKDIR /app
COPY --from=builder /app/target/release/vividshift ./
COPY --from=builder /app/config ./config

# Set ownership and permissions
RUN chown -R vividshift:vividshift /app
USER vividshift

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
  CMD curl -f http://localhost:8080/health || exit 1

CMD ["./vividshift"]
```

## Load Balancer Configuration

### Nginx Configuration
```nginx
# /etc/nginx/sites-available/vividshift
upstream vividshift_backend {
    least_conn;
    server app1:8080 max_fails=3 fail_timeout=30s;
    server app2:8080 max_fails=3 fail_timeout=30s;
    server app3:8080 max_fails=3 fail_timeout=30s;
}

server {
    listen 80;
    listen [::]:80;
    server_name your-domain.com;
    
    # Redirect HTTP to HTTPS
    return 301 https://$server_name$request_uri;
}

server {
    listen 443 ssl http2;
    listen [::]:443 ssl http2;
    server_name your-domain.com;
    
    # SSL configuration
    ssl_certificate /path/to/cert.pem;
    ssl_certificate_key /path/to/key.pem;
    ssl_protocols TLSv1.2 TLSv1.3;
    ssl_ciphers ECDHE-RSA-AES256-GCM-SHA512:DHE-RSA-AES256-GCM-SHA512;
    ssl_prefer_server_ciphers off;
    
    # Security headers
    add_header X-Frame-Options DENY;
    add_header X-Content-Type-Options nosniff;
    add_header X-XSS-Protection "1; mode=block";
    add_header Strict-Transport-Security "max-age=63072000; includeSubDomains; preload";
    
    # Rate limiting
    limit_req_zone $binary_remote_addr zone=api:10m rate=10r/s;
    limit_req_zone $binary_remote_addr zone=auth:10m rate=5r/s;
    
    location / {
        proxy_pass http://vividshift_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Timeouts
        proxy_connect_timeout 5s;
        proxy_send_timeout 60s;
        proxy_read_timeout 60s;
        
        # Rate limiting
        limit_req zone=api burst=20 nodelay;
    }
    
    location /auth/ {
        proxy_pass http://vividshift_backend;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
        proxy_set_header X-Forwarded-For $proxy_add_x_forwarded_for;
        proxy_set_header X-Forwarded-Proto $scheme;
        
        # Stricter rate limiting for auth endpoints
        limit_req zone=auth burst=5 nodelay;
    }
    
    location /health {
        proxy_pass http://vividshift_backend;
        access_log off;
    }
    
    location /metrics {
        proxy_pass http://vividshift_backend;
        # Restrict access to monitoring systems
        allow 10.0.0.0/8;
        allow 172.16.0.0/12;
        allow 192.168.0.0/16;
        deny all;
    }
}
```

### HAProxy Configuration
```haproxy
# /etc/haproxy/haproxy.cfg
global
    daemon
    maxconn 4096
    log stdout local0

defaults
    mode http
    timeout connect 5s
    timeout client 30s
    timeout server 30s
    option httplog

frontend vividshift_frontend
    bind *:80
    bind *:443 ssl crt /path/to/cert.pem
    redirect scheme https if !{ ssl_fc }
    
    # Rate limiting
    stick-table type ip size 100k expire 30s store http_req_rate(10s)
    http-request track-sc0 src
    http-request reject if { sc_http_req_rate(0) gt 20 }
    
    default_backend vividshift_backend

backend vividshift_backend
    balance roundrobin
    option httpchk GET /health
    
    server app1 app1:8080 check inter 10s
    server app2 app2:8080 check inter 10s
    server app3 app3:8080 check inter 10s
```

## Monitoring and Observability

### Prometheus Configuration
```yaml
# prometheus.yml
global:
  scrape_interval: 15s
  evaluation_interval: 15s

scrape_configs:
  - job_name: 'vividshift'
    static_configs:
      - targets: ['app:8080']
    metrics_path: /metrics
    scrape_interval: 10s
    
  - job_name: 'postgres'
    static_configs:
      - targets: ['postgres-exporter:9187']
    
  - job_name: 'redis'
    static_configs:
      - targets: ['redis-exporter:9121']

rule_files:
  - "alert_rules.yml"

alerting:
  alertmanagers:
    - static_configs:
        - targets:
          - alertmanager:9093
```

### Grafana Dashboard Configuration
```json
{
  "dashboard": {
    "title": "VividShift Monitoring",
    "panels": [
      {
        "title": "Request Rate",
        "type": "graph",
        "targets": [
          {
            "expr": "rate(http_requests_total[5m])",
            "legendFormat": "{{method}} {{endpoint}}"
          }
        ]
      },
      {
        "title": "Response Time",
        "type": "graph",
        "targets": [
          {
            "expr": "histogram_quantile(0.95, rate(http_request_duration_seconds_bucket[5m]))",
            "legendFormat": "95th percentile"
          }
        ]
      },
      {
        "title": "Database Connections",
        "type": "singlestat",
        "targets": [
          {
            "expr": "database_connections_active",
            "legendFormat": "Active Connections"
          }
        ]
      }
    ]
  }
}
```

## Backup and Recovery

### Automated Backup Setup
```bash
# Create backup script
cat > /etc/cron.daily/vividshift-backup << 'EOF'
#!/bin/bash
set -e

BACKUP_DIR="/var/backups/vividshift"
DATE=$(date +%Y%m%d_%H%M%S)
BACKUP_FILE="vividshift_backup_${DATE}.sql.gz"

# Create backup
docker-compose exec -T db pg_dump -U postgres vividshift_prod | gzip > "${BACKUP_DIR}/${BACKUP_FILE}"

# Verify backup
gunzip -t "${BACKUP_DIR}/${BACKUP_FILE}"

# Clean old backups (keep 30 days)
find "${BACKUP_DIR}" -name "*.sql.gz" -mtime +30 -delete

# Upload to cloud storage (optional)
# aws s3 cp "${BACKUP_DIR}/${BACKUP_FILE}" s3://your-backup-bucket/
EOF

chmod +x /etc/cron.daily/vividshift-backup
```

### Disaster Recovery Procedures
```bash
# 1. Stop application
docker-compose stop app

# 2. Restore database
gunzip -c /var/backups/vividshift/backup_file.sql.gz | \
  docker-compose exec -T db psql -U postgres vividshift_prod

# 3. Run any pending migrations
docker-compose exec app cargo run --bin db_cli migrate

# 4. Verify data integrity
docker-compose exec app cargo run --bin db_cli validate

# 5. Start application
docker-compose start app

# 6. Verify functionality
curl -f http://localhost:8080/health
```

## Security Hardening

### Application Security
```bash
# Use non-root user in containers
RUN useradd -r -s /bin/false vividshift
USER vividshift

# Set secure file permissions
RUN chmod 600 /app/config/*.toml
RUN chmod 700 /app/logs

# Remove unnecessary packages
RUN apt-get remove --purge -y build-essential && \
    apt-get autoremove -y && \
    rm -rf /var/lib/apt/lists/*
```

### Network Security
```bash
# Configure firewall
ufw allow 22/tcp    # SSH
ufw allow 80/tcp    # HTTP
ufw allow 443/tcp   # HTTPS
ufw deny 8080/tcp   # Block direct app access
ufw deny 5432/tcp   # Block direct DB access
ufw enable

# Use Docker networks
docker network create vividshift_network --driver bridge
```

### SSL/TLS Configuration
```bash
# Generate SSL certificate (Let's Encrypt)
certbot --nginx -d your-domain.com

# Or use custom certificates
openssl req -x509 -nodes -days 365 -newkey rsa:2048 \
  -keyout /etc/ssl/private/vividshift.key \
  -out /etc/ssl/certs/vividshift.crt
```

## Performance Optimization

### Application Tuning
```toml
# backend/config/prod.toml
[database]
max_connections = 50
min_connections = 10
connect_timeout = 10

[server]
worker_threads = 8
max_blocking_threads = 512

[performance]
request_timeout = 30
keep_alive_timeout = 75
```

### Database Optimization
```sql
-- PostgreSQL configuration
ALTER SYSTEM SET shared_buffers = '256MB';
ALTER SYSTEM SET effective_cache_size = '1GB';
ALTER SYSTEM SET work_mem = '4MB';
ALTER SYSTEM SET maintenance_work_mem = '64MB';
ALTER SYSTEM SET checkpoint_completion_target = 0.9;
ALTER SYSTEM SET wal_buffers = '16MB';
ALTER SYSTEM SET default_statistics_target = 100;
SELECT pg_reload_conf();
```

### System-Level Optimization
```bash
# Increase file descriptor limits
echo "* soft nofile 65536" >> /etc/security/limits.conf
echo "* hard nofile 65536" >> /etc/security/limits.conf

# Optimize kernel parameters
echo "net.core.somaxconn = 65535" >> /etc/sysctl.conf
echo "net.ipv4.tcp_max_syn_backlog = 65535" >> /etc/sysctl.conf
sysctl -p
```

## Troubleshooting

### Common Deployment Issues

#### Container Startup Failures
```bash
# Check container logs
docker-compose logs app

# Check container status
docker-compose ps

# Inspect container configuration
docker inspect vividshift_app

# Debug container interactively
docker-compose exec app /bin/bash
```

#### Database Connection Issues
```bash
# Test database connectivity
docker-compose exec app psql $VIVIDSHIFT_DATABASE_URL -c "SELECT 1;"

# Check database logs
docker-compose logs db

# Verify network connectivity
docker-compose exec app ping db
```

#### Performance Issues
```bash
# Monitor resource usage
docker stats

# Check application metrics
curl http://localhost:8080/metrics

# Analyze slow queries
docker-compose exec db psql -U postgres -c "
  SELECT query, mean_time, calls 
  FROM pg_stat_statements 
  ORDER BY mean_time DESC 
  LIMIT 10;
"
```

### Health Check Procedures
```bash
# Comprehensive health check script
#!/bin/bash
set -e

echo "Checking application health..."
curl -f http://localhost:8080/health

echo "Checking database connectivity..."
docker-compose exec -T app cargo run --bin db_cli status

echo "Checking authentication..."
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password123"}' | jq -r '.token')

echo "Testing API functionality..."
curl -f -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/work-groups/history

echo "All health checks passed!"
```

## References
- [Getting Started Guide](GETTING_STARTED.md) - Quick setup instructions
- [Configuration Reference](CONFIGURATION.md) - Detailed configuration options
- [API Documentation](API_REFERENCE.md) - API endpoint reference
- [Architecture Overview](ARCHITECTURE.md) - System design documentation
- [Database Operations](database/OPERATIONS.md) - Database maintenance procedures
