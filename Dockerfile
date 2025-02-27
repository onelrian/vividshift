# Build stage
FROM rust:latest as builder

WORKDIR /app

# Copy only dependencies first (for caching)
COPY Cargo.toml Cargo.lock ./
RUN mkdir src && echo "fn main() {}" > src/main.rs  # Dummy main file to avoid copying src prematurely
RUN cargo build --release --target x86_64-unknown-linux-musl || true
RUN rm -rf src  # Remove dummy src

# Now copy actual source code
COPY . .

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools && rustup target add x86_64-unknown-linux-musl

# Build the actual release binary
RUN cargo build --release --target x86_64-unknown-linux-musl

# Runtime stage
FROM alpine:latest
WORKDIR /app

# Copy compiled Rust binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/work_group_generator /usr/local/bin/work_distribution
RUN chmod +x /usr/local/bin/work_distribution

# Copy data files
COPY file_a.txt file_b.txt /app/

# Run the program
CMD ["/usr/local/bin/work_distribution"]
