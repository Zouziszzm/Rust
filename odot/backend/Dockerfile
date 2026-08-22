# syntax=docker/dockerfile:1

FROM rust:1.85-bookworm AS builder
WORKDIR /app

COPY Cargo.toml Cargo.lock ./
COPY migrations ./migrations
COPY src ./src

RUN cargo build --release

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /app
COPY --from=builder /app/target/release/odot /usr/local/bin/odot
COPY migrations ./migrations

ENV HOST=0.0.0.0
ENV PORT=8080

EXPOSE 8080
USER nobody
ENTRYPOINT ["/usr/local/bin/odot"]
