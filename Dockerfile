# Stage 1: Build
FROM rust:1.82-bookworm AS builder

WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src/ src/
COPY migrations/ migrations/

RUN cargo build --release --bin eas-weather-rs --bin eas-migrate

# Stage 2: Runtime
FROM debian:bookworm-slim

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

RUN groupadd -r eas && useradd -r -g eas eas

WORKDIR /app

COPY --from=builder /app/target/release/eas-weather-rs /app/
COPY --from=builder /app/target/release/eas-migrate /app/
COPY --from=builder /app/migrations /app/migrations
COPY --from=builder /app/config /app/config

RUN chown -R eas:eas /app

USER eas

EXPOSE 8080

# Default: run the server. Override with docker run ... /app/eas-migrate
ENTRYPOINT ["/app/eas-weather-rs"]
