#!/bin/bash

# VividShift Backend Setup Script

set -e

echo "🚀 Setting up VividShift Backend Development Environment..."

# Check if Docker and Docker Compose are installed
if ! command -v docker &> /dev/null; then
    echo "❌ Docker is not installed. Please install Docker first."
    exit 1
fi

if ! command -v docker-compose &> /dev/null; then
    echo "❌ Docker Compose is not installed. Please install Docker Compose first."
    exit 1
fi

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo "❌ Rust is not installed. Please install Rust first: https://rustup.rs/"
    exit 1
fi

# Create .env file if it doesn't exist
if [ ! -f .env ]; then
    echo "📝 Creating .env file from template..."
    cp .env.example .env
    echo "✅ Created .env file. Please review and update the configuration as needed."
else
    echo "ℹ️ .env file already exists."
fi

# Create logs directory
mkdir -p logs
echo "✅ Created logs directory."

# Create data directories for Docker volumes
mkdir -p data/postgres data/redis
echo "✅ Created data directories."

# Build and start services
echo "🐳 Building and starting Docker services..."
docker-compose up -d --build

# Wait for services to be ready
echo "⏳ Waiting for services to be ready..."
sleep 10

# Check if services are running
if docker-compose ps | grep -q "Up"; then
    echo "✅ Services are running successfully!"
    echo ""
    echo "🌐 Backend API: http://localhost:8080"
    echo "📊 Health Check: http://localhost:8080/health"
    echo "🗄️ PostgreSQL: localhost:5432"
    echo "🔴 Redis: localhost:6379"
    echo "📈 Prometheus: http://localhost:9090"
    echo "📊 Grafana: http://localhost:3000 (admin/admin)"
    echo ""
    echo "🔧 To view logs: docker-compose logs -f"
    echo "🛑 To stop services: docker-compose down"
else
    echo "❌ Some services failed to start. Check logs with: docker-compose logs"
    exit 1
fi
