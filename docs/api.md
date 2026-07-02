# API Documentation

Interactive documentation ships with the server at:

- Swagger UI: `/swagger-ui`
- OpenAPI JSON: `/api-doc/openapi.json`

## Authentication

Bearer JWT in the `Authorization` header:

```
Authorization: Bearer <access_token>
```

Access tokens are short-lived (default 15 min). Refresh with `POST /api/v1/auth/refresh`.

## Errors

```json
{ "error": { "code": "unauthorized", "message": "unauthorized" } }
```

Codes: `bad_request`, `unauthorized`, `forbidden`, `not_found`, `conflict`, `validation_error`, `rate_limited`, `internal_error`.
