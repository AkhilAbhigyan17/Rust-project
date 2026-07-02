# Architecture

```
┌────────────┐   HTTP    ┌───────────────┐   fn calls   ┌────────────┐   SQL   ┌────────────┐
│  Client    │ ────────► │   Handlers    │ ───────────► │  Services  │ ──────► │Repositories│ ─► Postgres
└────────────┘           └───────────────┘              └────────────┘         └────────────┘
                              ▲
                              │ middleware
                              │ (auth, request-id, rate-limit, CORS, tracing)
```

## Layer responsibilities

- **Handlers** – parse HTTP, validate DTOs, call services, format responses.
- **Services** – transactional business logic (register + verification token, invite + audit, etc.).
- **Repositories** – one module per aggregate, only place that touches SQL.
- **Auth** – JWT issuance & extractor. Refresh tokens live in `sessions` with SHA-256 hashes.
- **Permissions** – `require_permission(pool, user, org, code)` reads role→permission joins.

## Refresh-token rotation

On `POST /auth/refresh`, the old session row is revoked and a new one inserted with `rotated_from = old.id`. Re-use of a revoked refresh token yields `401`.

## Multi-tenancy

Every organization has three system roles seeded on creation (`owner`, `admin`, `member`) mapped to permission codes from the shared catalog. Custom roles are org-scoped.
