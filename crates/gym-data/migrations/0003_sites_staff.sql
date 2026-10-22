-- Site names and logins are unique inside a gym. V1 keeps one membership per
-- user so a person cannot accidentally inherit permissions from two sites.
CREATE UNIQUE INDEX idx_sites_organization_name
    ON sites (organization_id, name COLLATE NOCASE);

CREATE UNIQUE INDEX idx_staff_one_membership_per_user
    ON staff_memberships (organization_id, user_id);
