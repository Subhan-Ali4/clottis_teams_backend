# CLOTTIS Teams Backend

Rust API foundation built with Actix Web, SQLx and PostgreSQL.

## Architecture

```text
HTTP route -> executor -> service -> PostgreSQL/external dependency
```

- `routes/`: HTTP transport, extraction and response mapping.
- `executors/`: use-case orchestration between routes and services.
- `services/`: reusable application and dependency logic.
- `structs/`: typed requests, responses, state and domain-safe enums.
- `configs/`: validated environment configuration.
- `utils/`: shared errors and narrow helpers.
- `deploy/`: deployment configuration and scripts.

## Local Setup

```bash
cp .env.example .env
docker compose up -d postgres
cargo run
```

Endpoints:

- `GET /api/v1/health` - process liveness.
- `GET /api/v1/health/ready` - PostgreSQL readiness.

## Verification

```bash
cargo fmt --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-targets --all-features
docker compose config
```
