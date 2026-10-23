use serde::{Deserialize, Serialize};

use crate::error::CoreError;

/// Roles enabled in V1. Coach, marketing and member are pending.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Role {
    Owner,
    Manager,
    Cashier,
    Accounting,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub enum Permission {
    ManageSettings,
    ManageStaff,
    ViewFinance,
    OperatePos,
    ManageInventory,
    ManageCatalog,
    ManageBackups,
    ViewAudit,
}

impl Role {
    pub const fn as_str(self) -> &'static str {
        match self {
            Role::Owner => "owner",
            Role::Manager => "manager",
            Role::Cashier => "cashier",
            Role::Accounting => "accounting",
        }
    }
}

impl Permission {
    pub const fn as_str(self) -> &'static str {
        match self {
            Permission::ManageSettings => "manageSettings",
            Permission::ManageStaff => "manageStaff",
            Permission::ViewFinance => "viewFinance",
            Permission::OperatePos => "operatePos",
            Permission::ManageInventory => "manageInventory",
            Permission::ManageCatalog => "manageCatalog",
            Permission::ManageBackups => "manageBackups",
            Permission::ViewAudit => "viewAudit",
        }
    }
}

pub const V1_ROLES: [Role; 4] = [Role::Owner, Role::Manager, Role::Cashier, Role::Accounting];

pub const V1_PERMISSIONS: [Permission; 8] = [
    Permission::ManageSettings,
    Permission::ManageStaff,
    Permission::ViewFinance,
    Permission::OperatePos,
    Permission::ManageInventory,
    Permission::ManageCatalog,
    Permission::ManageBackups,
    Permission::ViewAudit,
];

/// Authorization is role + organization + site aware. Resource-level checks are
/// applied by the caller before writing.
pub fn permissions_for(role: Role) -> &'static [Permission] {
    match role {
        Role::Owner => &[
            Permission::ManageSettings,
            Permission::ManageStaff,
            Permission::ViewFinance,
            Permission::OperatePos,
            Permission::ManageInventory,
            Permission::ManageCatalog,
            Permission::ManageBackups,
            Permission::ViewAudit,
        ],
        Role::Manager => &[
            Permission::ManageStaff,
            Permission::ViewFinance,
            Permission::OperatePos,
            Permission::ManageInventory,
            Permission::ManageCatalog,
            Permission::ManageBackups,
            Permission::ViewAudit,
        ],
        Role::Cashier => &[Permission::OperatePos, Permission::ManageInventory],
        Role::Accounting => &[Permission::ViewFinance, Permission::ViewAudit],
    }
}

pub fn authorize(role: Role, permission: Permission) -> Result<(), CoreError> {
    if permissions_for(role).contains(&permission) {
        Ok(())
    } else {
        Err(CoreError::Forbidden(permission))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn cashier_cannot_manage_backups() {
        assert_eq!(
            authorize(Role::Cashier, Permission::ManageBackups),
            Err(CoreError::Forbidden(Permission::ManageBackups))
        );
    }

    #[test]
    fn cashier_cannot_change_catalog_prices() {
        assert_eq!(
            authorize(Role::Cashier, Permission::ManageCatalog),
            Err(CoreError::Forbidden(Permission::ManageCatalog))
        );
    }

    #[test]
    fn owner_can_manage_everything() {
        for permission in V1_PERMISSIONS {
            assert!(authorize(Role::Owner, permission).is_ok());
        }
    }

    #[test]
    fn permission_matrix_matches_v1_contract() {
        assert_eq!(permissions_for(Role::Owner), V1_PERMISSIONS.as_slice());
        assert_eq!(
            permissions_for(Role::Manager),
            &[
                Permission::ManageStaff,
                Permission::ViewFinance,
                Permission::OperatePos,
                Permission::ManageInventory,
                Permission::ManageCatalog,
                Permission::ManageBackups,
                Permission::ViewAudit,
            ]
        );
        assert_eq!(
            permissions_for(Role::Cashier),
            &[Permission::OperatePos, Permission::ManageInventory]
        );
        assert_eq!(
            permissions_for(Role::Accounting),
            &[Permission::ViewFinance, Permission::ViewAudit]
        );
    }

    #[test]
    fn accounting_is_read_only() {
        assert!(authorize(Role::Accounting, Permission::ViewFinance).is_ok());
        assert!(authorize(Role::Accounting, Permission::ViewAudit).is_ok());
        assert!(authorize(Role::Accounting, Permission::OperatePos).is_err());
        assert!(authorize(Role::Accounting, Permission::ManageBackups).is_err());
    }
}
