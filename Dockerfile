# Build stage
FROM rust:latest as builder

WORKDIR /app

# Install dependencies for musl-based static linking
RUN apt update && apt install -y musl-tools && rustup target add x86_64-unknown-linux-musl

# Copy Cargo files first for caching
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs  
RUN cargo build --release --target x86_64-unknown-linux-musl  

# Now copy actual source code
COPY . .

# Build the actual release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Rename binary for clarity
RUN mv target/x86_64-unknown-linux-musl/release/work_group_generator /app/work_distribution

# Runtime stage
FROM alpine:latest

WORKDIR /app

# Copy compiled Rust binary
COPY --from=builder /app/work_distribution /usr/local/bin/work_distribution
RUN chmod +x /usr/local/bin/work_distribution

# Copy data files
COPY file_a.txt file_b.txt /app/

# Run the program
CMD ["/usr/local/bin/work_distribution"]
