FROM rust:latest as builder

WORKDIR /app
COPY . .

# Install musl tools for static linking
RUN apt update && apt install -y musl-tools && rustup target add x86_64-unknown-linux-musl

# Build with musl (statically linked)
RUN cargo build --release --target x86_64-unknown-linux-musl

# Use a minimal base image like Alpine (musl-based)
FROM alpine:latest
WORKDIR /app

COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/work_group_generator /usr/local/bin/work_distribution
COPY names.txt .

CMD ["work_distribution"]
