# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the source code into the container
COPY . .

# Debug: List files in /app to verify Cargo.toml is copied
RUN ls -la /app

# Build the application
RUN cargo build --release

# Use a smaller base image for the final stage
FROM debian:bullseye-slim

# Install necessary libraries (if your binary depends on them)
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/work_group_generator /usr/local/bin/work_distribution

# Set the working directory
WORKDIR /app

# Copy the names.txt file
COPY names.txt .

# Ensure the binary has the correct permissions
RUN chmod +x /usr/local/bin/work_distribution

# Run the application
CMD ["work_distribution"]
