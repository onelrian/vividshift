# VividShift Configuration Reference

## Overview
Complete configuration reference for VividShift Generic Assignment Engine. This document covers all configuration options, environment variables, and customization possibilities.

**Target Audience:** System administrators, DevOps engineers, developers

## Configuration System

VividShift uses a hierarchical configuration system that loads settings in the following order:

1. **Default configuration** (`backend/config/default.toml`)
2. **Environment-specific overrides** (`backend/config/{dev,staging,prod}.toml`)
3. **Environment variables** (prefixed with `VIVIDSHIFT_`)
4. **Local overrides** (`backend/config/local.toml` - optional, not in git)

Later sources override earlier ones, allowing flexible customization for different environments.

## Environment Variables

### Required Variables

```bash
# Database connection
VIVIDSHIFT_DATABASE_URL=postgresql://postgres:password@localhost:5432/vividshift_dev

# Authentication
VIVIDSHIFT_AUTH_JWT_SECRET=your-secure-secret-key-change-in-production
```

### Server Configuration

```bash
# Server binding
VIVIDSHIFT_SERVER_HOST=0.0.0.0              # Default: 127.0.0.1
VIVIDSHIFT_SERVER_PORT=8080                  # Default: 8080
ENVIRONMENT=dev                              # Options: dev, staging, prod
```

### Database Configuration

```bash
# Connection settings
VIVIDSHIFT_DATABASE_URL=postgresql://user:pass@host:port/database
VIVIDSHIFT_DATABASE_MAX_CONNECTIONS=10      # Default: 10
VIVIDSHIFT_DATABASE_MIN_CONNECTIONS=1       # Default: 1
VIVIDSHIFT_DATABASE_CONNECT_TIMEOUT=30      # Seconds, default: 30
```

### Authentication Configuration

```bash
# JWT settings
VIVIDSHIFT_AUTH_JWT_SECRET=your-secret-key
VIVIDSHIFT_AUTH_JWT_EXPIRATION=86400         # Seconds, default: 86400 (24 hours)
VIVIDSHIFT_AUTH_BCRYPT_COST=12               # Default: 12
```

### Logging Configuration

```bash
# Logging settings
VIVIDSHIFT_LOGGING_LEVEL=info                # Options: debug, info, warn, error
VIVIDSHIFT_LOGGING_FILE_ENABLED=false       # Default: false
VIVIDSHIFT_LOGGING_FILE_PATH=logs/app.log    # Default: logs/app.log
VIVIDSHIFT_LOGGING_JSON_FORMAT=false        # Default: false
RUST_LOG=info                                # Rust-specific logging
RUST_BACKTRACE=1                             # Enable backtraces: 0, 1, full
```

## Configuration Files

### Default Configuration (`backend/config/default.toml`)

```toml
[server]
host = "0.0.0.0"
port = 8080
environment = "dev"

[database]
url = "postgresql://localhost:5432/vividshift_dev"
max_connections = 10
min_connections = 1
connect_timeout = 30

[auth]
jwt_secret = "dev-secret-change-in-production"
jwt_expiration = 86400  # 24 hours in seconds
bcrypt_cost = 12

[logging]
level = "info"
file_enabled = false
file_path = "logs/app.log"
json_format = false

[domain]
name = "work_groups"
display_name = "Work Group Assignment System"
description = "Default work group assignment domain"
version = "1.0"

[rule_engine]
default_strategy = "balanced_rotation"
max_execution_time_ms = 30000
enable_parallel_processing = true
validation_mode = "Permissive"

[engines]
assignment_engines = ["balanced_rotation", "random_assignment", "skill_based"]
validation_engines = ["capacity_check", "availability_check", "skill_matching"]
distribution_algorithms = ["workload_balancer"]
default_strategy = "balanced_rotation"
```

### Environment-Specific Overrides

