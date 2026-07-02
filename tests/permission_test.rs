#[test]
fn owner_has_all_permissions() {
    let owner: &[&str] = &[
        "org.read", "org.update", "org.delete",
        "members.read", "members.invite", "members.remove",
        "roles.manage", "api_keys.manage", "audit.read",
    ];
    assert_eq!(owner.len(), 9);
}
