import type {
  AccessDecision,
  AccessEventSummary,
  AccessRequest,
  AuditEventSummary,
  AddCatalogPriceInput,
  AddCatalogVariantInput,
  AuditIntegrity,
  BackupSummary,
  CashMovement,
  CashSession,
  CatalogProduct,
  CloseCashSessionInput,
  CreateCatalogItemInput,
  CreateSaleInput,
  CreateSaleReturnInput,
  AdjustInventoryInput,
  FinancialReport,
  FinancialReportExport,
  FinancialReportInput,
  HealthReport,
  InventoryItem,
  InventoryMovement,
  ListPaymentChargesInput,
  Member,
  MemberAccount,
  Membership,
  MembershipDashboard,
  MembershipPayment,
  MembershipPlan,
  OpenCashSessionInput,
  PaymentCharge,
  Prospect,
  RecordCashMovementInput,
  RecordInventoryEntryInput,
  RecordMembershipPaymentInput,
  RecoveryCode,
  RecoveryKit,
  RenewMembershipInput,
  RestoreOutcome,
  ResumeMembershipInput,
  Sale,
  SaleReturn,
  SessionSummary,
  SetupState,
  StaffRole,
  StaffStatus,
  StaffSummary,
  UpdateCatalogProductInput,
  UpdateCatalogVariantInput,
  UpdateInventoryThresholdInput,
  UpdateMemberInput,
  UpdateMembershipPlanInput,
  UpdateProspectInput,
  CancelMembershipInput,
  CreateMemberInput,
  CreateMembershipInput,
  CreateMembershipPlanInput,
  CreateProspectInput,
  PauseMembershipInput,
} from "@gym/contracts";
import { invoke } from "@tauri-apps/api/core";

export function isTauriRuntime(): boolean {
  return typeof window !== "undefined" && "__TAURI_INTERNALS__" in window;
}

export const api = {
  health: () => invoke<HealthReport>("health"),
  setupState: () => invoke<SetupState>("get_setup_state"),
  bootstrapOwner: (name: string, password: string, gymName?: string) =>
    invoke<RecoveryKit>("bootstrap_owner", { name, password, gymName }),
  generateOwnerRecovery: () => invoke<RecoveryCode>("generate_owner_recovery"),
  recoverOwner: (recoveryCode: string, newPassword: string) =>
    invoke<RecoveryKit>("recover_owner", { recoveryCode, newPassword }),
  login: (name: string, password: string) =>
    invoke<SessionSummary>("login", { name, password }),
  logout: () => invoke<void>("logout"),
  currentSession: () => invoke<SessionSummary | null>("current_session"),
  listStaff: () => invoke<StaffSummary[]>("list_staff"),
  createStaff: (name: string, role: StaffRole, password: string) =>
    invoke<StaffSummary>("create_staff", { name, role, password }),
  updateStaff: (userId: string, role: StaffRole, status: StaffStatus) =>
    invoke<StaffSummary>("update_staff", { userId, role, status }),
  membershipDashboard: () =>
    invoke<MembershipDashboard>("membership_dashboard"),
  listMembershipPlans: () =>
    invoke<MembershipPlan[]>("list_membership_plans"),
  createMembershipPlan: (input: CreateMembershipPlanInput) =>
    invoke<MembershipPlan>("create_membership_plan", { input }),
  updateMembershipPlan: (input: UpdateMembershipPlanInput) =>
    invoke<MembershipPlan>("update_membership_plan", { input }),
  listMembers: (query?: string) =>
    invoke<Member[]>("list_members", { query: query?.trim() || null }),
  getMember: (memberId: string) =>
    invoke<Member>("get_member", { memberId }),
  createMember: (input: CreateMemberInput) =>
    invoke<Member>("create_member", { input }),
  updateMember: (input: UpdateMemberInput) =>
    invoke<Member>("update_member", { input }),
  listProspects: (query?: string) =>
    invoke<Prospect[]>("list_prospects", { query: query?.trim() || null }),
  createProspect: (input: CreateProspectInput) =>
    invoke<Prospect>("create_prospect", { input }),
  updateProspect: (input: UpdateProspectInput) =>
    invoke<Prospect>("update_prospect", { input }),
  convertProspect: (prospectId: string) =>
    invoke<Member>("convert_prospect", { prospectId }),
  listMemberships: (memberId: string) =>
    invoke<Membership[]>("list_memberships", { memberId }),
  createMembership: (input: CreateMembershipInput) =>
    invoke<Membership>("create_membership", { input }),
  renewMembership: (input: RenewMembershipInput) =>
    invoke<Membership>("renew_membership", { input }),
  pauseMembership: (input: PauseMembershipInput) =>
    invoke<Membership>("pause_membership", { input }),
  resumeMembership: (input: ResumeMembershipInput) =>
    invoke<Membership>("resume_membership", { input }),
  cancelMembership: (input: CancelMembershipInput) =>
    invoke<Membership>("cancel_membership", { input }),
  memberAccount: (memberId: string) =>
    invoke<MemberAccount>("member_account", { memberId }),
  recordMembershipPayment: (input: RecordMembershipPaymentInput) =>
    invoke<MembershipPayment>("record_membership_payment", { input }),
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
  listSales: () => invoke<Sale[]>("list_sales"),
  createSale: (input: CreateSaleInput) => invoke<Sale>("create_sale", { input }),
  listSaleReturns: (saleId?: string) =>
    invoke<SaleReturn[]>("list_sale_returns", { saleId: saleId ?? null }),
  createSaleReturn: (input: CreateSaleReturnInput) =>
    invoke<SaleReturn>("create_sale_return", { input }),
  listPaymentCharges: (input: ListPaymentChargesInput = {}) =>
    invoke<PaymentCharge[]>("list_payment_charges", { input }),
  financialReport: (input: FinancialReportInput) =>
    invoke<FinancialReport>("financial_report", { input }),
  exportFinancialReportCsv: (input: FinancialReportInput) =>
    invoke<FinancialReportExport>("export_financial_report_csv", { input }),
  listInventory: () => invoke<InventoryItem[]>("list_inventory"),
  listInventoryMovements: (options?: { variantId?: string; limit?: number }) =>
    invoke<InventoryMovement[]>("list_inventory_movements", {
      variantId: options?.variantId ?? null,
      limit: options?.limit ?? null,
    }),
  recordInventoryEntry: (input: RecordInventoryEntryInput) =>
    invoke<InventoryMovement>("record_inventory_entry", { input }),
  adjustInventory: (input: AdjustInventoryInput) =>
    invoke<InventoryMovement>("adjust_inventory", { input }),
  updateInventoryThreshold: (input: UpdateInventoryThresholdInput) =>
    invoke<InventoryItem>("update_inventory_threshold", { input }),
  currentCashSession: () => invoke<CashSession | null>("current_cash_session"),
  listCashSessions: () => invoke<CashSession[]>("list_cash_sessions"),
  openCashSession: (input: OpenCashSessionInput) =>
    invoke<CashSession>("open_cash_session", { input }),
  closeCashSession: (input: CloseCashSessionInput) =>
    invoke<CashSession>("close_cash_session", { input }),
  listCashMovements: (sessionId: string) =>
    invoke<CashMovement[]>("list_cash_movements", { sessionId }),
  recordCashMovement: (input: RecordCashMovementInput) =>
    invoke<CashMovement>("record_cash_movement", { input }),
  listAuditEvents: (limit = 100) =>
    invoke<AuditEventSummary[]>("list_audit_events", { limit }),
  verifyAuditChain: () => invoke<AuditIntegrity>("verify_audit_chain"),
};
