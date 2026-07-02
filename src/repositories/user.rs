use crate::models::User;
use crate::utils::error::ApiResult;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(pool: &PgPool, email: &str, password_hash: &str, display_name: Option<&str>) -> ApiResult<User> {
    let u = sqlx::query_as::<_, User>(
        r#"INSERT INTO users (email, password_hash, display_name)
           VALUES ($1, $2, $3) RETURNING *"#,
    )
    .bind(email)
    .bind(password_hash)
    .bind(display_name)
    .fetch_one(pool)
    .await?;
    Ok(u)
}

pub async fn find_by_email(pool: &PgPool, email: &str) -> ApiResult<Option<User>> {
    let u = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE email = $1 AND deleted_at IS NULL",
    )
    .bind(email)
    .fetch_optional(pool)
    .await?;
    Ok(u)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<User>> {
    let u = sqlx::query_as::<_, User>(
        "SELECT * FROM users WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id)
    .fetch_optional(pool)
    .await?;
    Ok(u)
}

pub async fn update_password(pool: &PgPool, id: Uuid, password_hash: &str) -> ApiResult<()> {
    sqlx::query("UPDATE users SET password_hash = $1, updated_at = NOW() WHERE id = $2")
        .bind(password_hash)
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}

pub async fn mark_email_verified(pool: &PgPool, id: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE users SET email_verified = TRUE, updated_at = NOW() WHERE id = $1")
        .bind(id)
        .execute(pool)
        .await?;
    Ok(())
}
