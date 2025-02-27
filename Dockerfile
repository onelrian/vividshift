FROM rust:latest as builder

WORKDIR /app
COPY . .
RUN cargo build --release

# Minimal runtime image
FROM debian:buster-slim
WORKDIR /app

COPY --from=builder /app/target/release/work_group_generator /usr/local/bin/work_distribution
COPY names.txt .

CMD ["work_distribution"]
