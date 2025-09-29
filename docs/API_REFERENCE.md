# VividShift API Reference

## Overview
Complete API documentation for the VividShift Generic Assignment Engine. This document covers all endpoints, request/response formats, authentication, and usage examples.

**Target Audience:** Frontend developers, API consumers, integration developers

## Base URL
```
http://localhost:8080  # Development
https://your-domain.com  # Production
```

## Authentication

All protected endpoints require JWT authentication via the `Authorization` header:

```
Authorization: Bearer <jwt_token>
```

### Authentication Endpoints

#### Register User
```http
POST /auth/register
Content-Type: application/json

{
  "username": "admin",
  "email": "admin@example.com",
  "password": "password123"
}
```

**Response:**
```json
{
  "message": "User registered successfully",
  "user": {
    "id": "uuid",
    "username": "admin",
    "email": "admin@example.com",
    "role": "user"
  }
}
```

#### Login
```http
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
    "role": "admin"
  }
}
```

## Assignment Endpoints

### Generate Assignments (Advanced)

Create assignments using configurable strategies and rules.

```http
POST /api/assignments/generate
Authorization: Bearer <token>
Content-Type: application/json

{
  "strategy": "balanced_rotation",
  "participants": ["uuid1", "uuid2"],
  "targets": ["uuid3", "uuid4"],
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
    "strategy_parameters": {},
    "validation_results": []
  },
  "created_at": "2025-01-26T09:00:00Z"
}
```

### Generate Work Groups (Legacy)

Backward-compatible endpoint for simple work group assignments.

```http
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

**Response:**
```json
{
  "assignments": {
    "Parlor": ["Alice", "Bob", "Charlie"],
    "Kitchen": ["David", "Alice"]
  },
  "generation_info": {
    "attempt_number": 1,
    "timestamp": "2024-01-01T12:00:00Z",
    "total_people": 4,
    "total_assignments": 5
  }
}
```

### Get Assignment History

```http
GET /api/work-groups/history
Authorization: Bearer <token>
```

**Response:**
```json
{
  "history": {
    "2024-01-01": {
      "assignments": {},
      "generation_info": {}
    }
  },
  "last_updated": "2024-01-01T12:00:00Z"
}
```

## Entity Management Endpoints

### List Entities

```http
GET /api/entities/{entity_type}
Authorization: Bearer <token>
```

**Parameters:**
- `entity_type`: `participant` or `assignment_target`

**Response:**
```json
[
  {
    "id": "uuid",
    "name": "Alice Johnson",
    "attributes": {
      "skills": ["cleaning", "organizing"],
      "availability": true
    },
    "created_at": "2024-01-01T12:00:00Z"
  }
]
```

### Create Entity

```http
POST /api/entities/{entity_type}
Authorization: Bearer <token>
Content-Type: application/json

{
  "name": "Alice Johnson",
  "group": "TeamA",
  "availability": true,
  "skills": ["leadership", "communication"]
}
```

**Response:**
```json
{
  "id": "uuid",
  "name": "Alice Johnson",
  "entity_type": "participant",
  "attributes": {
    "group": "TeamA",
    "availability": true,
    "skills": ["leadership", "communication"]
  },
  "created_at": "2024-01-01T12:00:00Z"
}
```

### Get Specific Entity

```http
GET /api/entities/{entity_type}/{entity_id}
Authorization: Bearer <token>
```

### Update Entity

```http
PUT /api/entities/{entity_type}/{entity_id}
Authorization: Bearer <token>
Content-Type: application/json