#### Development (`backend/config/dev.toml`)
```toml
[server]
host = "127.0.0.1"

[logging]
level = "debug"
file_enabled = true

[database]
url = "postgresql://postgres:password@localhost:5432/vividshift_dev"

[rule_engine]
validation_mode = "Permissive"
```

#### Staging (`backend/config/staging.toml`)
```toml
[server]
host = "0.0.0.0"
environment = "staging"

[logging]
level = "info"
file_enabled = true
json_format = true

[database]
max_connections = 20
min_connections = 5

[rule_engine]
validation_mode = "Strict"
```

#### Production (`backend/config/prod.toml`)
```toml
[server]
host = "0.0.0.0"
environment = "prod"

[logging]
level = "warn"
file_enabled = true
json_format = true

[database]
max_connections = 50
min_connections = 10
connect_timeout = 10

[auth]
bcrypt_cost = 14

[rule_engine]
validation_mode = "Strict"
max_execution_time_ms = 10000
```

## Domain Configuration

### Creating Custom Domains

Create domain-specific configurations in `backend/config/domain/`:

#### Example: Team Scheduling (`backend/config/domain/team_scheduling.toml`)

```toml
[domain]
name = "team_scheduling"
display_name = "Team Scheduling System"
description = "Assign team members to shifts and responsibilities"
version = "1.0.0"

[entities.shift]
name = "shift"
display_name = "Work Shift"
description = "A scheduled work period"
version = "1.0"

[entities.shift.fields.name]
field_type = "String"
required = true
display_name = "Shift Name"
description = "Name of the work shift"

[entities.shift.fields.name.constraints]
min_length = 1
max_length = 100
unique = true

[entities.shift.fields.start_time]
field_type = "DateTime"
required = true
display_name = "Start Time"
description = "When the shift begins"

[entities.shift.fields.duration_hours]
field_type = "Integer"
required = true
default_value = 8
display_name = "Duration (Hours)"
description = "Length of shift in hours"

[entities.shift.fields.duration_hours.constraints]
min = 1
max = 24

[entities.shift.fields.required_skills]
field_type = "Array"
required = false
display_name = "Required Skills"
description = "Skills needed for this shift"

[entities.team_member]
name = "team_member"
display_name = "Team Member"
description = "A person who can be assigned to shifts"
version = "1.0"

[entities.team_member.fields.name]
field_type = "String"
required = true
display_name = "Name"
description = "Team member's name"

[entities.team_member.fields.skills]
field_type = "Array"
required = false
display_name = "Skills"
description = "Team member's skills"

[entities.team_member.fields.availability]
field_type = "Object"
required = false
display_name = "Availability"
description = "When the team member is available"

[default_data.shifts]
morning = { name = "Morning Shift", start_time = "08:00", duration_hours = 8 }
afternoon = { name = "Afternoon Shift", start_time = "16:00", duration_hours = 8 }
night = { name = "Night Shift", start_time = "00:00", duration_hours = 8 }

[[business_rules]]
name = "no_double_booking"
description = "Prevent assigning same person to overlapping shifts"
rule_type = "Validation"
condition = "!has_time_conflict(participant, shift)"
action = "reject_assignment"
priority = 1
enabled = true

[[business_rules]]
name = "skill_requirements"
description = "Ensure assigned person has required skills"
rule_type = "Validation"
condition = "participant.skills.contains_all(shift.required_skills)"
action = "warn_assignment"
priority = 2
enabled = true
```

### Field Types and Constraints

#### Supported Field Types
- `String` - Text values
- `Integer` - Whole numbers
- `Float` - Decimal numbers
- `Boolean` - True/false values
- `DateTime` - Date and time values
- `Array` - Lists of values
- `Object` - Complex nested data

#### Field Constraints
```toml
[entities.example.fields.text_field.constraints]
min_length = 1
max_length = 255
pattern = "^[A-Za-z0-9_]+$"
unique = true

[entities.example.fields.number_field.constraints]
min = 0
max = 100

[entities.example.fields.array_field.constraints]
min_items = 1
max_items = 10
allowed_values = ["option1", "option2", "option3"]
```

