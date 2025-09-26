# ---- Stage 1: The Builder ----
FROM rust:latest AS builder

# Install dependencies for static linking
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools && rustup target add x86_64-unknown-linux-musl

# Copy the backend source code and build
COPY backend/ .
RUN cargo build --release --target x86_64-unknown-linux-musl

# ---- Stage 2: The Final Image ----
FROM debian:bullseye-slim

# Install CA certificates for HTTPS requests
RUN apt-get update && apt-get install -y ca-certificates && rm -rf /var/lib/apt/lists/*

# Create app user
RUN useradd -r -s /bin/false appuser

# Create necessary directories
RUN mkdir -p /app/logs && chown appuser:appuser /app/logs

# Copy the compiled binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/vividshift_backend /usr/local/bin/

# Copy configuration files
COPY backend/config/ /app/config/

# Set working directory
WORKDIR /app

# Switch to non-root user
USER appuser

# Expose port
EXPOSE 8080

# Health check
HEALTHCHECK --interval=30s --timeout=3s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:8080/health || exit 1

# Set the command to run the application
CMD ["vividshift_backend"]