# Getting Started with VividShift

## Overview
This guide will help you get VividShift running locally in under 5 minutes and generate your first assignments.

**Target Audience:** New users, developers, evaluators

## Prerequisites
- Docker and Docker Compose
- Git
- curl (for testing)
- jq (optional, for JSON parsing)
- 4GB RAM minimum

## Quick Setup

### 1. Clone and Setup
```bash
git clone https://github.com/onelrian/vividshift.git
cd vividshift
cp .env.example .env
```

### 2. Start Services
```bash
docker-compose up -d --build
```

This starts all required services:
- VividShift API (port 8080)
- PostgreSQL database (port 5432)
- Redis cache (port 6379)
- Prometheus monitoring (port 9090)
- Grafana dashboard (port 3000)

### 3. Verify Installation
```bash
curl http://localhost:8080/health
# Expected: {"service":"vividshift-backend","status":"healthy"}

curl http://localhost:8080/ready
# Expected: {"status":"ready"}
```

## First Steps

### 1. Register a User
```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","email":"admin@example.com","password":"password123"}'
```

Expected response:
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

### 2. Login and Get Token
```bash
TOKEN=$(curl -s -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password123"}' | jq -r '.token')

echo "Your token: $TOKEN"
```

### 3. Generate Your First Assignment
```bash
# Generate work group assignments using the legacy endpoint
curl -X POST http://localhost:8080/api/work-groups/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{"names_a":["Alice","Bob","Charlie"],"names_b":["David","Eve","Frank"]}'
```

Expected response:
```json
{
  "assignments": {
    "Parlor": ["Alice", "Bob", "Charlie", "David", "Eve"],
    "Frontyard": ["Frank", "Alice", "Bob"],
    "Tank": ["Charlie", "David"],
    "Toilet A": ["Eve", "Frank"],
    "Toilet B": ["Alice", "Bob", "Charlie", "David"],
    "Bin": ["Eve"]
  },
  "generation_info": {
    "attempt_number": 1,
    "timestamp": "2024-01-01T12:00:00Z",
    "total_people": 6,
    "total_assignments": 17
  }
}
```

### 4. View Assignment History
```bash
curl -H "Authorization: Bearer $TOKEN" \
  http://localhost:8080/api/work-groups/history
```

## Understanding the Results

The assignment system automatically:
1. **Distributes participants** across available work areas
2. **Balances workload** to ensure fair distribution
3. **Tracks history** for future assignment optimization
4. **Validates capacity** to ensure all areas are properly staffed

## Working with Participants and Targets

### Create Custom Participants
```bash
# Create a participant with specific skills
curl -X POST http://localhost:8080/api/entities/participant \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Alice Johnson",
    "group": "TeamA",
    "availability": true,
    "skills": ["cleaning", "organizing", "leadership"]
  }'
```

### Create Assignment Targets
```bash
# Create a custom assignment target
curl -X POST http://localhost:8080/api/entities/assignment_target \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "name": "Conference Room",
    "required_count": 3,
    "priority": 1
  }'
```

### Generate Advanced Assignments
```bash
# Use the advanced assignment endpoint with strategy selection
curl -X POST http://localhost:8080/api/assignments/generate \
  -H "Authorization: Bearer $TOKEN" \
  -H "Content-Type: application/json" \
  -d '{
    "strategy": "balanced_rotation",
    "participants": [],
    "targets": [],
    "parameters": {
      "rotation_weight": 0.8,
      "balance_weight": 0.2
    }
  }'
```

## Monitoring and Observability

### Access Monitoring Dashboards
- **Application Health**: http://localhost:8080/health
- **Prometheus Metrics**: http://localhost:9090
- **Grafana Dashboard**: http://localhost:3000 (admin/admin)

### View Application Logs
```bash
# View all service logs
docker-compose logs -f

# View just the application logs
docker-compose logs -f app

# View database logs
docker-compose logs -f db
```

## Common Use Cases

### 1. Work Group Assignments
Perfect for assigning people to cleaning tasks, work areas, or rotating duties.

### 2. Team Scheduling
Assign team members to shifts, projects, or responsibilities.

### 3. Resource Allocation
Distribute resources, equipment, or facilities among participants.

### 4. Event Management
Assign volunteers to event roles, stations, or time slots.

## Configuration Customization

### Environment Variables
Edit your `.env` file to customize:

```bash
# Server configuration
VIVIDSHIFT_SERVER_HOST=0.0.0.0
VIVIDSHIFT_SERVER_PORT=8080

# Database configuration
VIVIDSHIFT_DATABASE_URL=postgresql://postgres:password@localhost:5432/vividshift_dev

# Authentication
VIVIDSHIFT_AUTH_JWT_SECRET=your-secret-key
VIVIDSHIFT_AUTH_JWT_EXPIRATION=86400

# Logging
VIVIDSHIFT_LOGGING_LEVEL=info
```

### Domain Configuration
Create custom domains by adding configuration files:

```bash
# Create a custom domain configuration
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
```

## Troubleshooting Quick Fixes

### Service Won't Start
```bash
# Check service status
docker-compose ps

# View error logs
docker-compose logs app

# Restart services
docker-compose restart
```

### Database Connection Issues
```bash
# Check database health
docker-compose exec db pg_isready -U postgres

# Reset database
docker-compose down -v
docker-compose up -d
```

### Authentication Problems
```bash
# Check if user exists
curl -X POST http://localhost:8080/auth/login \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","password":"password123"}'

# Register new user if needed
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"newuser","email":"new@example.com","password":"password123"}'
```

## What's Next?

Now that you have VividShift running:

1. **Explore the API** - [API Reference](API_REFERENCE.md) for all available endpoints
2. **Customize Configuration** - [Configuration Guide](CONFIGURATION.md) for advanced setup
3. **Understand the Architecture** - [Architecture Guide](ARCHITECTURE.md) for system design
4. **Deploy to Production** - [Deployment Guide](DEPLOYMENT.md) for production setup
5. **Contribute** - [Development Guide](DEVELOPMENT.md) for local development

## Need Help?

- **Common Issues**: Check [Troubleshooting](TROUBLESHOOTING.md)
- **API Questions**: Review [API Reference](API_REFERENCE.md)
- **Configuration**: See [Configuration Guide](CONFIGURATION.md)
- **Architecture**: Read [Architecture Overview](ARCHITECTURE.md)
