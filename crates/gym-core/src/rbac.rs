use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Roles enabled in the desktop application.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Owner,
    Manager,
    Receptionist,
    Advisor,
    Trainer,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    ViewMembers,
    ManageMembers,
    ViewAssignedMembers,
    ViewAccess,
    OperateAccess,
    ManageAccessSnapshots,
    ViewSales,
    OperatePos,
    SuperviseCash,
    ViewClasses,
    ManageClasses,
    ManageReservations,
    ViewMarketing,
    ManageProspects,
    ManageMarketing,
    ViewInventory,
    ManageInventory,
    ViewCatalog,
    ManageCatalog,
    ViewFinance,
    ManageFinance,
    ExportFinance,
    ViewReports,
    ViewCommercialReports,
    ViewCashReports,
    ExportReports,
    ViewStaff,
    ManageStaff,
    ManagePermissions,
    ViewSettings,
    ManageOperationalSettings,
    ManageSettings,
    ViewAudit,
    ManageBackups,
    ViewIntegrations,
    ManageIntegrations,
    ViewMaintenance,
    ManageMaintenance,
    ReportMaintenanceIssue,
}

/// A per-user exception on top of the role template.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PermissionOverride {
    pub permission: Permission,
    pub allow: bool,
}

impl Role {
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Manager => "manager",
            Role::Receptionist => "receptionist",
            Role::Advisor => "advisor",
            Role::Trainer => "trainer",
        }
    }
}

impl Permission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Permission::ViewMembers => "viewMembers",
            Permission::ManageMembers => "manageMembers",
            Permission::ViewAssignedMembers => "viewAssignedMembers",
            Permission::ViewAccess => "viewAccess",
            Permission::OperateAccess => "operateAccess",
            Permission::ManageAccessSnapshots => "manageAccessSnapshots",
            Permission::ViewSales => "viewSales",
            Permission::OperatePos => "operatePos",
            Permission::SuperviseCash => "superviseCash",
            Permission::ViewClasses => "viewClasses",
            Permission::ManageClasses => "manageClasses",
            Permission::ManageReservations => "manageReservations",
            Permission::ViewMarketing => "viewMarketing",
            Permission::ManageProspects => "manageProspects",
            Permission::ManageMarketing => "manageMarketing",
            Permission::ViewInventory => "viewInventory",
            Permission::ManageInventory => "manageInventory",
            Permission::ViewCatalog => "viewCatalog",
            Permission::ManageCatalog => "manageCatalog",
            Permission::ViewFinance => "viewFinance",
            Permission::ManageFinance => "manageFinance",
            Permission::ExportFinance => "exportFinance",
            Permission::ViewReports => "viewReports",
            Permission::ViewCommercialReports => "viewCommercialReports",
            Permission::ViewCashReports => "viewCashReports",
            Permission::ExportReports => "exportReports",
            Permission::ViewStaff => "viewStaff",
            Permission::ManageStaff => "manageStaff",
            Permission::ManagePermissions => "managePermissions",
            Permission::ViewSettings => "viewSettings",
            Permission::ManageOperationalSettings => "manageOperationalSettings",
            Permission::ManageSettings => "manageSettings",
            Permission::ViewAudit => "viewAudit",
            Permission::ManageBackups => "manageBackups",
            Permission::ViewIntegrations => "viewIntegrations",
            Permission::ManageIntegrations => "manageIntegrations",
            Permission::ViewMaintenance => "viewMaintenance",
            Permission::ManageMaintenance => "manageMaintenance",
            Permission::ReportMaintenanceIssue => "reportMaintenanceIssue",
        }
    }
}

pub const ROLES: [Role; 5] = [
    Role::Owner,
    Role::Manager,
    Role::Receptionist,
    Role::Advisor,
    Role::Trainer,
];

