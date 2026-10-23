import type {
  AccessDecision,
  AccessEventSummary,
  AccessRequest,
  AuditEventSummary,
  AddCatalogPriceInput,
  AddCatalogVariantInput,
  AuditIntegrity,
  BackupSummary,
  CatalogProduct,
  CreateCatalogItemInput,
  HealthReport,
  RestoreOutcome,
  SessionSummary,
  SetupState,
  SiteStatus,
  SiteSummary,
  StaffRole,
  StaffStatus,
  StaffSummary,
  UpdateCatalogProductInput,
  UpdateCatalogVariantInput,
} from "@gym/contracts";
import { invoke } from "@tauri-apps/api/core";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const api = {
  health: () => invoke<HealthReport>("health"),
  setupState: () => invoke<SetupState>("get_setup_state"),
  bootstrapOwner: (name: string, password: string, gymName?: string) =>
    invoke<SessionSummary>("bootstrap_owner", { name, password, gymName }),
  login: (name: string, password: string) =>
    invoke<SessionSummary>("login", { name, password }),
  logout: () => invoke<void>("logout"),
  currentSession: () => invoke<SessionSummary | null>("current_session"),
  listSites: () => invoke<SiteSummary[]>("list_sites"),
  createSite: (name: string) => invoke<SiteSummary>("create_site", { name }),
  updateSite: (siteId: string, name: string, status: SiteStatus) =>
    invoke<SiteSummary>("update_site", { siteId, name, status }),
  listStaff: () => invoke<StaffSummary[]>("list_staff"),
  createStaff: (name: string, role: StaffRole, siteId: string, password: string) =>
    invoke<StaffSummary>("create_staff", { name, role, siteId, password }),
  updateStaff: (userId: string, role: StaffRole, siteId: string, status: StaffStatus) =>
    invoke<StaffSummary>("update_staff", { userId, role, siteId, status }),
  simulateAccess: (request: AccessRequest) =>
    invoke<AccessDecision>("simulate_access", { request }),
  listAccessEvents: (limit = 20) =>
    invoke<AccessEventSummary[]>("list_access_events", { limit }),
  listBackups: () => invoke<BackupSummary[]>("list_backups"),
  createBackup: (passphrase: string) =>
    invoke<BackupSummary>("create_backup", { passphrase }),
  restoreBackup: (fileName: string, passphrase: string) =>
    invoke<RestoreOutcome>("restore_backup", { fileName, passphrase }),
  listCatalog: () => invoke<CatalogProduct[]>("list_catalog"),
  createCatalogItem: (input: CreateCatalogItemInput) =>
    invoke<CatalogProduct>("create_catalog_item", { input }),
  addCatalogVariant: (input: AddCatalogVariantInput) =>
    invoke<CatalogProduct>("add_catalog_variant", { input }),
  updateCatalogProduct: (input: UpdateCatalogProductInput) =>
    invoke<CatalogProduct>("update_catalog_product", { input }),
  updateCatalogVariant: (input: UpdateCatalogVariantInput) =>
    invoke<CatalogProduct>("update_catalog_variant", { input }),
  addCatalogPrice: (input: AddCatalogPriceInput) =>
    invoke<CatalogProduct>("add_catalog_price", { input }),
  listAuditEvents: (limit = 100) =>
    invoke<AuditEventSummary[]>("list_audit_events", { limit }),
  verifyAuditChain: () => invoke<AuditIntegrity>("verify_audit_chain"),
};
