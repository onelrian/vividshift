# Start from Rust base image
FROM rust:latest

WORKDIR /app

# Install dependencies for static linking
RUN apt update && apt install -y musl-tools && rustup target add x86_64-unknown-linux-musl

# Copy source code
COPY . .

# Build the Rust application
RUN cargo build --release --target x86_64-unknown-linux-musl

# Run the binary
CMD ["./target/x86_64-unknown-linux-musl/release/work_group_generator"]
