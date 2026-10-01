FROM rust:1.94-bookworm AS builder
WORKDIR /app
COPY Cargo.toml Cargo.lock ./
COPY src ./src
RUN cargo build --release --locked

FROM debian:bookworm-slim AS runtime
RUN apt-get update \
    && apt-get install -y --no-install-recommends ca-certificates \
    && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /app/target/release/clottis_teams_backend /usr/local/bin/clottis-teams-backend
EXPOSE 8080
USER nobody
ENTRYPOINT ["clottis-teams-backend"]
