# VividShift User Manual

## Quick Start Guide

### 1. Register User
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"alice","email":"alice@example.com","password":"password123"}'
```

### 2. Generate Assignments
```bash
TOKEN="your_token_from_registration"
curl -X POST http://localhost:8080/api/work-groups/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"names_a":["Alice","Bob"],"names_b":["Charlie","David"]}'
```

### 3. View History
```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/work-groups/history
```

### 4. View Assignments Config
```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/work-groups/assignments
```

## Health Checks
```bash
curl http://localhost:8080/health
curl http://localhost:8080/ready
```

## Error Codes
- 200: Success
- 400: Bad request/invalid JSON
- 401: Unauthorized/invalid token
- 404: Not found

## Core Features

### Assignment Generation
- Multi-Strategy Support: Random, skill-based, and custom assignment algorithms
- Constraint Validation: Capacity limits, availability checks, skill requirements
- Optimization: Workload balancing and fairness algorithms
- Retry Logic: Automatic retry with different strategies on failure

### Rule Engine
- Pluggable Strategies: Easily add new assignment algorithms
- Validation Pipeline: Multi-stage validation with configurable severity
- Parallel Processing: Concurrent execution for high-performance scenarios
- Configuration-Driven: Runtime behavior modification without code changes

### Entity Management
- Generic Data Model: Flexible attribute-based entity storage
- Type System: Support for participants, targets, and custom entity types
- Metadata Tracking: Version control, audit trails, status management
- CRUD Operations: Full lifecycle management with validation

### Authentication & Authorization
- JWT Authentication: Secure token-based access control
- Role Management: Admin and user role separation
- Session Management: Configurable token expiration and refresh