pub const PERMISSIONS: [Permission; 39] = [
    Permission::ViewMembers,
    Permission::ManageMembers,
    Permission::ViewAssignedMembers,
    Permission::ViewAccess,
    Permission::OperateAccess,
    Permission::ManageAccessSnapshots,
    Permission::ViewSales,
    Permission::OperatePos,
    Permission::SuperviseCash,
    Permission::ViewClasses,
    Permission::ManageClasses,
    Permission::ManageReservations,
    Permission::ViewMarketing,
    Permission::ManageProspects,
    Permission::ManageMarketing,
    Permission::ViewInventory,
    Permission::ManageInventory,
    Permission::ViewCatalog,
    Permission::ManageCatalog,
    Permission::ViewFinance,
    Permission::ManageFinance,
    Permission::ExportFinance,
    Permission::ViewReports,
    Permission::ViewCommercialReports,
    Permission::ViewCashReports,
    Permission::ExportReports,
    Permission::ViewStaff,
    Permission::ManageStaff,
    Permission::ManagePermissions,
    Permission::ViewSettings,
    Permission::ManageOperationalSettings,
    Permission::ManageSettings,
    Permission::ViewAudit,
    Permission::ManageBackups,
    Permission::ViewIntegrations,
    Permission::ManageIntegrations,
    Permission::ViewMaintenance,
    Permission::ManageMaintenance,
    Permission::ReportMaintenanceIssue,
];

/// Permissions that only the owner can hold, even with a manual override.
pub const OWNER_ONLY: [Permission; 2] = [Permission::ManagePermissions, Permission::ManageBackups];

pub fn permission_from_str(value: &str) -> Option<Permission> {
    PERMISSIONS
        .iter()
        .copied()
        .find(|permission| permission.as_str() == value)
}

pub fn role_from_str(value: &str) -> Option<Role> {
    ROLES.iter().copied().find(|role| role.as_str() == value)
}

pub fn is_owner_only(permission: Permission) -> bool {
    OWNER_ONLY.contains(&permission)
}

/// Role template. Overrides are applied by [`effective_permissions`].
pub fn permissions_for(role: Role) -> &'static [Permission] {
    match role {
        Role::Owner => &PERMISSIONS,
        Role::Manager => &[
            Permission::ViewMembers,
            Permission::ManageMembers,
            Permission::ViewAccess,
            Permission::OperateAccess,
            Permission::ManageAccessSnapshots,
            Permission::ViewSales,
            Permission::OperatePos,
            Permission::SuperviseCash,
            Permission::ViewClasses,
            Permission::ManageClasses,
            Permission::ManageReservations,
            Permission::ViewMarketing,
            Permission::ManageProspects,
            Permission::ManageMarketing,
            Permission::ViewInventory,
            Permission::ManageInventory,
            Permission::ViewCatalog,
            Permission::ManageCatalog,
            Permission::ViewFinance,
            Permission::ExportFinance,
            Permission::ViewReports,
            Permission::ViewCommercialReports,
            Permission::ViewCashReports,
            Permission::ExportReports,
            Permission::ViewStaff,
            Permission::ManageStaff,
            Permission::ViewSettings,
            Permission::ManageOperationalSettings,
            Permission::ManageSettings,
            Permission::ViewAudit,
            Permission::ViewIntegrations,
            Permission::ManageIntegrations,
            Permission::ViewMaintenance,
            Permission::ManageMaintenance,
            Permission::ReportMaintenanceIssue,
        ],
        Role::Receptionist => &[
            Permission::ViewMembers,
            Permission::ManageMembers,
            Permission::ViewAccess,
            Permission::OperateAccess,
            Permission::ViewSales,
            Permission::OperatePos,
            Permission::ViewClasses,
            Permission::ManageReservations,
            Permission::ViewMarketing,
            Permission::ManageProspects,
            Permission::ViewInventory,
            Permission::ManageInventory,
            Permission::ViewCatalog,
            Permission::ViewReports,
            Permission::ViewCashReports,
            Permission::ViewMaintenance,
            Permission::ReportMaintenanceIssue,
        ],
        Role::Advisor => &[
            Permission::ViewMembers,
            Permission::ManageMembers,
            Permission::ViewSales,
            Permission::ViewClasses,
            Permission::ManageReservations,
            Permission::ViewMarketing,
            Permission::ManageProspects,
            Permission::ManageMarketing,
            Permission::ViewCatalog,
            Permission::ViewReports,
            Permission::ViewCommercialReports,
            Permission::ViewMaintenance,
            Permission::ReportMaintenanceIssue,
        ],
        Role::Trainer => &[
            Permission::ViewMembers,
            Permission::ViewAssignedMembers,
            Permission::ViewClasses,
            Permission::ManageClasses,
            Permission::ViewMaintenance,
            Permission::ReportMaintenanceIssue,
        ],
    }
}

