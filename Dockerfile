# Build Stage
FROM rust:latest as builder

WORKDIR /app
COPY . .

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools && rustup target add x86_64-unknown-linux-musl

# Clean previous builds to prevent cache issues
RUN cargo clean

# Build with musl (statically linked)
RUN cargo build --release --target x86_64-unknown-linux-musl

# Minimal runtime image
FROM alpine:latest

WORKDIR /app

# Copy the compiled Rust binary
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/work_group_generator /usr/local/bin/work_distribution

# Ensure the binary is executable
RUN chmod +x /usr/local/bin/work_distribution

# Copy the required names file (ensure this is always updated)
COPY names.txt /app/names.txt

# Run the program
CMD ["/usr/local/bin/work_distribution"]