## Rule Engine Configuration

### Assignment Strategies Configuration

```toml
[rule_engine.strategies.balanced_rotation]
enabled = true
default_parameters = { rotation_weight = 0.8, balance_weight = 0.2 }
max_attempts = 50

[rule_engine.strategies.skill_based]
enabled = true
default_parameters = { skill_threshold = 0.7, fallback_strategy = "random_assignment" }

[rule_engine.strategies.random_assignment]
enabled = true
default_parameters = { seed = 42, ensure_capacity = true }
```

### Validation Rules Configuration

```toml
[rule_engine.validators.capacity_check]
enabled = true
severity = "Error"
default_parameters = { strict_mode = true, tolerance = 0.1 }

[rule_engine.validators.availability_check]
enabled = true
severity = "Warning"
default_parameters = { check_conflicts = true, time_window = "24h" }

[rule_engine.validators.skill_matching]
enabled = true
severity = "Warning"
default_parameters = { required_match = 0.8, report_missing = true }
```

## Advanced Configuration

### Custom Rule Configuration (`backend/config/rules/assignment.toml`)

```toml
[assignment_strategy]
name = "custom_rotation"
description = "Custom rotation with priority weighting"
enabled = true

[assignment_strategy.parameters]
rotation_weight = 0.6
priority_weight = 0.3
balance_weight = 0.1
max_attempts = 25

[assignment_strategy.constraints]
max_assignments_per_participant = 3
min_assignments_per_participant = 1
respect_availability = true

[validation_rules]
strict_capacity = { enabled = true, tolerance = 0.05 }
skill_matching = { enabled = true, threshold = 0.9 }
availability_check = { enabled = true, strict = false }

[distribution_algorithm]
name = "weighted_round_robin"
parameters = { weight_factor = 1.5, randomization = 0.1 }
```

### Performance Configuration

```toml
[performance]
# Connection pool settings
database_pool_size = 20
database_pool_timeout = 30
database_idle_timeout = 600

# Processing limits
max_concurrent_assignments = 10
assignment_timeout_ms = 30000
validation_timeout_ms = 5000

# Caching settings
cache_enabled = true
cache_ttl_seconds = 3600
cache_max_size = 1000
```

### Monitoring Configuration

```toml
[monitoring]
# Metrics collection
metrics_enabled = true
metrics_port = 9090
detailed_metrics = false

# Health checks
health_check_interval = 30
database_health_timeout = 5
cache_health_timeout = 3

# Alerting thresholds
response_time_threshold_ms = 1000
error_rate_threshold = 0.05
connection_pool_threshold = 0.8
```

## Security Configuration

### Production Security Settings

```toml
[security]
# CORS settings
cors_enabled = true
cors_origins = ["https://yourdomain.com"]
cors_methods = ["GET", "POST", "PUT", "DELETE"]
cors_headers = ["Authorization", "Content-Type"]

# Rate limiting
rate_limiting_enabled = true
requests_per_minute = 60
burst_size = 10

# Security headers
security_headers_enabled = true
hsts_enabled = true
content_security_policy = "default-src 'self'"

# Session security
secure_cookies = true
same_site_cookies = "Strict"
session_timeout = 3600
```

### SSL/TLS Configuration

```toml
[tls]
enabled = true
cert_file = "/path/to/cert.pem"
key_file = "/path/to/key.pem"
min_version = "1.2"
cipher_suites = ["TLS_AES_256_GCM_SHA384", "TLS_CHACHA20_POLY1305_SHA256"]
```

## Configuration Validation

### Startup Validation

VividShift validates configuration at startup and reports errors:

```bash
# Check configuration validity
cargo run --bin config_validator

# Validate specific environment
ENVIRONMENT=prod cargo run --bin config_validator
```

