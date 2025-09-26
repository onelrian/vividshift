# VividShift Deployment Guide

## Prerequisites

- Docker and Docker Compose
- Git for source code management
- 4GB RAM minimum, 8GB recommended
- 2 CPU cores minimum

## Quick Start

### 1. Clone Repository
```bash
git clone https://github.com/onelrian/vividshift.git
cd vividshift
git checkout feature/fullstack-app
```

### 2. Environment Configuration
Copy and customize environment variables:
```bash
cp .env.example .env
# Edit .env with your specific values
```

The application uses a hierarchical configuration system:
1. **Default configuration**: `backend/config/default.toml`
2. **Environment-specific overrides**: `backend/config/{dev,staging,prod}.toml`
3. **Environment variables**: Prefixed with `VIVIDSHIFT_`
4. **Local overrides**: `backend/config/local.toml` (optional, not in git)

Environment variables in `.env` will override configuration file values.

### 3. Start Services
```bash
docker-compose up -d --build
```

## Architecture Overview

VividShift is a domain-agnostic Generic Assignment Engine built with Rust, designed for high-performance assignment generation with pluggable rule engines and validation systems.

### Core Components

**Application State (AppState)**
- Unified state management with configuration, authentication, entity management, and rule engine

**API Layer**
- RESTful API built with Axum framework
- Health endpoints, authentication, work group operations

**Rule Engine System**
- Assignment Strategies: Random, skill-based, and pluggable custom strategies
- Validation Rules: Capacity, availability, and skill matching validators
- Distribution Algorithms: Workload balancing and custom distribution logic

**Entity Management**
- Generic entity system supporting participants and assignment targets
- Flexible data model with CRUD operations and metadata tracking

**Configuration System**
- Hierarchical configuration loading (default → environment → variables)
- Runtime validation and environment-specific overrides

### 4. Verify Deployment
```bash
# Check service status
docker-compose ps

# Test health endpoint
curl http://localhost:8080/health
```

## Environment Variables

The application loads environment variables from `.env` file in development and from the environment in production.

### Required Variables
- `ENVIRONMENT`: Deployment environment (dev, staging, prod)
- `VIVIDSHIFT_DATABASE_URL`: PostgreSQL connection string
- `VIVIDSHIFT_AUTH_JWT_SECRET`: JWT signing secret (change in production)

### Optional Variables
- `VIVIDSHIFT_SERVER_HOST`: Server bind address (default: 0.0.0.0)
- `VIVIDSHIFT_SERVER_PORT`: Server port (default: 8080)
- `VIVIDSHIFT_DATABASE_MAX_CONNECTIONS`: Database pool size (default: 10)
- `VIVIDSHIFT_DATABASE_MIN_CONNECTIONS`: Minimum pool size (default: 1)
- `VIVIDSHIFT_DATABASE_CONNECT_TIMEOUT`: Connection timeout in seconds (default: 30)
- `VIVIDSHIFT_AUTH_JWT_EXPIRATION`: JWT expiration in seconds (default: 86400)
- `VIVIDSHIFT_AUTH_BCRYPT_COST`: Bcrypt hashing cost (default: 12)
- `VIVIDSHIFT_LOGGING_LEVEL`: Log level (debug, info, warn, error)
- `VIVIDSHIFT_LOGGING_FILE_ENABLED`: Enable file logging (default: false)
- `VIVIDSHIFT_LOGGING_FILE_PATH`: Log file path (default: logs/app.log)
- `VIVIDSHIFT_LOGGING_JSON_FORMAT`: Use JSON log format (default: false)
- `RUST_LOG`: Rust-specific logging configuration
- `RUST_BACKTRACE`: Enable Rust backtraces (1 or full)

## Service Ports

- **Application**: 8080 (HTTP API)
- **PostgreSQL**: 5432 (Database)
- **Redis**: 6379 (Cache/Sessions)
- **Prometheus**: 9090 (Metrics)
- **Grafana**: 3000 (Dashboard)

## Production Deployment

### 1. Security Configuration
```bash
# Generate secure JWT secret
openssl rand -hex 32

# Update database credentials
# Configure TLS certificates
# Set up firewall rules
```

### 2. Database Setup
```bash
# Create production database
createdb vividshift_prod

# Run migrations (if applicable)
# Set up database backups
```

### 3. Monitoring Setup
```bash
# Access Grafana dashboard
open http://localhost:3000
# Default credentials: admin/admin

# Configure Prometheus targets
# Set up alerting rules
```

### 4. Load Balancer Configuration
```nginx
upstream vividshift {
    server app1:8080;
    server app2:8080;
}

server {
    listen 80;
    server_name your-domain.com;
    
    location / {
        proxy_pass http://vividshift;
        proxy_set_header Host $host;
        proxy_set_header X-Real-IP $remote_addr;
    }
}
```

## Verification Steps

### 1. Health Checks
```bash
# Application health
curl http://localhost:8080/health
# Expected: {"status": "healthy"}

# Readiness check
curl http://localhost:8080/ready
# Expected: {"status": "ready"}
```

### 2. Authentication Test
```bash
# Register user
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"test","email":"test@example.com","password":"password123"}'

# Login
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"test","password":"password123"}'
```

### 3. Assignment Generation
```bash
# Generate work groups (legacy endpoint)
curl -X POST http://localhost:8080/api/work-groups/generate \
  -H "Content-Type: application/json" \
  -H "Authorization: Bearer <token>" \
  -d '{"names_a":["Alice","Bob"],"names_b":["Charlie","David"]}'
```

### 4. Monitoring Access
```bash
# Prometheus metrics
curl http://localhost:9090/metrics

# Grafana dashboard
open http://localhost:3000
```

## Troubleshooting

### Common Issues

#### Application Won't Start
- Check environment variables are set correctly
- Verify database connectivity
- Review application logs: `docker-compose logs app`

#### Database Connection Errors
- Ensure PostgreSQL is running and accessible
- Verify connection string format
- Check network connectivity between containers

#### Authentication Failures
- Verify JWT secret is configured
- Check token expiration settings
- Review user credentials and roles

#### Performance Issues
- Monitor resource usage: `docker stats`
- Check database query performance
- Review application metrics in Grafana

### Log Analysis
```bash
# View application logs
docker-compose logs -f app

# Check specific service logs
docker-compose logs postgres
docker-compose logs redis

# Monitor all services
docker-compose logs -f
```

## Scaling Considerations

### Horizontal Scaling
- Multiple application instances behind load balancer
- Shared database and Redis instances
- Session affinity not required (stateless design)

### Database Scaling
- Read replicas for query performance
- Connection pooling optimization
- Database partitioning for large datasets

### Monitoring Scaling
- Prometheus federation for multi-cluster monitoring
- Grafana organization for team separation
- Alert manager for notification routing
