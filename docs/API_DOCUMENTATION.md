# 🚀 VividShift Generic Assignment Engine API Documentation

## Overview

The VividShift Generic Assignment Engine is a domain-agnostic system for managing entity assignments through configurable rules and strategies. It has been transformed from a simple work group CLI tool into a production-ready web service.

## 🔐 Authentication

All protected endpoints require JWT authentication via the `Authorization` header:

```
Authorization: Bearer <jwt_token>
```

### Get Authentication Token

```bash
POST /auth/login
Content-Type: application/json

{
  "username": "admin",
  "password": "password123"
}
```

**Response:**
```json
{
  "token": "eyJ0eXAiOiJKV1QiLCJhbGciOiJIUzI1NiJ9...",
  "user": {
    "id": "uuid",
    "username": "admin",
    "role": "Admin"
  }
}
```

## 🎯 Generic Assignment Endpoints

### Generate Assignments

Create assignments using configurable strategies and rules.

```bash
POST /api/assignments/generate
Authorization: Bearer <token>
Content-Type: application/json

{
  "strategy": "balanced_rotation",
  "participants": ["uuid1", "uuid2"],  // Optional: filter participants
  "targets": ["uuid3", "uuid4"],       // Optional: filter targets
  "parameters": {
    "rotation_weight": 0.8,
    "balance_weight": 0.2,
    "max_attempts": 25
  },
  "constraints": [
    {
      "name": "max_assignments_per_participant",
      "constraint_type": "limit",
      "parameters": {"max": 3}
    }
  ]
}
```

**Response:**
```json
{
  "id": "assignment-uuid",
  "strategy_used": "balanced_rotation",
  "assignments": [
    {
      "participant_id": "uuid1",
      "target_id": "uuid3",
      "confidence": 0.85,
      "metadata": {
        "strategy": "balanced_rotation",
        "score": 0.85,
        "target_name": "Parlor"
      }
    }
  ],
  "metadata": {
    "attempts": 1,
    "execution_time_ms": 45,
    "strategy_parameters": {...},
    "validation_results": [...]
  },
  "created_at": "2025-01-26T09:00:00Z"
}
```

## 🏗️ Entity Management Endpoints

### List Entities

```bash
GET /api/entities/{entity_type}
Authorization: Bearer <token>
```

### Create Entity

```bash
POST /api/entities/{entity_type}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Alice",
  "group": "TeamA",
  "availability": true,
  "skills": ["leadership", "communication"]
}
```

### Get Specific Entity

```bash
GET /api/entities/{entity_type}/{entity_id}
Authorization: Bearer <token>
```

### Update Entity

```bash
PUT /api/entities/{entity_type}/{entity_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "availability": false,
  "skills": ["leadership", "communication", "project_management"]
}
```

### Delete Entity

```bash
DELETE /api/entities/{entity_type}/{entity_id}
Authorization: Bearer <token>
```

## ⚙️ Rule Engine Endpoints

### List Available Strategies

```bash
GET /api/rules/strategies
Authorization: Bearer <token>
```

**Response:**
```json
["balanced_rotation", "random_assignment", "skill_based"]
```

### List Available Validators

```bash
GET /api/rules/validators
Authorization: Bearer <token>
```

**Response:**
```json
["capacity_check", "availability_check", "skill_matching"]
```

## 🔄 Legacy Endpoints (Backward Compatibility)

### Generate Work Groups (Legacy)

```bash
POST /api/work-groups/generate
Authorization: Bearer <token>
Content-Type: application/json

{
  "names_a": ["Alice", "Bob"],
  "names_b": ["Charlie", "David"],
  "custom_assignments": {
    "Parlor": 3,
    "Kitchen": 2
  }
}
```

### Get Assignment History

```bash
GET /api/work-groups/history
Authorization: Bearer <token>
```

### Get/Update Assignment Configuration

```bash
GET /api/work-groups/assignments
POST /api/work-groups/assignments
Authorization: Bearer <token>
```

## 📊 Health & Monitoring Endpoints

### Health Check

```bash
GET /health
```

### Readiness Check

```bash
GET /ready
```

### Metrics (Prometheus)

```bash
GET /metrics
```

## 🌍 Environment Configuration

The system supports multiple environments through configuration files:

- **Development**: `backend/config/dev.toml`
- **Staging**: `backend/config/staging.toml`
- **Production**: `backend/config/prod.toml`

### Environment Variables

