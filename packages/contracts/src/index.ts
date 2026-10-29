/**
 * TypeScript mirror of `crates/gym-contracts`.
 * Keep both sides in sync; `CONTRACT_VERSION` guards breaking changes.
 */

export const CONTRACT_VERSION = "3.1.0";

export type HealthStatus = "ok" | "degraded" | "down";

export interface ModuleHealth {
  name: string;
  status: HealthStatus;
  detail?: string;
}

export interface DatabaseStatus {
  path: string;
  encrypted: boolean;
  cipherVersion?: string;
  schemaVersion: number;
  integrityOk: boolean;
  organizations: number;
}

export interface HealthReport {
  service: string;
  version: string;
  contractVersion: string;
  status: HealthStatus;
  modules: ModuleHealth[];
  database: DatabaseStatus | null;
  checkedAt: string;
}

export interface AccessRequest {
  credentialId: string;
  deviceId: string;
  idempotencyKey: string;
}

export interface AccessDecision {
  eventId: string;
  allowed: boolean;
  reason: string;
  duplicate: boolean;
  idempotencyKey: string;
  latencyMs: number;
  decidedAt: string;
}

export interface AccessEventSummary {
  eventId: string;
  allowed: boolean;
  idempotencyKey: string;
  occurredAt: string;
}

export interface AccessSnapshot {
  revision: number;
  generatedAt: string;
  memberCount: number;
  credentialCount: number;
  checksum: string;
  signature: string;
  payloadJson: string;
}

export interface MemberCredential {
  id: string;
  memberId: string;
  memberNumber: string;
  memberName: string;
  code: string;
  status: string;
  revision: number;
  issuedAt: string;
  revokedAt: string | null;
}

export interface AccessEventInput {
  credentialCode: string;
  deviceId: string;
  zone: string | null;
  allowed: boolean;
  reason: string;
  snapshotRevision: number;
  occurredAt: string;
  idempotencyKey: string;
}

export interface AccessEvent {
  id: string;
  credentialCode: string;
  memberId: string | null;
  memberNumber: string | null;
  memberName: string | null;
  deviceId: string;
  zone: string | null;
  allowed: boolean;
  reason: string;
  snapshotRevision: number;
  idempotencyKey: string;
  payloadHash: string;
  reconciliationStatus: string;
  occurredAt: string;
}

export interface AccessReconciliationInput {
  snapshotRevision: number;
  events: AccessEventInput[];
}

export interface AccessConflict {
  idempotencyKey: string;
  reason: string;
}

export interface AccessReconciliationSummary {
  snapshotRevision: number;
  inserted: number;
  duplicates: number;
  stale: number;
  conflicts: AccessConflict[];
}

export interface AttendanceInput {
  from: string;
  to: string;
}

export interface AttendanceHour {
  hour: number;
  visits: number;
  uniqueMembers: number;
}

export interface AttendanceDay {
  date: string;
  visits: number;
  uniqueMembers: number;
}

export interface AttendancePlan {
  planId: string | null;
  planName: string | null;
  visits: number;
  uniqueMembers: number;
}

export interface AttendanceZone {
  zone: string | null;
  visits: number;
  uniqueMembers: number;
}

export interface AttendanceMember {
  memberId: string;
  memberNumber: string;
  memberName: string;
  visits: number;
  lastVisitAt: string;
}

export interface AttendanceReport {
  from: string;
  to: string;
  generatedAt: string;
  totalVisits: number;
  uniqueMembers: number;
  peakHour: number;
  lowDemandHours: AttendanceHour[];
  hours: AttendanceHour[];
  days: AttendanceDay[];
  plans: AttendancePlan[];
  zones: AttendanceZone[];
  members: AttendanceMember[];
}

export type StaffRole =
  | "owner"
  | "manager"
  | "receptionist"
  | "advisor"
  | "trainer";

export type Permission =
  | "viewMembers"
  | "manageMembers"
  | "viewAssignedMembers"
  | "viewAccess"
  | "operateAccess"
  | "manageAccessSnapshots"
  | "viewSales"
  | "operatePos"
  | "superviseCash"
  | "viewClasses"
  | "manageClasses"
  | "manageReservations"
  | "viewMarketing"
  | "manageProspects"
  | "manageMarketing"
  | "viewInventory"
  | "manageInventory"
  | "viewCatalog"
  | "manageCatalog"
  | "viewFinance"
  | "manageFinance"
  | "exportFinance"
  | "viewReports"
  | "viewCommercialReports"
  | "viewCashReports"
  | "exportReports"
  | "viewStaff"
  | "manageStaff"
  | "managePermissions"
  | "viewSettings"
  | "manageOperationalSettings"
  | "manageSettings"
  | "viewAudit"
  | "manageBackups"
  | "viewIntegrations"
  | "manageIntegrations"
  | "viewMaintenance"
  | "manageMaintenance"
  | "reportMaintenanceIssue";

