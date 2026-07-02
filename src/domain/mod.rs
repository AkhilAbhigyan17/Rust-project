//! Cross-cutting domain constants (system roles etc.).
pub const ROLE_OWNER: &str = "owner";
pub const ROLE_ADMIN: &str = "admin";
pub const ROLE_MEMBER: &str = "member";

pub const OWNER_PERMISSIONS: &[&str] = &[
    "org.read", "org.update", "org.delete",
    "members.read", "members.invite", "members.remove",
    "roles.manage", "api_keys.manage", "audit.read",
];
pub const ADMIN_PERMISSIONS: &[&str] = &[
    "org.read", "org.update",
    "members.read", "members.invite", "members.remove",
    "roles.manage", "api_keys.manage", "audit.read",
];
pub const MEMBER_PERMISSIONS: &[&str] = &["org.read", "members.read"];
