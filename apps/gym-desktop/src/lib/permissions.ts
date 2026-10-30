import type { Permission, StaffRole } from "@gym/contracts";

export const roleLabels: Record<StaffRole, string> = {
  owner: "Dueño",
  manager: "Gerente",
  receptionist: "Recepción",
  advisor: "Asesor",
  trainer: "Entrenador",
};

export const roleDescriptions: Record<StaffRole, string> = {
  owner: "Control total del gimnasio y permisos sensibles.",
  manager: "Supervisa la operación, el personal y la configuración.",
  receptionist: "Atiende socios, cobros, accesos y reservas.",
  advisor: "Da seguimiento comercial y gestiona prospectos.",
  trainer: "Consulta socios asignados y administra clases.",
};

export const ownerOnlyPermissions: Permission[] = [
  "managePermissions",
  "manageBackups",
];

export const permissionGroups: { label: string; permissions: Permission[] }[] = [
  {
    label: "Socios",
    permissions: ["viewMembers", "manageMembers", "viewAssignedMembers"],
  },
  {
    label: "Acceso y asistencia",
    permissions: ["viewAccess", "operateAccess", "manageAccessSnapshots"],
  },
  {
    label: "Ventas y caja",
    permissions: ["viewSales", "operatePos", "superviseCash"],
  },
  {
    label: "Clases y reservas",
    permissions: ["viewClasses", "manageClasses", "manageReservations"],
  },
  {
    label: "Marketing",
    permissions: ["viewMarketing", "manageProspects", "manageMarketing"],
  },
  {
    label: "Inventario y catálogo",
    permissions: [
      "viewInventory",
      "manageInventory",
      "viewCatalog",
      "manageCatalog",
    ],
  },
  {
    label: "Finanzas",
    permissions: ["viewFinance", "manageFinance", "exportFinance"],
  },
  {
    label: "Reportes",
    permissions: [
      "viewReports",
      "viewCommercialReports",
      "viewCashReports",
      "exportReports",
    ],
  },
  {
    label: "Colaboradores",
    permissions: ["viewStaff", "manageStaff", "managePermissions"],
  },
  {
    label: "Configuración",
    permissions: ["viewSettings", "manageOperationalSettings", "manageSettings"],
  },
  {
    label: "Sistema",
    permissions: [
      "viewAudit",
      "manageBackups",
      "viewIntegrations",
      "manageIntegrations",
      "viewMaintenance",
      "manageMaintenance",
      "reportMaintenanceIssue",
    ],
  },
];

export const permissionLabels: Record<Permission, string> = {
  viewMembers: "Ver socios",
  manageMembers: "Administrar socios",
  viewAssignedMembers: "Ver socios asignados",
  viewAccess: "Ver accesos y asistencia",
  operateAccess: "Registrar accesos",
  manageAccessSnapshots: "Publicar snapshots de acceso",
  viewSales: "Ver ventas",
  operatePos: "Cobrar y operar caja",
  superviseCash: "Supervisar caja",
  viewClasses: "Ver clases",
  manageClasses: "Administrar clases",
  manageReservations: "Administrar reservas",
  viewMarketing: "Ver marketing",
  manageProspects: "Administrar prospectos",
  manageMarketing: "Administrar campañas",
  viewInventory: "Ver inventario",
  manageInventory: "Administrar inventario",
  viewCatalog: "Ver catálogo",
  manageCatalog: "Administrar catálogo y precios",
  viewFinance: "Ver finanzas",
  manageFinance: "Administrar finanzas",
  exportFinance: "Exportar finanzas",
  viewReports: "Ver reportes",
  viewCommercialReports: "Ver reportes comerciales",
  viewCashReports: "Ver reportes de caja",
  exportReports: "Exportar reportes",
  viewStaff: "Ver colaboradores",
  manageStaff: "Administrar colaboradores",
  managePermissions: "Administrar permisos",
  viewSettings: "Ver configuración",
  manageOperationalSettings: "Administrar configuración operativa",
  manageSettings: "Administrar configuración general",
  viewAudit: "Ver auditoría",
  manageBackups: "Administrar backups",
  viewIntegrations: "Ver complementos",
  manageIntegrations: "Administrar complementos",
  viewMaintenance: "Ver mantenimiento",
  manageMaintenance: "Administrar mantenimiento",
  reportMaintenanceIssue: "Reportar incidencias",
};

export const rolePermissions: Record<StaffRole, Permission[]> = {
  owner: Object.keys(permissionLabels) as Permission[],
  manager: [
    "viewMembers",
    "manageMembers",
    "viewAccess",
    "operateAccess",
    "manageAccessSnapshots",
    "viewSales",
    "operatePos",
    "superviseCash",
    "viewClasses",
    "manageClasses",
    "manageReservations",
    "viewMarketing",
    "manageProspects",
    "manageMarketing",
    "viewInventory",
    "manageInventory",
    "viewCatalog",
    "manageCatalog",
    "viewFinance",
    "exportFinance",
    "viewReports",
    "viewCommercialReports",
    "viewCashReports",
    "exportReports",
    "viewStaff",
    "manageStaff",
    "viewSettings",
    "manageOperationalSettings",
    "manageSettings",
    "viewAudit",
    "viewIntegrations",
    "manageIntegrations",
    "viewMaintenance",
    "manageMaintenance",
    "reportMaintenanceIssue",
  ],
  receptionist: [
    "viewMembers",
    "manageMembers",
    "viewAccess",
    "operateAccess",
    "viewSales",
    "operatePos",
    "viewClasses",
    "manageReservations",
    "viewMarketing",
    "manageProspects",
    "viewInventory",
    "manageInventory",
    "viewCatalog",
    "viewReports",
    "viewCashReports",
    "viewMaintenance",
    "reportMaintenanceIssue",
  ],
  advisor: [
    "viewMembers",
    "manageMembers",
    "viewSales",
    "viewClasses",
    "manageReservations",
    "viewMarketing",
    "manageProspects",
    "manageMarketing",
    "viewCatalog",
    "viewReports",
    "viewCommercialReports",
    "viewMaintenance",
    "reportMaintenanceIssue",
  ],
  trainer: [
    "viewMembers",
    "viewAssignedMembers",
    "viewClasses",
    "manageClasses",
    "viewMaintenance",
    "reportMaintenanceIssue",
  ],
};

export function effectivePermission(
  role: StaffRole,
  overrides: { permission: Permission; effect: "allow" | "deny" }[],
  permission: Permission,
): boolean {
  if (role === "owner") {
    return true;
  }
  const override = overrides.find((entry) => entry.permission === permission);
  if (override) {
    return override.effect === "allow";
  }
  return rolePermissions[role].includes(permission);
}
