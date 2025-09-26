# VividShift Backend

A production-ready Rust backend service for work group distribution with authentication, environment-based configuration, and comprehensive monitoring.

## 🏗️ Architecture

This application has been restructured from a simple CLI tool into a scalable web service with:

- **Clean Architecture**: Modular design with separate concerns
- **Authentication**: JWT-based API authentication with role-based access
- **Environment Configuration**: Dev/Staging/Production environment support
- **Database Integration**: PostgreSQL with connection pooling
- **Monitoring**: Prometheus metrics and Grafana dashboards
- **Containerization**: Docker and Docker Compose for development
- **CI/CD**: GitHub Actions pipeline with automated testing and deployment

## 🚀 Quick Start

### Prerequisites

- [Docker](https://docs.docker.com/get-docker/) and [Docker Compose](https://docs.docker.com/compose/install/)
- [Rust](https://rustup.rs/) (for local development)
- [Git](https://git-scm.com/)

### Setup

1. **Clone the repository:**
   ```bash
   git clone https://github.com/onelrian/VividShift.git
   cd VividShift
   ```

2. **Run the setup script:**
   ```bash
   ./scripts/setup.sh
   ```

3. **Access the services:**
   - **API**: http://localhost:8080
   - **Health Check**: http://localhost:8080/health
   - **Grafana**: http://localhost:3000 (admin/admin)
   - **Prometheus**: http://localhost:9090

## 📡 API Endpoints

### Authentication
- `POST /auth/login` - User login
- `POST /auth/register` - User registration

### Work Groups (Protected)
- `POST /api/work-groups/generate` - Generate work assignments
- `GET /api/work-groups/history` - Get assignment history
- `GET /api/work-groups/assignments` - Get current assignments configuration
- `POST /api/work-groups/assignments` - Update assignments (Admin only)

### Health & Monitoring
- `GET /health` - Health check
- `GET /ready` - Readiness check

### Example Usage

1. **Login:**
   ```bash
   curl -X POST http://localhost:8080/auth/login \
     -H "Content-Type: application/json" \
     -d '{"username": "admin", "password": "password123"}'
   ```

2. **Generate Work Groups:**
   ```bash
   curl -X POST http://localhost:8080/api/work-groups/generate \
     -H "Authorization: Bearer YOUR_JWT_TOKEN" \
     -H "Content-Type: application/json" \
     -d '{
       "names_a": ["Alice", "Bob", "Charlie"],
       "names_b": ["David", "Eve", "Frank"]
     }'
   ```

## 🔧 Configuration

### Environment Variables

Copy `.env.example` to `.env` and configure:

```bash
# Environment
ENVIRONMENT=dev

# Database
VIVIDSHIFT_DATABASE_URL=postgresql://postgres:password@localhost:5432/vividshift_dev

# Authentication
VIVIDSHIFT_AUTH_JWT_SECRET=your-super-secret-jwt-key
VIVIDSHIFT_AUTH_JWT_EXPIRATION=86400

# Server
VIVIDSHIFT_SERVER_HOST=127.0.0.1
VIVIDSHIFT_SERVER_PORT=8080

# Logging
VIVIDSHIFT_LOGGING_LEVEL=debug
VIVIDSHIFT_LOGGING_FILE_ENABLED=true
```

### Configuration Files

Environment-specific configurations are in `backend/config/`:
- `default.toml` - Base configuration
- `dev.toml` - Development overrides
- `staging.toml` - Staging environment
- `prod.toml` - Production environment

## 🏃‍♂️ Development

### Local Development

1. **Start dependencies:**
   ```bash
   docker-compose up -d db redis prometheus grafana
   ```

2. **Run the backend:**
   ```bash
   cd backend
   cargo run
   ```

3. **Run tests:**
   ```bash
   cd backend
   cargo test
   ```

### Docker Development

```bash
docker-compose up --build
```

## 🔒 Security

- JWT tokens for API authentication
- Role-based access control (Admin, User, Viewer)
- Bcrypt password hashing
- Environment-based secret management
- Non-root container execution

### Default Users

- **Admin**: `admin` / `password123`
- **User**: `user` / `password123`

*Change these in production!*

## 📊 Monitoring

### Metrics

- Application metrics via Prometheus
- Custom business metrics for work group generation
- Database connection pool metrics
- HTTP request metrics

### Dashboards

Access Grafana at http://localhost:3000 with default credentials `admin/admin`.

### Logging

- Structured logging with tracing
- Environment-specific log levels
- File and console output
- JSON format for production

## 🚀 Deployment

### CI/CD Pipeline

The GitHub Actions pipeline includes:
- **Testing**: Rust formatting, clippy, and unit tests
- **Building**: Multi-stage Docker builds with caching
- **Deployment**: Automatic deployment to staging/production

### Manual Deployment

1. **Build production image:**
   ```bash
   docker build -t vividshift-backend .
   ```

2. **Deploy with environment variables:**
   ```bash
   docker run -d \
     -p 8080:8080 \
     -e ENVIRONMENT=prod \
     -e VIVIDSHIFT_AUTH_JWT_SECRET=your-production-secret \
     vividshift-backend
   ```

## 📁 Project Structure

```
VividShift/
├── backend/                 # Rust backend service
│   ├── src/
│   │   ├── api/            # HTTP API endpoints
│   │   ├── auth/           # Authentication & authorization
│   │   ├── config/         # Configuration management
│   │   ├── services/       # Business logic services
│   │   └── main.rs         # Application entry point
│   ├── config/             # Environment configurations
│   └── Dockerfile          # Backend container
├── monitoring/             # Monitoring configuration
│   ├── prometheus.yml      # Prometheus config
│   └── grafana/           # Grafana dashboards
├── scripts/               # Setup and utility scripts
├── docker-compose.yml     # Development environment
└── .github/workflows/     # CI/CD pipeline
```

## 🤝 Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Add tests
5. Submit a pull request

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