export interface SetupState {
  configured: boolean;
  gymName?: string;
  ownerReady: boolean;
}

export interface SessionSummary {
  userId: string;
  organizationId: string;
  organizationName: string;
  name: string;
  role: StaffRole;
  permissions: Permission[];
}

export interface RecoveryKit {
  session: SessionSummary;
  recoveryCode: string;
}

export interface RecoveryCode {
  recoveryCode: string;
}

export type StaffStatus = "active" | "revoked";

export type PermissionEffect = "allow" | "deny";

export interface PermissionOverride {
  permission: Permission;
  effect: PermissionEffect;
}

export interface UpdateStaffPermissionsInput {
  userId: string;
  overrides: PermissionOverride[];
}

export interface StaffSummary {
  userId: string;
  name: string;
  role: StaffRole;
  status: StaffStatus;
  lastLoginAt: string | null;
  createdAt: string;
  permissionOverrides: PermissionOverride[];
}

export type CatalogStatus = "active" | "archived";

export type CatalogUnit = "piece" | "service" | "session" | "package";

export interface CatalogPrice {
  id: string;
  currency: string;
  amountMinor: number;
  taxRateBps: number;
  priceIncludesTax: boolean;
  effectiveFrom: string;
  createdAt: string;
}

export interface CatalogVariant {
  id: string;
  name: string;
  sku: string;
  barcode: string | null;
  unit: CatalogUnit;
  costMinor: number;
  status: CatalogStatus;
  currentPrice: CatalogPrice | null;
  createdAt: string;
  updatedAt: string;
}

export interface CatalogProduct {
  id: string;
  name: string;
  category: string | null;
  status: CatalogStatus;
  variants: CatalogVariant[];
  createdAt: string;
  updatedAt: string;
}

export interface CatalogPriceInput {
  amountMinor: number;
  taxRateBps: number;
  priceIncludesTax: boolean;
  effectiveFrom?: string;
}

export interface CatalogVariantInput {
  name: string;
  sku: string;
  barcode?: string;
  unit: CatalogUnit;
  costMinor: number;
  price: CatalogPriceInput;
}

export interface CreateCatalogItemInput {
  name: string;
  category?: string;
  variant: CatalogVariantInput;
}

export interface AddCatalogVariantInput {
  productId: string;
  variant: CatalogVariantInput;
}

export interface UpdateCatalogProductInput {
  productId: string;
  name: string;
  category?: string;
  status: CatalogStatus;
}

export interface UpdateCatalogVariantInput {
  variantId: string;
  name: string;
  sku: string;
  barcode?: string;
  unit: CatalogUnit;
  costMinor: number;
  status: CatalogStatus;
}

export interface AddCatalogPriceInput {
  variantId: string;
  price: CatalogPriceInput;
}

export type PaymentMethod = "cash" | "transfer" | "point";

export type MemberStatus = "active" | "inactive" | "archived";

export type ProspectStatus = "new" | "contacted" | "trial" | "converted" | "lost";

export type MembershipStatus =
  | "scheduled"
  | "active"
  | "paused"
  | "expired"
  | "cancelled";

export type MembershipChargeStatus = "pending" | "paid" | "void";

export interface MembershipPlan {
  id: string;
  name: string;
  description: string | null;
  durationDays: number;
  priceMinor: number;
  currency: string;
  graceDays: number;
  version: number;
  accessDays: string;
  accessStartMinute: number;
  accessEndMinute: number;
  minAge: number | null;
  maxAge: number | null;
  active: boolean;
  createdAt: string;
  updatedAt: string;
}

export interface Member {
  id: string;
  memberNumber: string;
  firstName: string;
  lastName: string;
  phone: string | null;
  email: string | null;
  birthDate: string | null;
  status: MemberStatus;
  joinedAt: string;
  notes: string | null;
  activeMembership: Membership | null;
  debtMinor: number;
  createdAt: string;
  updatedAt: string;
}

