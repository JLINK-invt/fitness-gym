-- Fase 4.1: replace legacy cashier/accounting roles with the operational
-- receptionist/advisor/trainer model and add per-user permission overrides.

UPDATE users
SET status = 'disabled',
    updated_at = strftime('%Y-%m-%dT%H:%M:%fZ', 'now')
WHERE id IN (
    SELECT user_id FROM staff_memberships WHERE role = 'accounting'
);

CREATE TABLE staff_memberships_v20 (
    id TEXT PRIMARY KEY NOT NULL,
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    user_id TEXT NOT NULL,
    role TEXT NOT NULL CHECK (role IN ('owner', 'manager', 'receptionist', 'advisor', 'trainer')),
    status TEXT NOT NULL DEFAULT 'active' CHECK (status IN ('active', 'revoked')),
    created_at TEXT NOT NULL,
    UNIQUE (organization_id, user_id)
) STRICT;

INSERT INTO staff_memberships_v20
    (id, organization_id, user_id, role, status, created_at)
SELECT
    id,
    organization_id,
    user_id,
    CASE role
        WHEN 'cashier' THEN 'receptionist'
        WHEN 'accounting' THEN 'advisor'
        ELSE role
    END,
    CASE
        WHEN role = 'accounting' THEN 'revoked'
        ELSE status
    END,
    created_at
FROM staff_memberships;

DROP TABLE staff_memberships;
ALTER TABLE staff_memberships_v20 RENAME TO staff_memberships;

CREATE INDEX idx_staff_organization ON staff_memberships (organization_id);
CREATE INDEX idx_staff_user ON staff_memberships (organization_id, user_id);
CREATE UNIQUE INDEX idx_staff_one_active_owner
    ON staff_memberships (organization_id)
    WHERE role = 'owner' AND status = 'active';
CREATE UNIQUE INDEX idx_staff_one_membership_per_user
    ON staff_memberships (organization_id, user_id);

CREATE TABLE staff_permission_overrides (
    organization_id TEXT NOT NULL REFERENCES organizations (id) ON DELETE CASCADE,
    user_id TEXT NOT NULL REFERENCES users (id) ON DELETE CASCADE,
    permission TEXT NOT NULL CHECK (length(trim(permission)) > 0),
    effect TEXT NOT NULL CHECK (effect IN ('allow', 'deny')),
    created_at TEXT NOT NULL,
    updated_at TEXT NOT NULL,
    PRIMARY KEY (organization_id, user_id, permission)
) STRICT;

CREATE INDEX idx_staff_permission_overrides_user
    ON staff_permission_overrides (organization_id, user_id);