```bash
# Core Configuration
ENVIRONMENT=dev|staging|prod
VIVIDSHIFT_SERVER_HOST=127.0.0.1
VIVIDSHIFT_SERVER_PORT=8080

# Database
VIVIDSHIFT_DATABASE_URL=postgresql://user:pass@host:port/db

# Authentication
VIVIDSHIFT_AUTH_JWT_SECRET=your-secret-key
VIVIDSHIFT_AUTH_JWT_EXPIRATION=86400

# Logging
VIVIDSHIFT_LOGGING_LEVEL=debug|info|warn|error
VIVIDSHIFT_LOGGING_FILE_ENABLED=true
VIVIDSHIFT_LOGGING_JSON_FORMAT=false
```

## 🎨 Domain Configuration

Create new domains by adding configuration files:

### Domain Definition (`backend/config/domain/your_domain.toml`)

```toml
[domain]
name = "your_domain"
display_name = "Your Domain System"
description = "Custom assignment domain"

[entities.your_entity]
name = "your_entity"
display_name = "Your Entity"

[entities.your_entity.fields.name]
field_type = "String"
required = true
constraints = { min_length = 1, max_length = 100 }

[entities.your_entity.fields.capacity]
field_type = "Integer"
required = true
constraints = { min = 1, max = 50 }
```

### Rule Configuration (`backend/config/rules/your_rules.toml`)

```toml
[assignment_strategy]
name = "your_strategy"
parameters = { weight = 0.5, threshold = 10 }

[validation_rules]
your_validation = { enabled = true, strict = false }
```

## 🔌 Extending the System

### Adding New Assignment Strategies

1. Implement the `AssignmentStrategy` trait:

```rust
use async_trait::async_trait;
use crate::services::rule_engine::AssignmentStrategy;

pub struct YourStrategy;

#[async_trait]
impl AssignmentStrategy for YourStrategy {
    async fn execute(&self, participants: &[GenericEntity], 
                    targets: &[GenericEntity], 
                    config: &StrategyConfig) -> Result<Vec<Assignment>> {
        // Your custom logic here
    }
    
    fn name(&self) -> &str { "your_strategy" }
    fn description(&self) -> &str { "Your custom strategy" }
    fn validate_config(&self, config: &StrategyConfig) -> Result<()> { Ok(()) }
}
```

2. Register in `main.rs`:

```rust
rule_engine.register_strategy(YourStrategy);
```

### Adding New Validation Rules

1. Implement the `ValidationRule` trait:

```rust
use async_trait::async_trait;
use crate::services::rule_engine::ValidationRule;

pub struct YourValidator;

#[async_trait]
impl ValidationRule for YourValidator {
    async fn validate(&self, assignments: &[Assignment], 
                     participants: &[GenericEntity], 
                     targets: &[GenericEntity],
                     config: &HashMap<String, serde_json::Value>) -> Result<ValidationResult> {
        // Your validation logic here
    }
    
    fn name(&self) -> &str { "your_validator" }
    fn severity(&self) -> ValidationSeverity { ValidationSeverity::Warning }
}
```

2. Register in `main.rs`:

```rust
rule_engine.register_validator(YourValidator);
```

## 🚀 Quick Start Examples

### 1. Basic Assignment Generation

```bash
# 1. Get authentication token
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}' | jq -r '.token')

# 2. Create participants
curl -X POST http://localhost:8080/api/entities/participant \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "group": "TeamA", "availability": true}'

# 3. Generate assignments
curl -X POST http://localhost:8080/api/assignments/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"strategy": "balanced_rotation", "participants": [], "targets": []}'
```

### 2. Custom Domain Setup

```bash
# 1. Create domain configuration file
cat > backend/config/domain/team_scheduling.toml << EOF
[domain]
name = "team_scheduling"
display_name = "Team Scheduling System"

[entities.shift]
name = "shift"
display_name = "Work Shift"

[entities.shift.fields.start_time]
field_type = "DateTime"
required = true
EOF

# 2. Restart application to load new domain
docker-compose restart app
```

## ⚠️ Error Handling

The API returns standard HTTP status codes:

- `200` - Success
- `400` - Bad Request (validation errors)
- `401` - Unauthorized (invalid/missing token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `500` - Internal Server Error

Error responses include details:

```json
{
  "error": "Validation failed",
  "details": "Required field 'name' is missing",
  "code": "VALIDATION_ERROR"
}
```
