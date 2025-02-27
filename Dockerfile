# Use the official Rust image as the base image
FROM rust:latest as builder

# Set the working directory
WORKDIR /app

# Copy the source code into the container
COPY . .

# Build the application
RUN cargo build --release

# Use a newer base image for the final stage
FROM debian:bookworm-slim 

# Install necessary libraries (if your binary depends on them)
RUN apt-get update && apt-get install -y libssl-dev && rm -rf /var/lib/apt/lists/*

# Copy the compiled binary from the builder stage
COPY --from=builder /app/target/release/work_distribution /usr/local/bin/work_distribution

# Set the working directory
WORKDIR /app

# Copy the names.txt file
COPY names.txt .

# Run the application
CMD ["work_distribution"]