FROM rust:1-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY .sqlx ./.sqlx
COPY src ./src
COPY examples ./examples
COPY migrations ./migrations
ENV SQLX_OFFLINE=true
RUN cargo build --release --bin usage-gate --example mock_upstream

FROM debian:bookworm-slim
RUN apt-get update \
 && apt-get install -y --no-install-recommends ca-certificates \
 && rm -rf /var/lib/apt/lists/*
COPY --from=builder /app/target/release/usage-gate /usr/local/bin/usage-gate
COPY --from=builder /app/target/release/examples/mock_upstream /usr/local/bin/mock_upstream
CMD ["/usr/local/bin/usage-gate"]
