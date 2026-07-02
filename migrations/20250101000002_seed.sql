-- Seed system permissions
INSERT INTO permissions (code, description) VALUES
    ('org.read', 'Read organization'),
    ('org.update', 'Update organization'),
    ('org.delete', 'Delete organization'),
    ('members.read', 'Read members'),
    ('members.invite', 'Invite members'),
    ('members.remove', 'Remove members'),
    ('roles.manage', 'Manage roles and permissions'),
    ('api_keys.manage', 'Manage API keys'),
    ('audit.read', 'Read audit logs')
ON CONFLICT (code) DO NOTHING;
