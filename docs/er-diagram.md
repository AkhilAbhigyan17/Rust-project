# ER Diagram

```
users ─┬──< organization_members >──┬─ organizations
       │                            │
       │                            ├──< invitations
       │                            ├──< api_keys
       │                            └──< audit_logs
       │
       ├──< sessions
       └──< verification_tokens

roles ──< role_permissions >── permissions
roles ── organization_members
```

See `migrations/20250101000001_init.sql` for the full DDL.