### Configuration Schema

The system validates configuration against predefined schemas:

```rust
// Example validation rules
pub struct ServerConfig {
    #[validate(range(min = 1, max = 65535))]
    pub port: u16,
    
    #[validate(ip)]
    pub host: String,
    
    #[validate(custom = "validate_environment")]
    pub environment: String,
}
```

## Environment-Specific Examples

### Docker Compose Environment

```yaml
# docker-compose.yml
services:
  app:
    environment:
      - ENVIRONMENT=prod
      - VIVIDSHIFT_DATABASE_URL=postgresql://postgres:password@db:5432/vividshift_prod
      - VIVIDSHIFT_AUTH_JWT_SECRET=${JWT_SECRET}
      - VIVIDSHIFT_LOGGING_LEVEL=info
      - VIVIDSHIFT_LOGGING_JSON_FORMAT=true
```

### Kubernetes ConfigMap

```yaml
apiVersion: v1
kind: ConfigMap
metadata:
  name: vividshift-config
data:
  ENVIRONMENT: "prod"
  VIVIDSHIFT_SERVER_HOST: "0.0.0.0"
  VIVIDSHIFT_SERVER_PORT: "8080"
  VIVIDSHIFT_LOGGING_LEVEL: "info"
  VIVIDSHIFT_LOGGING_JSON_FORMAT: "true"
  VIVIDSHIFT_DATABASE_MAX_CONNECTIONS: "50"
```

### AWS ECS Task Definition

```json
{
  "environment": [
    {"name": "ENVIRONMENT", "value": "prod"},
    {"name": "VIVIDSHIFT_DATABASE_URL", "value": "postgresql://..."},
    {"name": "VIVIDSHIFT_AUTH_JWT_SECRET", "valueFrom": "arn:aws:secretsmanager:..."}
  ]
}
```

## Configuration Best Practices

### Security Best Practices

1. **Never commit secrets** to version control
2. **Use environment variables** for sensitive data
3. **Rotate JWT secrets** regularly
4. **Use strong bcrypt costs** in production (14+)
5. **Enable SSL/TLS** for database connections

### Performance Best Practices

1. **Tune connection pools** based on load
2. **Use appropriate log levels** (warn/error in production)
3. **Enable JSON logging** for structured log analysis
4. **Configure timeouts** appropriately
5. **Monitor resource usage** and adjust limits

### Operational Best Practices

1. **Use environment-specific configs** for different deployments
2. **Validate configuration** before deployment
3. **Document custom configurations** thoroughly
4. **Test configuration changes** in staging first
5. **Monitor configuration drift** in production

## Troubleshooting Configuration

### Common Issues

#### Configuration Not Loading
```bash
# Check file permissions
ls -la backend/config/

# Verify TOML syntax
toml-lint backend/config/default.toml

# Check environment variable format
env | grep VIVIDSHIFT_
```

#### Database Connection Issues
```bash
# Test database connectivity
psql $VIVIDSHIFT_DATABASE_URL -c "SELECT 1;"

# Check connection pool settings
# Ensure max_connections > expected concurrent users
```

#### Authentication Problems
```bash
# Verify JWT secret is set
echo $VIVIDSHIFT_AUTH_JWT_SECRET

# Check token expiration settings
# Ensure expiration is appropriate for your use case
```

### Configuration Debugging

Enable debug logging to troubleshoot configuration issues:

```bash
VIVIDSHIFT_LOGGING_LEVEL=debug cargo run
```

This will show detailed information about configuration loading and validation.

## References
- [Getting Started](GETTING_STARTED.md) - Quick setup guide
- [Architecture](ARCHITECTURE.md) - System architecture overview
- [API Reference](API_REFERENCE.md) - API endpoint documentation
- [Security Guide](SECURITY.md) - Security configuration details
- [Deployment Guide](DEPLOYMENT.md) - Production deployment procedures
