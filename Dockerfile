FROM rust:latest as builder

WORKDIR /app

# Copy the project files
COPY . .

RUN rm -f Cargo.lock && cargo build --release

# Use a minimal base image for the final container
FROM debian:buster-slim

COPY --from=builder /app/target/release/work_distribution /usr/local/bin/work_distribution

# Set the working directory
WORKDIR /app

# Copy the names.txt file
COPY names.txt .

CMD ["./lambert_w_function"]