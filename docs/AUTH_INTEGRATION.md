# Native auth integration (review before deployment)

The auth-slice PR #2 is a standalone Express/SQLite demo. This feature branch implements corresponding routes in the existing Rust/Actix backend and PostgreSQL.

## Endpoints
- POST /api/v1/auth/login {email,password}; sets HttpOnly session cookie.
- POST /api/v1/auth/logout; deletes session.
- GET /api/v1/auth/me; authenticated profile.
- POST /api/v1/auth/change-password {currentPassword,newPassword}; revokes other sessions.
- GET /api/v1/users; owner/admin list.
- POST /api/v1/users {name,email,role}; owner/admin create account; returns initial password once.
- POST /api/v1/users/{id}/reset-password; owner/admin reset, invalidates sessions.
- PATCH /api/v1/users/{id}/status {isActive}; owner/admin activate/deactivate.

Permissions: Admin cannot manage Owners, and non-Owners cannot create Owners. Self-reset and self-deactivation are refused. Frontend sends X-Requested-With for writes; server checks Origin when present and rejects missing custom headers.

## Setup (not automatic yet)
1. Review the migration and pgcrypto availability with the database administrator.
2. Back up production DB. Apply SQLx migrations to a nonproduction PostgreSQL first.
3. Run scripts/bootstrap_owner.sql from a private interactive psql session in an isolated environment.
4. Configure a same-origin reverse proxy for /api/v1 and secure cookies over HTTPS.
5. Do not deploy to production until CI, end-to-end auth tests, rate limiting, CSRF/origin protections, session expiry handling, and rollback strategy have been reviewed.

No production migration or deployment is performed by this PR.
