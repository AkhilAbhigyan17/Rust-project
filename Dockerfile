# ---- Builder ----
FROM rust:1.82-slim AS builder
WORKDIR /app
RUN apt-get update && apt-get install -y pkg-config libssl-dev ca-certificates && rm -rf /var/lib/apt/lists/*
COPY Cargo.toml ./
COPY src ./src
COPY migrations ./migrations
RUN cargo build --release

# ---- Runtime ----
FROM debian:bookworm-slim
WORKDIR /app
RUN apt-get update && apt-get install -y ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/iam-platform /usr/local/bin/iam-platform
COPY --from=builder /app/migrations ./migrations
EXPOSE 8080
ENV RUST_LOG=info
CMD ["iam-platform"]
