import type {
  AccessDecision,
  AccessEvent,
  AccessEventInput,
  AccessEventSummary,
  AccessReconciliationInput,
  AccessReconciliationSummary,
  AccessRequest,
  AccessSnapshot,
  AttendanceInput,
  AttendanceReport,
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
  MemberCredential,
  MemberAccount,
  Membership,
  MembershipDashboard,
  MembershipPayment,
  MembershipPaymentReceipt,
  MembershipPlan,
  OpenCashSessionInput,
  PaymentCharge,
  Prospect,
  RecordCashMovementInput,
  RecordInventoryEntryInput,
  NewReportScheduleInput,
  RecordMembershipPaymentInput,
  RecoveryCode,
  ReportRun,
  ReportSchedule,
  RecoveryKit,
  RenewMembershipInput,
  RestoreOutcome,
  ResumeMembershipInput,
  Sale,
  SaleReturn,
  SessionSummary,
  SetReportScheduleEnabledInput,
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
  UpdateStaffPermissionsInput,
  CancelMembershipInput,
  CreateMemberInput,
  CreateMembershipInput,
  CreateMembershipPlanInput,
  CreateProspectInput,
  PauseMembershipInput,
  BookClassInput,
  Campaign,
  ClassBooking,
  ClassSession,
  CreateCampaignInput,
  CreateClassInput,
  CreateEquipmentInput,
  CreateMaintenanceOrderInput,
  Equipment,
  GymSettings,
  IntegrationJournalSummary,
  IntegrationLink,
  MaintenanceOrder,
  MarketingFunnel,
  UpdateCampaignInput,
  UpdateClassBookingInput,
  UpdateClassInput,
  UpdateEquipmentInput,
  UpdateGymSettingsInput,
  UpdateMaintenanceOrderInput,
  UpsertIntegrationLinkInput,
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
  updateStaffPermissions: (input: UpdateStaffPermissionsInput) =>
    invoke<StaffSummary>("update_staff_permissions", { input }),
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
  membershipPaymentReceipt: (paymentId: string) =>
    invoke<MembershipPaymentReceipt>("membership_payment_receipt", { paymentId }),
  simulateAccess: (request: AccessRequest) =>
    invoke<AccessDecision>("simulate_access", { request }),
  listAccessEvents: (limit = 20) =>
    invoke<AccessEventSummary[]>("list_access_events", { limit }),
  publishAccessSnapshot: () =>
    invoke<AccessSnapshot>("publish_access_snapshot"),
  latestAccessSnapshot: () =>
    invoke<AccessSnapshot | null>("latest_access_snapshot"),
  listMemberCredentials: (memberId: string) =>
    invoke<MemberCredential[]>("list_member_credentials", { memberId }),
  issueMemberCredential: (memberId: string) =>
    invoke<MemberCredential>("issue_member_credential", { memberId }),
  revokeMemberCredential: (credentialId: string) =>
    invoke<MemberCredential>("revoke_member_credential", { credentialId }),
  recordAccessEvent: (input: AccessEventInput) =>
    invoke<AccessEvent>("record_access_event", { input }),
  listAccessEventLog: (limit = 50) =>
    invoke<AccessEvent[]>("list_access_event_log", { limit }),
  reconcileAccessEvents: (input: AccessReconciliationInput) =>
    invoke<AccessReconciliationSummary>("reconcile_access_events", { input }),
  attendanceReport: (input: AttendanceInput) =>
    invoke<AttendanceReport>("attendance_report", { input }),
  createReportSchedule: (input: NewReportScheduleInput) =>
    invoke<ReportSchedule>("create_report_schedule", { input }),
  listReportSchedules: () =>
    invoke<ReportSchedule[]>("list_report_schedules"),
  setReportScheduleEnabled: (input: SetReportScheduleEnabledInput) =>
    invoke<ReportSchedule>("set_report_schedule_enabled", { input }),
  listReportRuns: (limit = 50) =>
    invoke<ReportRun[]>("list_report_runs", { limit }),
  runDueReportSchedules: () =>
    invoke<ReportRun[]>("run_due_report_schedules"),
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
  getGymSettings: () => invoke<GymSettings>("get_gym_settings"),
  updateGymSettings: (input: UpdateGymSettingsInput) =>
    invoke<GymSettings>("update_gym_settings", { input }),
  listIntegrationLinks: () =>
    invoke<IntegrationLink[]>("list_integration_links"),
  upsertIntegrationLink: (input: UpsertIntegrationLinkInput) =>
    invoke<IntegrationLink>("upsert_integration_link", { input }),
  integrationJournalSummary: () =>
    invoke<IntegrationJournalSummary>("integration_journal_summary"),
  listClasses: () => invoke<ClassSession[]>("list_classes"),
  createClass: (input: CreateClassInput) =>
    invoke<ClassSession>("create_class", { input }),
  updateClass: (input: UpdateClassInput) =>
    invoke<ClassSession>("update_class", { input }),
  listClassBookings: (classId: string) =>
    invoke<ClassBooking[]>("list_class_bookings", { classId }),
  bookClass: (input: BookClassInput) =>
    invoke<ClassBooking>("book_class", { input }),
  updateClassBooking: (input: UpdateClassBookingInput) =>
    invoke<ClassBooking>("update_class_booking", { input }),
  listCampaigns: () => invoke<Campaign[]>("list_campaigns"),
  createCampaign: (input: CreateCampaignInput) =>
    invoke<Campaign>("create_campaign", { input }),
  updateCampaign: (input: UpdateCampaignInput) =>
    invoke<Campaign>("update_campaign", { input }),
  marketingFunnel: (options?: { inactivityDays?: number; expiryDays?: number }) =>
    invoke<MarketingFunnel>("marketing_funnel", {
      inactivityDays: options?.inactivityDays ?? null,
      expiryDays: options?.expiryDays ?? null,
    }),
  listEquipment: () => invoke<Equipment[]>("list_equipment"),
  createEquipment: (input: CreateEquipmentInput) =>
    invoke<Equipment>("create_equipment", { input }),
  updateEquipment: (input: UpdateEquipmentInput) =>
    invoke<Equipment>("update_equipment", { input }),
  listMaintenanceOrders: (status?: string) =>
    invoke<MaintenanceOrder[]>("list_maintenance_orders", {
      status: status ?? null,
    }),
  createMaintenanceOrder: (input: CreateMaintenanceOrderInput) =>
    invoke<MaintenanceOrder>("create_maintenance_order", { input }),
  updateMaintenanceOrder: (input: UpdateMaintenanceOrderInput) =>
    invoke<MaintenanceOrder>("update_maintenance_order", { input }),
};
