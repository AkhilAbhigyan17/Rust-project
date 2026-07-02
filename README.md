# IAM Platform

A production-ready **Identity & Access Management** backend built with **Rust**, **Axum**, **SQLx**, **PostgreSQL**, **JWT**, and **Argon2**.

## Features

- **Authentication** – register, login, logout, JWT access tokens, refresh-token rotation, device sessions, email verification, forgot / reset / change password.
- **Multi-tenancy** – organizations with owners, invitations, ownership transfer, member removal, organization switching.
- **RBAC** – system roles (owner / admin / member), custom roles, permission catalog, permission evaluation engine, `require_permission` guard.
- **API Keys** – generate, rotate, revoke, per-key scopes and expiry.
- **Audit Logs** – every sensitive mutation logged with user, org, IP, user-agent, previous & new values.
- **Security** – Argon2id password hashing, refresh-token rotation & revocation, IP rate limiting, input validation (`validator`), parameterized SQL (SQLx), CORS, request-ID propagation, generic error responses to prevent enumeration.
- **Observability** – structured `tracing` logs, request IDs, health check.
- **Docs** – interactive **Swagger UI** at `/swagger-ui`, OpenAPI spec at `/api-doc/openapi.json`.
- **Docker** – multi-stage `Dockerfile` and `docker-compose.yml`.
- **CI** – GitHub Actions workflow for `fmt`, `clippy`, and `build`.

## Architecture

Clean architecture with clear separation of concerns:

```
handlers  ──►  services  ──►  repositories  ──►  database
   ▲              ▲                 ▲
   │              │                 │
  dto           domain             models
```

- `handlers/` – HTTP layer (Axum), request parsing, response formatting.
- `services/` – business logic (transactions, orchestration).
- `repositories/` – SQL access, one module per aggregate.
- `models/` – row structs mapped from Postgres.
- `dto/` – request/response schemas (with `utoipa` for OpenAPI).
- `middleware/` – request-id, rate limiting, client info extractor.
- `auth/` – JWT issue/decode, `AuthUser` extractor.
- `permissions/` – permission evaluation engine.
- `config/` – env-driven `Settings`.

## Quick start

### 1. With Docker Compose (recommended)

```bash
docker compose up --build
```

The API is available at <http://localhost:8080>. Swagger UI: <http://localhost:8080/swagger-ui>.

### 2. Local development

```bash
cp .env.example .env
docker compose up -d postgres
cargo run
```

Migrations run automatically on startup.

## Configuration

| Variable | Default | Description |
|---|---|---|
| `DATABASE_URL` | — | Postgres connection URL (required). |
| `JWT_SECRET` | — | Secret used to sign access tokens (required, ≥32 chars). |
| `JWT_ACCESS_TTL_SECS` | `900` | Access-token lifetime. |
| `JWT_REFRESH_TTL_SECS` | `1209600` | Refresh-token lifetime (14 days). |
| `BIND_ADDR` | `0.0.0.0:8080` | HTTP bind address. |
| `RUST_LOG` | `info` | `tracing` filter. |
| `CORS_ALLOWED_ORIGINS` | `*` | Comma-separated allow list. |

## API overview

Base path: `/api/v1`.

| Method | Path | Description |
|---|---|---|
| `POST` | `/auth/register` | Create an account. |
| `POST` | `/auth/login` | Password login → tokens. |
| `POST` | `/auth/refresh` | Rotate refresh token. |
| `POST` | `/auth/logout` | Revoke a session. |
| `POST` | `/auth/verify-email` | Verify an email. |
| `POST` | `/auth/forgot-password` | Request reset link. |
| `POST` | `/auth/reset-password` | Reset using token. |
| `POST` | `/auth/change-password` | Change while authenticated. |
| `GET`  | `/auth/me` | Current user. |
| `GET`  | `/auth/sessions` | List sessions. |
| `GET`  | `/organizations` | List my orgs. |
| `POST` | `/organizations` | Create org. |
| `POST` | `/organizations/{id}/switch` | Get access token scoped to org. |
| `GET`  | `/organizations/{id}/members` | List members. |
| `POST` | `/organizations/{id}/invitations` | Invite member. |
| `POST` | `/organizations/accept-invitation` | Accept invite. |
| `DELETE` | `/organizations/{id}/members/{user_id}` | Remove member. |
| `POST` | `/organizations/{id}/transfer-ownership` | Transfer ownership. |
| `GET`  | `/rbac/permissions` | Permission catalog. |
| `GET`  | `/rbac/{org_id}/roles` | List roles. |
| `POST` | `/rbac/{org_id}/roles` | Create custom role. |
| `GET`  | `/organizations/{id}/api-keys` | List API keys. |
| `POST` | `/organizations/{id}/api-keys` | Create key (returned **once**). |
| `POST` | `/organizations/{id}/api-keys/{key_id}/rotate` | Rotate. |
| `DELETE` | `/organizations/{id}/api-keys/{key_id}` | Revoke. |
| `GET`  | `/organizations/{id}/audit-logs` | Audit trail. |
| `GET`  | `/health` | Liveness. |

See [`docs/`](./docs) for the ER diagram, architecture diagram and expanded API docs.

## Testing

```bash
cargo test           # unit + placeholder integration tests
cargo test -- --ignored   # full integration tests (requires Postgres)
```

## License

MIT
