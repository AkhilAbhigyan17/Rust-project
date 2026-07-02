use crate::database::DbPool;
use crate::domain::{ADMIN_PERMISSIONS, MEMBER_PERMISSIONS, OWNER_PERMISSIONS, ROLE_ADMIN, ROLE_MEMBER, ROLE_OWNER};
use crate::models::{Organization, Invitation};
use crate::repositories::{invitation as inv_repo, organization as org_repo, role as role_repo, user as user_repo};
use crate::utils::error::{ApiError, ApiResult};
use crate::utils::token::{hash_token, random_token};
use chrono::{Duration, Utc};
use uuid::Uuid;

pub async fn create_organization(pool: &DbPool, owner_id: Uuid, name: &str, slug: &str) -> ApiResult<Organization> {
    let org = org_repo::create(pool, name, slug, owner_id).await?;
    // Bootstrap roles for this organization.
    let owner_role = role_repo::create(pool, Some(org.id), ROLE_OWNER, Some("Full access"), true).await?;
    let admin_role = role_repo::create(pool, Some(org.id), ROLE_ADMIN, Some("Administrator"), true).await?;
    let member_role = role_repo::create(pool, Some(org.id), ROLE_MEMBER, Some("Standard member"), true).await?;
    role_repo::attach_permissions_by_code(pool, owner_role.id, &OWNER_PERMISSIONS.iter().map(|s| s.to_string()).collect::<Vec<_>>()).await?;
    role_repo::attach_permissions_by_code(pool, admin_role.id, &ADMIN_PERMISSIONS.iter().map(|s| s.to_string()).collect::<Vec<_>>()).await?;
    role_repo::attach_permissions_by_code(pool, member_role.id, &MEMBER_PERMISSIONS.iter().map(|s| s.to_string()).collect::<Vec<_>>()).await?;
    org_repo::add_member(pool, org.id, owner_id, owner_role.id).await?;
    Ok(org)
}

pub async fn invite(pool: &DbPool, org_id: Uuid, email: &str, role_id: Uuid, invited_by: Uuid) -> ApiResult<(Invitation, String)> {
    let token = random_token(32);
    let hash = hash_token(&token);
    let exp = Utc::now() + Duration::days(7);
    let inv = inv_repo::create(pool, org_id, email, role_id, &hash, invited_by, exp).await?;
    Ok((inv, token))
}

pub async fn accept_invitation(pool: &DbPool, user_id: Uuid, token: &str) -> ApiResult<Uuid> {
    let hash = hash_token(token);
    let inv = inv_repo::find_valid(pool, &hash).await?.ok_or(ApiError::BadRequest("invalid invitation".into()))?;
    let user = user_repo::find_by_id(pool, user_id).await?.ok_or(ApiError::NotFound)?;
    if user.email.to_lowercase() != inv.email.to_lowercase() {
        return Err(ApiError::Forbidden);
    }
    org_repo::add_member(pool, inv.organization_id, user_id, inv.role_id).await?;
    inv_repo::mark_accepted(pool, inv.id).await?;
    Ok(inv.organization_id)
}

pub async fn transfer_ownership(pool: &DbPool, org_id: Uuid, current_owner: Uuid, new_owner: Uuid) -> ApiResult<()> {
    let org = org_repo::find_by_id(pool, org_id).await?.ok_or(ApiError::NotFound)?;
    if org.owner_id != current_owner {
        return Err(ApiError::Forbidden);
    }
    if !org_repo::is_member(pool, org_id, new_owner).await? {
        return Err(ApiError::BadRequest("new owner must be a member".into()));
    }
    org_repo::transfer_ownership(pool, org_id, new_owner).await?;
    Ok(())
}
