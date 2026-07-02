use crate::models::{Role, Permission};
use crate::utils::error::ApiResult;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(pool: &PgPool, org_id: Option<Uuid>, name: &str, description: Option<&str>, is_system: bool) -> ApiResult<Role> {
    let r = sqlx::query_as::<_, Role>(
        r#"INSERT INTO roles (organization_id, name, description, is_system)
           VALUES ($1,$2,$3,$4) RETURNING *"#,
    )
    .bind(org_id).bind(name).bind(description).bind(is_system)
    .fetch_one(pool).await?;
    Ok(r)
}

pub async fn find_by_name(pool: &PgPool, org_id: Uuid, name: &str) -> ApiResult<Option<Role>> {
    let r = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE organization_id = $1 AND name = $2",
    ).bind(org_id).bind(name).fetch_optional(pool).await?;
    Ok(r)
}

pub async fn list_for_org(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<Role>> {
    let list = sqlx::query_as::<_, Role>(
        "SELECT * FROM roles WHERE organization_id = $1 OR is_system = TRUE ORDER BY is_system DESC, name",
    ).bind(org_id).fetch_all(pool).await?;
    Ok(list)
}

pub async fn attach_permissions_by_code(pool: &PgPool, role_id: Uuid, codes: &[String]) -> ApiResult<()> {
    for code in codes {
        sqlx::query(
            r#"INSERT INTO role_permissions (role_id, permission_id)
               SELECT $1, id FROM permissions WHERE code = $2
               ON CONFLICT DO NOTHING"#,
        ).bind(role_id).bind(code).execute(pool).await?;
    }
    Ok(())
}

pub async fn list_permissions(pool: &PgPool) -> ApiResult<Vec<Permission>> {
    let list = sqlx::query_as::<_, Permission>("SELECT * FROM permissions ORDER BY code")
        .fetch_all(pool).await?;
    Ok(list)
}
