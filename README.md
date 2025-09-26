# VividShift

A production-ready assignment engine for automated participant-to-target assignment generation with configurable rules and strategies.

## Overview

VividShift is a domain-agnostic Generic Assignment Engine built with Rust that provides high-performance assignment generation through pluggable rule engines. The system supports multiple assignment strategies, validation rules, and distribution algorithms.

## Quick Start

### Prerequisites

- Docker and Docker Compose
- Git

### Installation

```bash
git clone https://github.com/onelrian/vividshift.git
cd vividshift
git checkout feature/fullstack-app
cp .env.example .env
docker-compose up -d --build
```

### Verification

```bash
curl http://localhost:8080/health
```

Expected response: `{"service":"vividshift-backend","status":"healthy"}`

## Usage

### Authentication

Register a user and obtain a JWT token:

```bash
curl -X POST http://localhost:8080/auth/register \
  -H "Content-Type: application/json" \
  -d '{"username":"admin","email":"admin@example.com","password":"password123"}'
```

### Generate Assignments

```bash
curl -X POST http://localhost:8080/api/work-groups/generate \
  -H "Authorization: Bearer <token>" \
  -H "Content-Type: application/json" \
  -d '{"names_a":["Alice","Bob"],"names_b":["Charlie","David"]}'
```

## Configuration

The application uses hierarchical configuration loading:

1. Default configuration (`backend/config/default.toml`)
2. Environment-specific overrides (`backend/config/{dev,staging,prod}.toml`)
3. Environment variables (prefixed with `VIVIDSHIFT_`)

### Required Environment Variables

- `VIVIDSHIFT_DATABASE_URL` - PostgreSQL connection string
- `VIVIDSHIFT_AUTH_JWT_SECRET` - JWT signing secret

### Optional Environment Variables

- `VIVIDSHIFT_SERVER_HOST` - Server bind address (default: 0.0.0.0)
- `VIVIDSHIFT_SERVER_PORT` - Server port (default: 8080)
- `ENVIRONMENT` - Deployment environment (dev/staging/prod)

## API Reference

### Core Endpoints

| Method | Endpoint | Description |
|--------|----------|-------------|
| GET | `/health` | Service health check |
| GET | `/ready` | Service readiness check |
| POST | `/auth/register` | User registration |
| POST | `/auth/login` | User authentication |
| POST | `/api/work-groups/generate` | Generate assignments |
| GET | `/api/work-groups/history` | Assignment history |

### Authentication

All protected endpoints require JWT authentication:

```bash
Authorization: Bearer <jwt-token>
```

## Development

### Local Development

```bash
cd backend
cargo build
cargo test
cargo run
```

### Docker Development

```bash
docker-compose up -d --build
docker-compose logs -f app
```

## Production Deployment

### Docker

```bash
docker build -t vividshift ./backend
docker run -d -p 8080:8080 \
  -e ENVIRONMENT=prod \
  -e VIVIDSHIFT_AUTH_JWT_SECRET=<secret> \
  vividshift
```

### Environment Configuration

Set the following environment variables for production:

- `ENVIRONMENT=prod`
- `VIVIDSHIFT_AUTH_JWT_SECRET` - Strong random secret
- `VIVIDSHIFT_DATABASE_URL` - Production database URL

## Monitoring

Access monitoring interfaces:

- **Application**: http://localhost:8080
- **Prometheus**: http://localhost:9090
- **Grafana**: http://localhost:3000 (admin/admin)

## Documentation

- [API Documentation](docs/API_DOCUMENTATION.md)
- [Deployment Guide](docs/DEPLOYMENT.md)
- [User Manual](docs/USER_MANUAL.md)

## License

MIT License - see [LICENSE](LICENSE) file for details.