/// Applies allow/deny overrides to the role template. Owner keeps every
/// permission and owner-only permissions cannot be granted to other roles.
pub fn effective_permissions(role: Role, overrides: &[PermissionOverride]) -> Vec<Permission> {
    if role == Role::Owner {
        return PERMISSIONS.to_vec();
    }

    let mut permissions = permissions_for(role).to_vec();
    for entry in overrides {
        if entry.allow {
            if !is_owner_only(entry.permission) && !permissions.contains(&entry.permission) {
                permissions.push(entry.permission);
            }
        } else {
            permissions.retain(|permission| *permission != entry.permission);
        }
    }
    permissions
}

pub fn is_allowed(role: Role, overrides: &[PermissionOverride], permission: Permission) -> bool {
    role == Role::Owner || effective_permissions(role, overrides).contains(&permission)
}

pub fn authorize(
    role: Role,
    overrides: &[PermissionOverride],
    permission: Permission,
) -> Result<(), CoreError> {
    if is_allowed(role, overrides, permission) {
        Ok(())
    } else {
        Err(CoreError::Forbidden(permission))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn receptionist_cannot_manage_backups() {
        assert_eq!(
            authorize(Role::Receptionist, &[], Permission::ManageBackups),
            Err(CoreError::Forbidden(Permission::ManageBackups))
        );
    }

    #[test]
    fn receptionist_cannot_change_catalog_prices() {
        assert_eq!(
            authorize(Role::Receptionist, &[], Permission::ManageCatalog),
            Err(CoreError::Forbidden(Permission::ManageCatalog))
        );
    }

    #[test]
    fn owner_can_manage_everything() {
        for permission in PERMISSIONS {
            assert!(authorize(Role::Owner, &[], permission).is_ok());
        }
    }

    #[test]
    fn manager_can_supervise_cash_but_not_manage_finance() {
        assert!(authorize(Role::Manager, &[], Permission::SuperviseCash).is_ok());
        assert!(authorize(Role::Manager, &[], Permission::ManageFinance).is_err());
        assert!(authorize(Role::Manager, &[], Permission::ManagePermissions).is_err());
    }

    #[test]
    fn trainer_stays_out_of_sales_and_finance() {
        assert!(authorize(Role::Trainer, &[], Permission::ManageClasses).is_ok());
        assert!(authorize(Role::Trainer, &[], Permission::OperatePos).is_err());
        assert!(authorize(Role::Trainer, &[], Permission::ViewFinance).is_err());
    }

    #[test]
    fn allow_override_grants_a_permission_deny_override_removes_one() {
        let allow = [PermissionOverride {
            permission: Permission::ViewFinance,
            allow: true,
        }];
        assert!(is_allowed(
            Role::Receptionist,
            &allow,
            Permission::ViewFinance
        ));

        let deny = [PermissionOverride {
            permission: Permission::OperatePos,
            allow: false,
        }];
        assert!(!is_allowed(
            Role::Receptionist,
            &deny,
            Permission::OperatePos
        ));
    }

    #[test]
    fn owner_only_permissions_cannot_be_granted_by_override() {
        let allow = [PermissionOverride {
            permission: Permission::ManageBackups,
            allow: true,
        }];
        assert!(!is_allowed(
            Role::Manager,
            &allow,
            Permission::ManageBackups
        ));
    }

    #[test]
    fn owner_cannot_be_denied() {
        let deny = [PermissionOverride {
            permission: Permission::ManageSettings,
            allow: false,
        }];
        assert!(is_allowed(Role::Owner, &deny, Permission::ManageSettings));
    }
}
