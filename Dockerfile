# ---- Stage 1: The Builder ----
# This stage compiles the Rust application.
FROM rust:latest AS builder

# Install dependencies for static linking
WORKDIR /app
RUN apt-get update && apt-get install -y musl-tools libpq-dev pkg-config && rustup target add x86_64-unknown-linux-musl

# Copy the source code and build the application
COPY . .
RUN cargo build --release --target x86_64-unknown-linux-musl


# ---- Stage 2: The Final Image ----
# This stage creates the small, final image for running the application.
FROM debian:bullseye-slim

# Copy the compiled binary from the "builder" stage to a safe location
# that is in the system's PATH.
COPY --from=builder /app/target/x86_64-unknown-linux-musl/release/work_group_generator /usr/local/bin/

# Set the command to run the application.
# Because it's in the PATH, we can just use the binary name.
CMD ["work_group_generator"]