{
  "availability": false,
  "skills": ["leadership", "communication", "project_management"]
}
```

### Delete Entity

```http
DELETE /api/entities/{entity_type}/{entity_id}
Authorization: Bearer <token>
```

## Rule Engine Endpoints

### List Available Strategies

```http
GET /api/rules/strategies
Authorization: Bearer <token>
```

**Response:**
```json
["balanced_rotation", "random_assignment", "skill_based"]
```

### List Available Validators

```http
GET /api/rules/validators
Authorization: Bearer <token>
```

**Response:**
```json
["capacity_check", "availability_check", "skill_matching"]
```

## Health & Monitoring Endpoints

### Health Check

```http
GET /health
```

**Response:**
```json
{
  "service": "vividshift-backend",
  "status": "healthy"
}
```

### Readiness Check

```http
GET /ready
```

**Response:**
```json
{
  "status": "ready",
  "database": "connected",
  "cache": "connected"
}
```

### Metrics (Prometheus)

```http
GET /metrics
```

Returns Prometheus-formatted metrics for monitoring.

## Assignment Strategies

### Balanced Rotation
- **Name:** `balanced_rotation`
- **Description:** Distributes assignments evenly considering history
- **Parameters:**
  - `rotation_weight` (0.0-1.0): Weight for rotation fairness
  - `balance_weight` (0.0-1.0): Weight for workload balance
  - `max_attempts` (integer): Maximum retry attempts

### Skill-Based Assignment
- **Name:** `skill_based`
- **Description:** Matches participants to targets based on required skills
- **Parameters:**
  - `skill_threshold` (0.0-1.0): Minimum skill match percentage
  - `fallback_strategy` (string): Strategy when skills don't match

### Random Assignment
- **Name:** `random_assignment`
- **Description:** Random distribution with capacity constraints
- **Parameters:**
  - `seed` (integer): Randomization seed for reproducibility
  - `ensure_capacity` (boolean): Guarantee minimum capacity

## Validation Rules

### Capacity Check
- **Name:** `capacity_check`
- **Description:** Ensures targets have sufficient participants
- **Severity:** Error
- **Parameters:**
  - `strict_mode` (boolean): Fail on any capacity violation
  - `tolerance` (0.0-1.0): Acceptable capacity deviation

### Availability Check
- **Name:** `availability_check`
- **Description:** Verifies participant availability
- **Severity:** Warning
- **Parameters:**
  - `check_conflicts` (boolean): Detect scheduling conflicts
  - `time_window` (string): Time window for availability

### Skill Matching
- **Name:** `skill_matching`
- **Description:** Validates skill requirements are met
- **Severity:** Warning
- **Parameters:**
  - `required_match` (0.0-1.0): Required skill match percentage
  - `report_missing` (boolean): Report missing skills

## Error Handling

### HTTP Status Codes
- `200` - Success
- `201` - Created
- `400` - Bad Request (validation errors)
- `401` - Unauthorized (invalid/missing token)
- `403` - Forbidden (insufficient permissions)
- `404` - Not Found
- `409` - Conflict (duplicate resource)
- `422` - Unprocessable Entity (business logic error)
- `500` - Internal Server Error

### Error Response Format

```json
{
  "error": "Validation failed",
  "details": "Required field 'name' is missing",
  "code": "VALIDATION_ERROR",
  "timestamp": "2024-01-01T12:00:00Z"
}
```

### Common Error Codes
- `VALIDATION_ERROR` - Request validation failed
- `AUTHENTICATION_ERROR` - Invalid or expired token
- `AUTHORIZATION_ERROR` - Insufficient permissions
- `RESOURCE_NOT_FOUND` - Requested resource doesn't exist
- `DUPLICATE_RESOURCE` - Resource already exists
- `BUSINESS_LOGIC_ERROR` - Business rule violation
- `INTERNAL_ERROR` - Unexpected server error

## Usage Examples

### Complete Workflow Example

```bash
# 1. Register and login
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username": "admin", "password": "password123"}' | jq -r '.token')

# 2. Create participants
curl -X POST http://localhost:8080/api/entities/participant \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Alice", "group": "TeamA", "availability": true}'

curl -X POST http://localhost:8080/api/entities/participant \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Bob", "group": "TeamB", "availability": true}'

# 3. Create assignment targets
curl -X POST http://localhost:8080/api/entities/assignment_target \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"name": "Kitchen", "required_count": 2}'

# 4. Generate assignments
curl -X POST http://localhost:8080/api/assignments/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"strategy": "balanced_rotation", "participants": [], "targets": []}'
```

### Batch Operations

```bash
# Create multiple participants
for name in Alice Bob Charlie David; do
  curl -X POST http://localhost:8080/api/entities/participant \
    -H "Authorization: Bearer $TOKEN" \
    -H "Content-Type: application/json" \
    -d "{\"name\": \"$name\", \"availability\": true}"
done

# Generate assignments with custom parameters
curl -X POST http://localhost:8080/api/assignments/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "skill_based",
    "parameters": {
      "skill_threshold": 0.7,
      "fallback_strategy": "random_assignment"
    },
    "constraints": [
      {
        "name": "max_assignments_per_participant",
        "constraint_type": "limit",
        "parameters": {"max": 2}
      }
    ]
  }'
```

## References
- [Getting Started](GETTING_STARTED.md) - Quick start guide
- [Configuration](CONFIGURATION.md) - Environment configuration
- [Architecture](ARCHITECTURE.md) - System architecture overview
- [Database Schema](database/SCHEMA.md) - Database design
- [Security Guide](SECURITY.md) - Security implementation
