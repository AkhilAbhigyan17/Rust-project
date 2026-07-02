# Testing

Unit tests run with `cargo test`. Integration tests are marked `#[ignore]` and require a live Postgres:

```bash
docker compose up -d postgres
export DATABASE_URL=postgres://iam:iam_secret@localhost:5432/iam
cargo test -- --ignored
```