export interface Membership {
  id: string;
  memberId: string;
  memberName: string;
  planId: string;
  planName: string;
  planVersion: number;
  planVersionId: string;
  status: MembershipStatus;
  startDate: string;
  endDate: string;
  priceMinor: number;
  currency: string;
  graceDays: number;
  accessDays: string;
  accessStartMinute: number;
  accessEndMinute: number;
  minAge: number | null;
  maxAge: number | null;
  pausedAt: string | null;
  pauseUntil: string | null;
  cancellationReason: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface Prospect {
  id: string;
  name: string;
  phone: string | null;
  email: string | null;
  source: string | null;
  status: ProspectStatus;
  notes: string | null;
  convertedMemberId: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface MembershipCharge {
  id: string;
  membershipId: string;
  memberId: string;
  description: string;
  amountMinor: number;
  currency: string;
  dueDate: string;
  status: MembershipChargeStatus;
  paidMinor: number;
  createdAt: string;
  updatedAt: string;
}

export interface MembershipPayment {
  id: string;
  chargeId: string;
  membershipId: string;
  memberId: string;
  amountMinor: number;
  paymentMethod: PaymentMethod;
  reference: string | null;
  receiptNumber: string;
  cashSessionId: string | null;
  receivedBy: string | null;
  receivedByName: string | null;
  createdAt: string;
}

export interface MembershipPaymentReceipt {
  paymentId: string;
  receiptNumber: string;
  gymName: string;
  memberNumber: string;
  memberName: string;
  chargeDescription: string;
  amountMinor: number;
  currency: string;
  paymentMethod: PaymentMethod;
  reference: string | null;
  receivedByName: string | null;
  createdAt: string;
}

export interface MembershipDashboardTrendPoint {
  month: string;
  newMembers: number;
  renewals: number;
  cancellations: number;
  incomeMinor: number;
}

export interface MemberAccount {
  memberId: string;
  currency: string;
  charges: MembershipCharge[];
  payments: MembershipPayment[];
  balanceMinor: number;
}

export interface MembershipDashboard {
  periodStart: string;
  periodEnd: string;
  timezone: string;
  activeMembers: number;
  activeMemberships: number;
  expiringSoon: number;
  pastDue: number;
  debtMinor: number;
  newMembersThisMonth: number;
  incomeThisMonthMinor: number;
  renewalsThisMonth: number;
  cancellationsThisMonth: number;
  trend: MembershipDashboardTrendPoint[];
  recentPayments: MembershipPaymentReceipt[];
  expiringMemberships: Membership[];
}

export interface CreateMemberInput {
  firstName: string;
  lastName: string;
  phone?: string;
  email?: string;
  birthDate?: string;
  notes?: string;
}

export interface UpdateMemberInput {
  memberId: string;
  firstName: string;
  lastName: string;
  phone?: string;
  email?: string;
  birthDate?: string;
  status: MemberStatus;
  notes?: string;
}

export interface CreateMembershipPlanInput {
  name: string;
  description?: string;
  durationDays: number;
  priceMinor: number;
  graceDays: number;
  accessDays: string;
  accessStartMinute: number;
  accessEndMinute: number;
  minAge?: number;
  maxAge?: number;
}

export interface UpdateMembershipPlanInput {
  planId: string;
  name: string;
  description?: string;
  durationDays: number;
  priceMinor: number;
  graceDays: number;
  accessDays: string;
  accessStartMinute: number;
  accessEndMinute: number;
  minAge?: number;
  maxAge?: number;
  active: boolean;
}

export interface CreateProspectInput {
  name: string;
  phone?: string;
  email?: string;
  source?: string;
  notes?: string;
}

export interface UpdateProspectInput {
  prospectId: string;
  name: string;
  phone?: string;
  email?: string;
  source?: string;
  status: ProspectStatus;
  notes?: string;
}

export interface CreateMembershipInput {
  memberId: string;
  planId: string;
  startDate?: string;
}

export interface RenewMembershipInput {
  membershipId: string;
  planId?: string;
}

export interface PauseMembershipInput {
  membershipId: string;
  untilDate: string;
}

export interface ResumeMembershipInput {
  membershipId: string;
}

export interface CancelMembershipInput {
  membershipId: string;
  reason: string;
}

export interface RecordMembershipPaymentInput {
  chargeId: string;
  amountMinor: number;
  paymentMethod: PaymentMethod;
  reference?: string;
  idempotencyKey: string;
}

export interface SaleItemInput {
  variantId: string;
  quantity: number;
}

export interface CreateSaleInput {
  items: SaleItemInput[];
  paymentMethod: PaymentMethod;
  cashReceivedMinor?: number;
  paymentReference?: string;
  idempotencyKey: string;
}

export interface SaleReturnItemInput {
  saleItemId: string;
  quantity: number;
}

export interface CreateSaleReturnInput {
  saleId: string;
  reason: string;
  items: SaleReturnItemInput[];
  paymentReference?: string;
  idempotencyKey: string;
}

export interface SaleReturnItem {
  id: string;
  saleItemId: string;
  productName: string;
  variantName: string;
  sku: string;
  quantity: number;
  amountMinor: number;
}

export interface SaleReturn {
  id: string;
  saleId: string;
  cashSessionId: string;
  currency: string;
  refundMethod: PaymentMethod;
  totalMinor: number;
  reason: string;
  authorizedBy: string | null;
  authorizedByName: string | null;
  paymentReference: string | null;
  createdAt: string;
  items: SaleReturnItem[];
}

export interface PaymentCharge {
  id: string;
  saleId: string;
  saleReturnId: string | null;
  kind: "charge" | "refund";
  method: PaymentMethod;
  provider: "manual" | "point";
  status: "approved";
  amountMinor: number;
  currency: string;
  externalReference: string | null;
  createdAt: string;
}

export interface ListPaymentChargesInput {
  saleId?: string;
  externalReference?: string;
  limit?: number;
}

export interface FinancialReportInput {
  from: string;
  to: string;
}

export interface PaymentMethodTotal {
  paymentMethod: PaymentMethod;
  salesCount: number;
  grossSalesMinor: number;
  refundsCount: number;
  refundedMinor: number;
  netSalesMinor: number;
}

export interface ProductSalesTotal {
  variantId: string;
  productName: string;
  variantName: string;
  sku: string;
  unit: CatalogUnit;
  soldUnits: number;
  refundedUnits: number;
  netUnits: number;
  grossSalesMinor: number;
  refundedMinor: number;
  netSalesMinor: number;
  costMinor: number;
  marginMinor: number;
}

export interface InventoryAdjustmentTotal {
  variantId: string;
  productName: string;
  variantName: string;
  sku: string;
  unitsDelta: number;
  costMinor: number;
}

export interface FinancialReport {
  from: string;
  to: string;
  generatedAt: string;
  currency: string;
  salesCount: number;
  grossSalesMinor: number;
  taxMinor: number;
  refundsCount: number;
  refundedMinor: number;
  netSalesMinor: number;
  averageSaleMinor: number;
  attachRateBps: number;
  itemsPerSaleMilli: number;
  costOfGoodsMinor: number;
  marginMinor: number;
  marginBps: number;
  inventoryValueStartMinor: number;
  inventoryValueEndMinor: number;
  inventoryTurnoverBps: number;
  adjustmentsUnits: number;
  adjustmentsCostMinor: number;
  paymentMethods: PaymentMethodTotal[];
  cashSessionsCount: number;
  openCashSessions: number;
  closedCashSessions: number;
  cashDifferenceMinor: number;
  cashDifferenceAbsMinor: number;
  productSales: ProductSalesTotal[];
  inventoryAdjustments: InventoryAdjustmentTotal[];
}

export interface NewReportScheduleInput {
  cadence: "daily" | "weekly" | "monthly";
  lookbackDays: number;
}

export interface ReportSchedule {
  id: string;
  reportType: string;
  cadence: string;
  lookbackDays: number;
  enabled: boolean;
  nextRunAt: string;
  lastRunAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface SetReportScheduleEnabledInput {
  scheduleId: string;
  enabled: boolean;
}

export interface ReportRun {
  id: string;
  scheduleId: string | null;
  reportType: string;
  periodFrom: string;
  periodTo: string;
  generatedAt: string;
  fileName: string | null;
  checksum: string;
  status: string;
  error: string | null;
}

export interface FinancialReportExport {
  fileName: string;
  path: string;
  bytes: number;
}

export interface GymSettings {
  id: string;
  businessName: string;
  legalName: string | null;
  taxId: string | null;
  currency: string;
  timezone: string;
  locale: string;
  openingTime: string;
  closingTime: string;
  slotMinutes: number;
  graceMinutes: number;
  inactivityDays: number;
  frozenDays: number;
  updatedAt: string;
}

export interface UpdateGymSettingsInput {
  businessName: string;
  legalName?: string | null;
  taxId?: string | null;
  currency: string;
  timezone: string;
  locale: string;
  openingTime: string;
  closingTime: string;
  slotMinutes: number;
  graceMinutes: number;
  inactivityDays: number;
  frozenDays: number;
}

export interface IntegrationLink {
  id: string;
  moduleId: string;
  displayName: string;
  contractVersion: string;
  status: string;
  capabilities: string[];
  lastSeenAt: string | null;
  lastEventAt: string | null;
  lastSnapshotAt: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface UpsertIntegrationLinkInput {
  moduleId: string;
  displayName: string;
  contractVersion: string;
  status: string;
  capabilities?: string[];
  notes?: string | null;
}

export interface IntegrationJournalSummary {
  pending: number;
  sent: number;
  deadLetter: number;
  inboxEvents: number;
  lastServerSequence: number;
}

export interface ClassSession {
  id: string;
  name: string;
  coachUserId: string | null;
  coachName: string | null;
  weekday: number;
  startTime: string;
  durationMinutes: number;
  capacity: number;
  room: string | null;
  status: string;
  notes: string | null;
  booked: number;
  attended: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateClassInput {
  name: string;
  coachUserId?: string | null;
  weekday: number;
  startTime: string;
  durationMinutes: number;
  capacity: number;
  room?: string | null;
  notes?: string | null;
}

export interface UpdateClassInput {
  classId: string;
  name: string;
  coachUserId?: string | null;
  weekday: number;
  startTime: string;
  durationMinutes: number;
  capacity: number;
  room?: string | null;
  status: string;
  notes?: string | null;
}

export interface ClassBooking {
  id: string;
  classId: string;
  className: string;
  memberId: string;
  memberName: string;
  status: string;
  note: string | null;
  bookedAt: string;
  updatedAt: string;
}

export interface BookClassInput {
  classId: string;
  memberId: string;
  note?: string | null;
}

export interface UpdateClassBookingInput {
  bookingId: string;
  status: string;
}

export interface Campaign {
  id: string;
  name: string;
  channel: string;
  audience: string;
  status: string;
  startsOn: string | null;
  endsOn: string | null;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateCampaignInput {
  name: string;
  channel: string;
  audience: string;
  status: string;
  startsOn?: string | null;
  endsOn?: string | null;
  notes?: string | null;
}

export interface UpdateCampaignInput {
  campaignId: string;
  name: string;
  channel: string;
  audience: string;
  status: string;
  startsOn?: string | null;
  endsOn?: string | null;
  notes?: string | null;
}

export interface FunnelStage {
  status: string;
  count: number;
}

export interface MarketingFunnel {
  prospectsTotal: number;
  prospectsConverted: number;
  conversionBps: number;
  stages: FunnelStage[];
  activeCampaigns: number;
  expiringMemberships: number;
  inactiveMembers: number;
}

export interface Equipment {
  id: string;
  name: string;
  code: string | null;
  area: string | null;
  status: string;
  acquiredOn: string | null;
  notes: string | null;
  openOrders: number;
  createdAt: string;
  updatedAt: string;
}

export interface CreateEquipmentInput {
  name: string;
  code?: string | null;
  area?: string | null;
  acquiredOn?: string | null;
  notes?: string | null;
}

export interface UpdateEquipmentInput {
  equipmentId: string;
  name: string;
  code?: string | null;
  area?: string | null;
  status: string;
  acquiredOn?: string | null;
  notes?: string | null;
}

export interface MaintenanceOrder {
  id: string;
  equipmentId: string | null;
  equipmentName: string | null;
  kind: string;
  status: string;
  priority: string;
  summary: string;
  detail: string | null;
  dueOn: string | null;
  reportedBy: string | null;
  completedAt: string | null;
  createdAt: string;
  updatedAt: string;
}

export interface CreateMaintenanceOrderInput {
  equipmentId?: string | null;
  kind: string;
  priority: string;
  summary: string;
  detail?: string | null;
  dueOn?: string | null;
  reportedBy?: string | null;
}

export interface UpdateMaintenanceOrderInput {
  orderId: string;
  status: string;
  priority: string;
  summary: string;
  detail?: string | null;
  dueOn?: string | null;
}

export interface SaleItem {
  id: string;
  variantId: string;
  priceVersionId: string;
  productName: string;
  variantName: string;
  sku: string;
  unit: CatalogUnit;
  quantity: number;
  unitAmountMinor: number;
  taxRateBps: number;
  priceIncludesTax: boolean;
  subtotalMinor: number;
  taxMinor: number;
  totalMinor: number;
  refundedQuantity: number;
  refundedMinor: number;
}

export interface Sale {
  id: string;
  currency: string;
  subtotalMinor: number;
  taxMinor: number;
  totalMinor: number;
  refundedMinor: number;
  paymentMethod: PaymentMethod;
  cashReceivedMinor: number | null;
  changeMinor: number | null;
  soldBy: string | null;
  soldByName: string | null;
  cashSessionId: string | null;
  paymentReference: string | null;
  items: SaleItem[];
  createdAt: string;
}

export interface InventoryItem {
  variantId: string;
  productName: string;
  variantName: string;
  sku: string;
  unit: CatalogUnit;
  stock: number;
  valueMinor: number;
  avgUnitCostMinor: number;
  minStock: number;
  lowStock: boolean;
}

export interface InventoryMovement {
  id: string;
  variantId: string;
  movementType: "entry" | "sale" | "return" | "adjustment";
  quantityDelta: number;
  unitCostMinor: number;
  totalCostMinor: number;
  sourceType: "manual" | "sale" | "sale_return" | "adjustment";
  sourceId: string | null;
  reference: string | null;
  note: string | null;
  authorizedBy: string | null;
  authorizedByName: string | null;
  createdAt: string;
}

export interface RecordInventoryEntryInput {
  variantId: string;
  quantity: number;
  unitCostMinor: number;
  reference?: string;
  note?: string;
}

export interface AdjustInventoryInput {
  variantId: string;
  quantityDelta: number;
  unitCostMinor?: number;
  reason: string;
}

export interface UpdateInventoryThresholdInput {
  variantId: string;
  minStock: number;
}

export type CashSessionStatus = "open" | "closed";

export interface CashSession {
  id: string;
  status: CashSessionStatus;
  currency: string;
  openingAmountMinor: number;
  expectedCashMinor: number;
  cashSalesMinor: number;
  salesCount: number;
  cashIncomeMinor: number;
  incomeMinor: number;
  cashExpenseMinor: number;
  expenseMinor: number;
  cashRefundsMinor: number;
  refundedMinor: number;
  refundsCount: number;
  movementsCount: number;
  countedAmountMinor: number | null;
  differenceMinor: number | null;
  differenceReason: string | null;
  openedBy: string | null;
  openedByName: string | null;
  openedAt: string;
  closedBy: string | null;
  closedByName: string | null;
  closedAt: string | null;
}

export type CashMovementType = "income" | "expense";

export interface CashMovement {
  id: string;
  cashSessionId: string;
  movementType: CashMovementType;
  category: string;
  amountMinor: number;
  paymentMethod: PaymentMethod;
  evidence: string;
  authorizedBy: string | null;
  authorizedByName: string | null;
  createdAt: string;
}

export interface OpenCashSessionInput {
  openingAmountMinor: number;
}

export interface CloseCashSessionInput {
  sessionId: string;
  countedAmountMinor: number;
  differenceReason: string | null;
}

export interface RecordCashMovementInput {
  sessionId: string;
  movementType: CashMovementType;
  category: string;
  amountMinor: number;
  paymentMethod: PaymentMethod;
  evidence: string;
}

export type AuditResult = "allowed" | "denied" | "failed";

export interface AuditEventSummary {
  id: string;
  sequence: number;
  actorId: string | null;
  actorName: string | null;
  action: string;
  resource: string;
  result: AuditResult;
  origin: string;
  correlationId: string | null;
  createdAt: string;
}

export interface AuditIntegrity {
  valid: boolean;
  checkedEvents: number;
  firstBrokenSequence: number | null;
}

export interface BackupSummary {
  id: string;
  fileName: string;
  createdAt: string;
  sizeBytes: number;
  checksum: string;
  schemaVersion: number;
}

export interface RestoreOutcome {
  backupId: string;
  restoredAt: string;
  schemaVersion: number;
  integrityOk: boolean;
}

export type ErrorCode =
  | "invalidRequest"
  | "notFound"
  | "storage"
  | "backup"
  | "unauthorized"
  | "internal";

export interface ApiError {
  code: ErrorCode;
  message: string;
  details?: string;
}

export function isApiError(value: unknown): value is ApiError {
  if (typeof value !== "object" || value === null) {
    return false;
  }
  const candidate = value as Record<string, unknown>;
  return typeof candidate.code === "string" && typeof candidate.message === "string";
}

export function errorMessage(error: unknown): string {
  if (isApiError(error)) {
    return error.details ? `${error.message} (${error.details})` : error.message;
  }
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}
