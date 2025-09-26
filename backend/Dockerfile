# Development Dockerfile for backend
FROM rust:latest

# Install development tools
RUN apt-get update && apt-get install -y \
    curl \
    postgresql-client \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /app

# Copy dependency files first for better caching
COPY Cargo.toml Cargo.lock ./

# Create a dummy main.rs to build dependencies
RUN mkdir src && echo "fn main() {}" > src/main.rs
RUN cargo build --release
RUN rm -rf src

# Copy source code
COPY . .

# Build the application
RUN cargo build --release

# Expose port
EXPOSE 8080

# Create logs directory
RUN mkdir -p logs

# Run the application
CMD ["cargo", "run", "--release"]
