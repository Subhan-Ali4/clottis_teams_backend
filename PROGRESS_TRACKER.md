# Progress Tracker

## Current Phase

Phase 0 - Repository foundation.

## Current Task

Create and verify the base Actix Web + SQLx + PostgreSQL structure.

## Completed

- Initial module layout.
- Typed configuration, responses and errors.
- Liveness and readiness health endpoints.
- Docker, Compose, environment example and CI baseline.

## Blockers / Decisions Required

- Authentication and authorization schema is intentionally not defined in this phase.

## Tests

- `cargo fmt --check` - passed.
- `cargo check` - passed.
- `cargo clippy --all-targets --all-features -- -D warnings` - passed.
- `cargo test --all-targets --all-features` - passed (2 tests).
- `docker compose config --quiet` - passed.
- Runtime `GET /api/v1/health` - passed.
- Runtime `GET /api/v1/health/ready` with PostgreSQL - passed.

## Exact Next Action

Review and approve the authentication contract before creating its schema or endpoints.
