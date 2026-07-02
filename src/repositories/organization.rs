use crate::models::{Organization, OrganizationMember};
use crate::utils::error::ApiResult;
use sqlx::PgPool;
use uuid::Uuid;

pub async fn create(pool: &PgPool, name: &str, slug: &str, owner_id: Uuid) -> ApiResult<Organization> {
    let o = sqlx::query_as::<_, Organization>(
        r#"INSERT INTO organizations (name, slug, owner_id) VALUES ($1,$2,$3) RETURNING *"#,
    )
    .bind(name).bind(slug).bind(owner_id)
    .fetch_one(pool).await?;
    Ok(o)
}

pub async fn find_by_id(pool: &PgPool, id: Uuid) -> ApiResult<Option<Organization>> {
    let o = sqlx::query_as::<_, Organization>(
        "SELECT * FROM organizations WHERE id = $1 AND deleted_at IS NULL",
    )
    .bind(id).fetch_optional(pool).await?;
    Ok(o)
}

pub async fn list_for_user(pool: &PgPool, user_id: Uuid) -> ApiResult<Vec<Organization>> {
    let list = sqlx::query_as::<_, Organization>(
        r#"SELECT o.* FROM organizations o
           JOIN organization_members m ON m.organization_id = o.id
           WHERE m.user_id = $1 AND o.deleted_at IS NULL
           ORDER BY o.created_at DESC"#,
    )
    .bind(user_id).fetch_all(pool).await?;
    Ok(list)
}

pub async fn add_member(pool: &PgPool, org_id: Uuid, user_id: Uuid, role_id: Uuid) -> ApiResult<OrganizationMember> {
    let m = sqlx::query_as::<_, OrganizationMember>(
        r#"INSERT INTO organization_members (organization_id, user_id, role_id)
           VALUES ($1,$2,$3) RETURNING *"#,
    )
    .bind(org_id).bind(user_id).bind(role_id)
    .fetch_one(pool).await?;
    Ok(m)
}

pub async fn remove_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> ApiResult<()> {
    sqlx::query("DELETE FROM organization_members WHERE organization_id = $1 AND user_id = $2")
        .bind(org_id).bind(user_id).execute(pool).await?;
    Ok(())
}

pub async fn list_members(pool: &PgPool, org_id: Uuid) -> ApiResult<Vec<OrganizationMember>> {
    let list = sqlx::query_as::<_, OrganizationMember>(
        "SELECT * FROM organization_members WHERE organization_id = $1 ORDER BY created_at",
    ).bind(org_id).fetch_all(pool).await?;
    Ok(list)
}

pub async fn transfer_ownership(pool: &PgPool, org_id: Uuid, new_owner: Uuid) -> ApiResult<()> {
    sqlx::query("UPDATE organizations SET owner_id = $1, updated_at = NOW() WHERE id = $2")
        .bind(new_owner).bind(org_id).execute(pool).await?;
    Ok(())
}

pub async fn is_member(pool: &PgPool, org_id: Uuid, user_id: Uuid) -> ApiResult<bool> {
    let r: (i64,) = sqlx::query_as(
        "SELECT COUNT(*) FROM organization_members WHERE organization_id = $1 AND user_id = $2",
    ).bind(org_id).bind(user_id).fetch_one(pool).await?;
    Ok(r.0 > 0)
}
