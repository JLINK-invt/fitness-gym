import type {
  Campaign,
  ClassBooking,
  ClassSession,
  Equipment,
  GymSettings,
  IntegrationLink,
  MaintenanceOrder,
  MarketingFunnel,
  Permission,
  PermissionEffect,
  StaffRole,
} from "@gym/contracts";

import {
  ownerOnlyPermissions,
  permissionLabels,
  rolePermissions,
} from "./permissions";

type Organization = { id: string; name: string };
type Owner = { id: string; name: string; password: string; recoveryCode: string };
type Session = {
  userId: string;
  organizationId: string;
  organizationName: string;
  name: string;
  role: StaffRole;
  permissions: Permission[];
};
type AccessEvent = {
  eventId: string;
  allowed: boolean;
  idempotencyKey: string;
  occurredAt: string;
};
type AccessEntry = { event: AccessEvent; requestHash: string };
type StoredCredential = {
  id: string;
  memberId: string;
  memberNumber: string;
  memberName: string;
  code: string;
  status: "active" | "revoked";
  revision: number;
  issuedAt: string;
  revokedAt: string | null;
};
type StoredAccessLog = {
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
};
type StoredSnapshot = {
  revision: number;
  generatedAt: string;
  memberCount: number;
  credentialCount: number;
  checksum: string;
  signature: string;
  payloadJson: string;
};
type StoredReportSchedule = {
  id: string;
  reportType: string;
  cadence: "daily" | "weekly" | "monthly";
  lookbackDays: number;
  enabled: boolean;
  nextRunAt: string;
  lastRunAt: string | null;
  createdAt: string;
  updatedAt: string;
};
type StoredReportRun = {
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
};
type StaffMember = {
  id: string;
  name: string;
  password: string;
  role: StaffRole;
  status: "active" | "revoked";
  lastLoginAt: string | null;
  createdAt: string;
  permissionOverrides: { permission: Permission; effect: PermissionEffect }[];
};
type Backup = {
  id: string;
  fileName: string;
  createdAt: string;
  sizeBytes: number;
  checksum: string;
  schemaVersion: number;
};
type AuditEvent = {
  id: string;
  sequence: number;
  actorId: string | null;
  actorName: string | null;
  action: string;
  resource: string;
  result: "allowed" | "denied" | "failed";
  origin: string;
  correlationId: string | null;
  createdAt: string;
};

type CatalogPrice = {
  id: string;
  currency: string;
  amountMinor: number;
  taxRateBps: number;
  priceIncludesTax: boolean;
  effectiveFrom: string;
  createdAt: string;
};
type StoredVariant = {
  id: string;
  name: string;
  sku: string;
  barcode: string | null;
  unit: "piece" | "service" | "session" | "package";
  costMinor: number;
  status: "active" | "archived";
  createdAt: string;
  updatedAt: string;
};
type StoredProduct = {
  id: string;
  name: string;
  category: string | null;
  status: "active" | "archived";
  variants: StoredVariant[];
  createdAt: string;
  updatedAt: string;
};
type StoredPrice = CatalogPrice & { variantId: string };
type StoredSaleItem = {
  id: string;
  variantId: string;
  priceVersionId: string;
  productName: string;
  variantName: string;
  sku: string;
  unit: StoredVariant["unit"];
  quantity: number;
  unitAmountMinor: number;
  taxRateBps: number;
  priceIncludesTax: boolean;
  subtotalMinor: number;
  taxMinor: number;
  totalMinor: number;
  refundedQuantity: number;
  refundedMinor: number;
};
type StoredSaleAttempt = {
  requestHash: string;
  saleId: string;
};
type StoredReturnAttempt = {
  requestHash: string;
  returnId: string;
};
type StoredPaymentCharge = {
  id: string;
  saleId: string;
  saleReturnId: string | null;
  kind: "charge" | "refund";
  method: "cash" | "transfer" | "point";
  provider: "manual" | "point";
  status: "approved";
  amountMinor: number;
  currency: string;
  externalReference: string | null;
  createdAt: string;
};

type StoredSale = {
  id: string;
  currency: string;
  subtotalMinor: number;
  taxMinor: number;
  totalMinor: number;
  refundedMinor: number;
  paymentMethod: "cash" | "transfer" | "point";
  cashReceivedMinor: number | null;
  changeMinor: number | null;
  soldBy: string | null;
  soldByName: string | null;
  cashSessionId: string | null;
  paymentReference: string | null;
  items: StoredSaleItem[];
  createdAt: string;
};
type StoredSaleReturnItem = {
  id: string;
  saleItemId: string;
  productName: string;
  variantName: string;
  sku: string;
  quantity: number;
  amountMinor: number;
};
type StoredSaleReturn = {
  id: string;
  saleId: string;
  cashSessionId: string;
  currency: string;
  refundMethod: "cash" | "transfer" | "point";
  totalMinor: number;
  reason: string;
  authorizedBy: string | null;
  authorizedByName: string | null;
  paymentReference: string | null;
  createdAt: string;
  items: StoredSaleReturnItem[];
};
type StoredInventoryMovement = {
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
};
type StoredCashMovement = {
  id: string;
  cashSessionId: string;
  movementType: "income" | "expense";
  category: string;
  amountMinor: number;
  paymentMethod: "cash" | "transfer" | "point";
  evidence: string;
  authorizedBy: string | null;
  authorizedByName: string | null;
  createdAt: string;
};
type StoredCashSession = {
  id: string;
  status: "open" | "closed";
  currency: string;
  openingAmountMinor: number;
  countedAmountMinor: number | null;
  differenceMinor: number | null;
  differenceReason: string | null;
  openedBy: string | null;
  openedByName: string | null;
  openedAt: string;
  closedBy: string | null;
  closedByName: string | null;
  closedAt: string | null;
};

type StoredMembershipPlan = {
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
};
type StoredMember = {
  id: string;
  memberNumber: string;
  firstName: string;
  lastName: string;
  phone: string | null;
  email: string | null;
  birthDate: string | null;
  status: "active" | "inactive" | "archived";
  joinedAt: string;
  notes: string | null;
  createdAt: string;
  updatedAt: string;
};
type StoredProspect = {
  id: string;
  name: string;
  phone: string | null;
  email: string | null;
  source: string | null;
  status: "new" | "contacted" | "trial" | "converted" | "lost";
  notes: string | null;
  convertedMemberId: string | null;
  createdAt: string;
  updatedAt: string;
};
type StoredMembership = {
  id: string;
  memberId: string;
  planId: string;
  planName: string;
  planVersion: number;
  planVersionId: string;
  status: "scheduled" | "active" | "paused" | "expired" | "cancelled";
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
};
type StoredMembershipCharge = {
  id: string;
  membershipId: string;
  memberId: string;
  description: string;
  amountMinor: number;
  currency: string;
  dueDate: string;
  status: "pending" | "paid" | "void";
  paidMinor: number;
  createdAt: string;
  updatedAt: string;
};
type StoredMembershipPayment = {
  id: string;
  chargeId: string;
  membershipId: string;
  memberId: string;
  amountMinor: number;
  paymentMethod: "cash" | "transfer" | "point";
  reference: string | null;
  receiptNumber: string;
  cashSessionId: string | null;
  receivedBy: string | null;
  receivedByName: string | null;
  createdAt: string;
};
type StoredMembershipPaymentAttempt = {
  requestHash: string;
  paymentId: string;
};

(() => {
  const state: {
    organizations: Organization[];
    owner: Owner | null;
    staff: StaffMember[];
    session: Session | null;
    accessEvents: AccessEvent[];
    accessEntries: Map<string, AccessEntry>;
    credentials: StoredCredential[];
    accessLog: StoredAccessLog[];
    snapshots: StoredSnapshot[];
    reportSchedules: StoredReportSchedule[];
    reportRuns: StoredReportRun[];
    backups: Backup[];
    secrets: Map<string, string>;
    auditEvents: AuditEvent[];
    catalog: StoredProduct[];
    catalogPrices: StoredPrice[];
    sales: StoredSale[];
    saleIdempotency: Map<string, StoredSaleAttempt>;
    saleReturns: StoredSaleReturn[];
    saleReturnIdempotency: Map<string, StoredReturnAttempt>;
    paymentCharges: StoredPaymentCharge[];
    cashSessions: StoredCashSession[];
    cashMovements: StoredCashMovement[];
    inventoryMovements: StoredInventoryMovement[];
    inventoryThresholds: Map<string, number>;
    membershipPlans: StoredMembershipPlan[];
    members: StoredMember[];
    prospects: StoredProspect[];
    memberships: StoredMembership[];
    membershipCharges: StoredMembershipCharge[];
    membershipPayments: StoredMembershipPayment[];
    membershipPaymentIdempotency: Map<string, StoredMembershipPaymentAttempt>;
    membershipReceiptSequence: number;
    memberSequence: number;
    gymSettings: GymSettings | null;
    integrationLinks: IntegrationLink[];
    classes: ClassSession[];
    classBookings: ClassBooking[];
    campaigns: Campaign[];
    equipment: Equipment[];
    maintenanceOrders: MaintenanceOrder[];
  } = {
    organizations: [],
    owner: null,
    staff: [],
    session: null,
    accessEvents: [],
    accessEntries: new Map(),
    credentials: [],
    accessLog: [],
    snapshots: [],
    reportSchedules: [],
    reportRuns: [],
    backups: [],
    secrets: new Map(),
    auditEvents: [],
    catalog: [],
    catalogPrices: [],
    sales: [],
    saleIdempotency: new Map(),
    saleReturns: [],
    saleReturnIdempotency: new Map(),
    paymentCharges: [],
    cashSessions: [],
    cashMovements: [],
    inventoryMovements: [],
    inventoryThresholds: new Map(),
    membershipPlans: [],
    members: [],
    prospects: [],
    memberships: [],
    membershipCharges: [],
    membershipPayments: [],
    membershipPaymentIdempotency: new Map(),
    membershipReceiptSequence: 0,
    memberSequence: 0,
    gymSettings: null,
    integrationLinks: [],
    classes: [],
    classBookings: [],
    campaigns: [],
    equipment: [],
    maintenanceOrders: [],
  };

  const now = () => new Date().toISOString();
  const uuid = () => crypto.randomUUID();
  const apiError = (code: string, message: string) => ({ code, message });
  const recoveryAlphabet = "ABCDEFGHJKLMNPQRSTUVWXYZ23456789";
  const makeRecoveryCode = () => {
    const bytes = crypto.getRandomValues(new Uint8Array(20));
    const code = Array.from(bytes, (byte) => recoveryAlphabet[byte % 32]).join("");
    return code.match(/.{5}/g)!.join("-");
  };
  const normalizeRecoveryCode = (value: string) =>
    value.replace(/[\s-]/g, "").toUpperCase();
  const permissionsFor = (
    role: StaffRole,
    overrides: { permission: Permission; effect: PermissionEffect }[] = [],
  ) => {
    const permissions = new Set<Permission>(rolePermissions[role]);
    for (const override of overrides) {
      if (ownerOnlyPermissions.includes(override.permission)) {
        continue;
      }
      if (override.effect === "allow") {
        permissions.add(override.permission);
      } else {
        permissions.delete(override.permission);
      }
    }
    return [...permissions];
  };
  const requireSession = () => {
    if (!state.session) {
      throw apiError("unauthorized", "Inicia sesión para continuar.");
    }
    return state.session;
  };
  const requirePermission = (permission: Permission) => {
    const session = requireSession();
    if (!session.permissions.includes(permission)) {
      throw apiError("unauthorized", "Tu rol no permite realizar esta acción.");
    }
    return session;
  };
  const requireAnyPermission = (permissions: Permission[]) => {
    const session = requireSession();
    if (!permissions.some((permission) => session.permissions.includes(permission))) {
      throw apiError("unauthorized", "Tu rol no permite realizar esta acción.");
    }
    return session;
  };
  const ownerStaff = () =>
    state.owner
      ? {
          userId: state.owner.id,
          name: state.owner.name,
          role: "owner" as const,
          status: "active" as const,
          lastLoginAt: null,
          createdAt: now(),
          permissionOverrides: [],
        }
      : null;
  const staffSummary = (member: StaffMember) => ({
    userId: member.id,
    name: member.name,
    role: member.role,
    status: member.status,
    lastLoginAt: member.lastLoginAt,
    createdAt: member.createdAt,
    permissionOverrides: member.permissionOverrides,
  });

  const dateOnly = (date = new Date()) => date.toISOString().slice(0, 10);
  const addDays = (value: string, days: number) => {
    const date = new Date(`${value}T00:00:00.000Z`);
    date.setUTCDate(date.getUTCDate() + days);
    return dateOnly(date);
  };
  const memberSnapshot = (member: StoredMember) => {
    const memberships = state.memberships.filter((item) => item.memberId === member.id);
    const active = memberships.find((item) =>
      ["active", "scheduled", "paused"].includes(item.status),
    );
    const debtMinor = state.membershipCharges
      .filter((charge) => charge.memberId === member.id && charge.status === "pending")
      .reduce((total, charge) => total + charge.amountMinor - charge.paidMinor, 0);
    return {
      id: member.id,
      memberNumber: member.memberNumber,
      firstName: member.firstName,
      lastName: member.lastName,
      phone: member.phone,
      email: member.email,
      birthDate: member.birthDate,
      status: member.status,
      joinedAt: member.joinedAt,
      notes: member.notes,
      activeMembership: active ? membershipSnapshot(active) : null,
      debtMinor,
      createdAt: member.createdAt,
      updatedAt: member.updatedAt,
    };
  };
  const membershipSnapshot = (membership: StoredMembership) => {
    const member = state.members.find((item) => item.id === membership.memberId);
    return {
      ...membership,
      memberName: member ? `${member.firstName} ${member.lastName}`.trim() : "Socio",
    };
  };
  const chargeSnapshot = (charge: StoredMembershipCharge) => ({ ...charge });
  const paymentSnapshot = (payment: StoredMembershipPayment) => ({ ...payment });
  const accountSnapshot = (memberId: string) => {
    const charges = state.membershipCharges
      .filter((charge) => charge.memberId === memberId)
      .sort((left, right) => right.dueDate.localeCompare(left.dueDate));
    const payments = state.membershipPayments
      .filter((payment) => payment.memberId === memberId)
      .sort((left, right) => right.createdAt.localeCompare(left.createdAt));
    return {
      memberId,
      currency: "MXN",
      charges: charges.map(chargeSnapshot),
      payments: payments.map(paymentSnapshot),
      balanceMinor: charges
        .filter((charge) => charge.status !== "void")
        .reduce((total, charge) => total + charge.amountMinor - charge.paidMinor, 0),
    };
  };
  const receiptSnapshot = (payment: StoredMembershipPayment) => {
    const charge = state.membershipCharges.find((item) => item.id === payment.chargeId);
    const member = state.members.find((item) => item.id === payment.memberId);
    return {
      paymentId: payment.id,
      receiptNumber: payment.receiptNumber,
      gymName: state.organizations[0]?.name ?? "Fitness Gym",
      memberNumber: member?.memberNumber ?? "",
      memberName: member ? `${member.firstName} ${member.lastName}`.trim() : "Socio",
      chargeDescription: charge?.description ?? "Membresía",
      amountMinor: payment.amountMinor,
      currency: charge?.currency ?? "MXN",
      paymentMethod: payment.paymentMethod,
      reference: payment.reference,
      receivedByName: payment.receivedByName,
      createdAt: payment.createdAt,
    };
  };
  const dashboardSnapshot = () => {
    const today = dateOnly();
    const monthStart = `${today.slice(0, 7)}-01`;
    const nextMonth = new Date(`${monthStart}T00:00:00.000Z`);
    nextMonth.setUTCMonth(nextMonth.getUTCMonth() + 1);
    const nextMonthStart = dateOnly(nextMonth);
    const soon = addDays(today, 7);
    const inMonth = (value: string) => value >= monthStart && value < nextMonthStart;
    const trend = Array.from({ length: 6 }, (_, index) => {
      const date = new Date(`${monthStart}T00:00:00.000Z`);
      date.setUTCMonth(date.getUTCMonth() - (5 - index));
      const month = date.toISOString().slice(0, 7);
      const start = `${month}-01`;
      const endDate = new Date(`${start}T00:00:00.000Z`);
      endDate.setUTCMonth(endDate.getUTCMonth() + 1);
      const end = dateOnly(endDate);
      return {
        month,
        newMembers: state.members.filter((member) => member.joinedAt >= start && member.joinedAt < end).length,
        renewals: 0,
        cancellations: 0,
        incomeMinor: state.membershipPayments
          .filter((payment) => payment.createdAt >= start && payment.createdAt < end)
          .reduce((total, payment) => total + payment.amountMinor, 0),
      };
    });
    return {
      periodStart: monthStart,
      periodEnd: addDays(nextMonthStart, -1),
      timezone: "UTC",
      activeMembers: state.members.filter((member) => member.status === "active").length,
      activeMemberships: state.memberships.filter((membership) => membership.status === "active").length,
      expiringSoon: state.memberships.filter(
        (membership) =>
          membership.status === "active" &&
          membership.endDate >= today &&
          membership.endDate <= soon,
      ).length,
      pastDue: new Set(
        state.membershipCharges
          .filter((charge) => charge.status === "pending" && charge.dueDate < today)
          .map((charge) => charge.memberId),
      ).size,
      debtMinor: state.membershipCharges
        .filter((charge) => charge.status === "pending")
        .reduce((total, charge) => total + charge.amountMinor - charge.paidMinor, 0),
      newMembersThisMonth: state.members.filter((member) => inMonth(member.joinedAt)).length,
      incomeThisMonthMinor: state.membershipPayments
        .filter((payment) => inMonth(payment.createdAt))
        .reduce((total, payment) => total + payment.amountMinor, 0),
      renewalsThisMonth: 0,
      cancellationsThisMonth: state.memberships.filter(
        (membership) => membership.status === "cancelled" && inMonth(membership.updatedAt),
      ).length,
      trend,
      recentPayments: state.membershipPayments
        .slice()
        .sort((left, right) => right.createdAt.localeCompare(left.createdAt))
        .slice(0, 5)
        .map(receiptSnapshot),
      expiringMemberships: state.memberships
        .filter(
          (membership) =>
            membership.status === "active" &&
            membership.endDate >= today &&
            membership.endDate <= soon,
        )
        .sort((left, right) => left.endDate.localeCompare(right.endDate))
        .slice(0, 5)
        .map(membershipSnapshot),
    };
  };
  const normalizeAccessDays = (value: string) => {
    const allowed = ["mon", "tue", "wed", "thu", "fri", "sat", "sun"];
    const days = value
      .split(",")
      .map((day) => day.trim().toLowerCase())
      .filter((day) => allowed.includes(day));
    if (days.length === 0) {
      throw apiError("invalidRequest", "El plan debe permitir al menos un día.");
    }
    return [...new Set(days)].join(",");
  };
  const optionalAge = (value: unknown) => {
    if (value === null || value === undefined || value === "") {
      return null;
    }
    const age = Number(value);
    if (!Number.isInteger(age) || age < 0 || age > 120) {
      throw apiError("invalidRequest", "La edad debe estar entre 0 y 120.");
    }
    return age;
  };
  const validatePlanAccess = (
    _accessDays: string,
    startMinute: number,
    endMinute: number,
    minAge: number | null,
    maxAge: number | null,
  ) => {
    if (!Number.isInteger(startMinute) || startMinute < 0 || startMinute > 1439) {
      throw apiError("invalidRequest", "La hora de inicio no es válida.");
    }
    if (!Number.isInteger(endMinute) || endMinute < 1 || endMinute > 1440) {
      throw apiError("invalidRequest", "La hora de fin no es válida.");
    }
    if (startMinute >= endMinute) {
      throw apiError("invalidRequest", "La hora de inicio debe ser anterior a la de fin.");
    }
    if (minAge !== null && maxAge !== null && maxAge < minAge) {
      throw apiError("invalidRequest", "La edad máxima no puede ser menor a la mínima.");
    }
  };
  const memberAgeOn = (birthDate: string, at: string) => {
    const birth = new Date(`${birthDate}T00:00:00.000Z`);
    const date = new Date(`${at}T00:00:00.000Z`);
    if (Number.isNaN(birth.getTime()) || birth > date) {
      throw apiError("invalidRequest", "La fecha de nacimiento no es válida.");
    }
    let age = date.getUTCFullYear() - birth.getUTCFullYear();
    if (
      date.getUTCMonth() < birth.getUTCMonth() ||
      (date.getUTCMonth() === birth.getUTCMonth() && date.getUTCDate() < birth.getUTCDate())
    ) {
      age -= 1;
    }
    return age;
  };
  const ensurePlanEligibility = (plan: StoredMembershipPlan, member: StoredMember, start: string) => {
    if (plan.minAge === null && plan.maxAge === null) {
      return;
    }
    if (!member.birthDate) {
      throw apiError("invalidRequest", "El socio necesita fecha de nacimiento para este plan.");
    }
    const age = memberAgeOn(member.birthDate, start);
    if (plan.minAge !== null && age < plan.minAge) {
      throw apiError("invalidRequest", "El socio no cumple la edad mínima del plan.");
    }
    if (plan.maxAge !== null && age > plan.maxAge) {
      throw apiError("invalidRequest", "El socio supera la edad máxima del plan.");
    }
  };
  const requireManageMembers = () => requirePermission("manageMembers");

  const recordAudit = (input: {
    actor?: Session | null;
    action: string;
    resource: string;
    result?: "allowed" | "denied" | "failed";
    correlationId?: string | null;
  }) => {
    if (state.organizations.length === 0) {
      return;
    }
    state.auditEvents.unshift({
      id: uuid(),
      sequence: state.auditEvents.length + 1,
      actorId: input.actor?.userId ?? null,
      actorName: input.actor?.name ?? null,
      action: input.action,
      resource: input.resource,
      result: input.result ?? "allowed",
      origin: "tauri",
      correlationId: input.correlationId ?? null,
      createdAt: now(),
    });
  };

  const health = () => ({
    service: "gym-desktop",
    version: "0.1.0",
    contractVersion: "3.1.0",
    status: "ok",
    modules: [
      { name: "gym-core", status: "ok" },
      { name: "gym-data", status: "ok" },
      { name: "gym-backup", status: "ok" },
    ],
    database: {
      path: "/tmp/fitness-gym-e2e/fitness-gym.db",
      encrypted: true,
      cipherVersion: "4.6.1 community",
      schemaVersion: 21,
      integrityOk: true,
      organizations: state.organizations.length,
    },
    checkedAt: now(),
  });

  const cashSessionSnapshot = (session: StoredCashSession) => {
    const sales = state.sales.filter((sale) => sale.cashSessionId === session.id);
    const movements = state.cashMovements.filter(
      (movement) => movement.cashSessionId === session.id,
    );
    const refunds = state.saleReturns.filter(
      (saleReturn) => saleReturn.cashSessionId === session.id,
    );
    const cashSalesMinor = sales
      .filter((sale) => sale.paymentMethod === "cash")
      .reduce((total, sale) => total + sale.totalMinor, 0);
    const cashIncomeMinor = movements
      .filter(
        (movement) =>
          movement.movementType === "income" && movement.paymentMethod === "cash",
      )
      .reduce((total, movement) => total + movement.amountMinor, 0);
    const incomeMinor = movements
      .filter((movement) => movement.movementType === "income")
      .reduce((total, movement) => total + movement.amountMinor, 0);
    const cashExpenseMinor = movements
      .filter(
        (movement) =>
          movement.movementType === "expense" && movement.paymentMethod === "cash",
      )
      .reduce((total, movement) => total + movement.amountMinor, 0);
    const expenseMinor = movements
      .filter((movement) => movement.movementType === "expense")
      .reduce((total, movement) => total + movement.amountMinor, 0);
    const cashRefundsMinor = refunds
      .filter((saleReturn) => saleReturn.refundMethod === "cash")
      .reduce((total, saleReturn) => total + saleReturn.totalMinor, 0);
    const refundedMinor = refunds.reduce(
      (total, saleReturn) => total + saleReturn.totalMinor,
      0,
    );
    return {
      ...session,
      expectedCashMinor:
        session.openingAmountMinor +
        cashSalesMinor +
        cashIncomeMinor -
        cashExpenseMinor -
        cashRefundsMinor,
      cashSalesMinor,
      salesCount: sales.length,
      cashIncomeMinor,
      incomeMinor,
      cashExpenseMinor,
      expenseMinor,
      cashRefundsMinor,
      refundedMinor,
      refundsCount: refunds.length,
      movementsCount: movements.length,
    };
  };
  const openCashSession = () =>
    state.cashSessions.find((session) => session.status === "open") ?? null;

  const averageCostFor = (variantId: string) => {
    const movements = state.inventoryMovements.filter(
      (movement) => movement.variantId === variantId,
    );
    const quantity = movements.reduce(
      (total, movement) => total + movement.quantityDelta,
      0,
    );
    const value = movements.reduce(
      (total, movement) => total + movement.totalCostMinor,
      0,
    );
    return quantity > 0 ? Math.round(value / quantity) : 0;
  };
  const inventorySnapshot = () =>
    allVariants()
      .filter(({ variant }) => variant.unit === "piece")
      .map(({ product, variant }) => {
        const movements = state.inventoryMovements.filter(
          (movement) => movement.variantId === variant.id,
        );
        const stock = movements.reduce(
          (total, movement) => total + movement.quantityDelta,
          0,
        );
        const valueMinor = movements.reduce(
          (total, movement) => total + movement.totalCostMinor,
          0,
        );
        const minStock = state.inventoryThresholds.get(variant.id) ?? 0;
        return {
          variantId: variant.id,
          productName: product.name,
          variantName: variant.name,
          sku: variant.sku,
          unit: variant.unit,
          stock,
          valueMinor,
          avgUnitCostMinor: stock > 0 ? Math.round(valueMinor / stock) : 0,
          minStock,
          lowStock: stock <= minStock,
        };
      })
      .sort((left, right) => left.productName.localeCompare(right.productName));
  const inventoryMovementSnapshot = (movement: StoredInventoryMovement) => ({
    id: movement.id,
    variantId: movement.variantId,
    movementType: movement.movementType,
    quantityDelta: movement.quantityDelta,
    unitCostMinor: movement.unitCostMinor,
    totalCostMinor: movement.totalCostMinor,
    sourceType: movement.sourceType,
    sourceId: movement.sourceId,
    reference: movement.reference,
    note: movement.note,
    authorizedBy: movement.authorizedBy,
    authorizedByName: movement.authorizedByName,
    createdAt: movement.createdAt,
  });
  const postInventoryMovement = (input: {
    variantId: string;
    movementType: StoredInventoryMovement["movementType"];
    quantityDelta: number;
    unitCostMinor: number;
    sourceType: StoredInventoryMovement["sourceType"];
    sourceId?: string | null;
    reference?: string | null;
    note?: string | null;
    authorizedBy?: string | null;
    authorizedByName?: string | null;
  }) => {
    const movement: StoredInventoryMovement = {
      id: uuid(),
      variantId: input.variantId,
      movementType: input.movementType,
      quantityDelta: input.quantityDelta,
      unitCostMinor: input.unitCostMinor,
      totalCostMinor: input.quantityDelta * input.unitCostMinor,
      sourceType: input.sourceType,
      sourceId: input.sourceId ?? null,
      reference: input.reference ?? null,
      note: input.note ?? null,
      authorizedBy: input.authorizedBy ?? null,
      authorizedByName: input.authorizedByName ?? null,
      createdAt: now(),
    };
    state.inventoryMovements.unshift(movement);
    return movement;
  };

  const normalizeReportRange = (from: string, to: string) => {
    const fromMs = Date.parse(from);
    const toMs = Date.parse(to);
    if (!Number.isFinite(fromMs) || !Number.isFinite(toMs)) {
      throw apiError("invalidRequest", "El rango del reporte no es válido.");
    }
    if (fromMs >= toMs) {
      throw apiError(
        "invalidRequest",
        "La fecha inicial debe ser anterior a la fecha final.",
      );
    }
    if (toMs - fromMs > 366 * 24 * 60 * 60 * 1_000) {
      throw apiError("invalidRequest", "El reporte admite como máximo 366 días.");
    }
    return { fromMs, toMs };
  };

  const financialReportSnapshot = (from: string, to: string) => {
    const { fromMs, toMs } = normalizeReportRange(from, to);
    const inRange = (value: string) => {
      const timestamp = Date.parse(value);
      return timestamp >= fromMs && timestamp < toMs;
    };
    const sales = state.sales.filter((sale) => inRange(sale.createdAt));
    const returns = state.saleReturns.filter((saleReturn) =>
      inRange(saleReturn.createdAt),
    );
    const grossSalesMinor = sales.reduce((total, sale) => total + sale.totalMinor, 0);
    const taxMinor = sales.reduce((total, sale) => total + sale.taxMinor, 0);
    const refundedMinor = returns.reduce(
      (total, saleReturn) => total + saleReturn.totalMinor,
      0,
    );
    const paymentMethods = (["cash", "transfer", "point"] as const).map((method) => {
      const methodSales = sales.filter((sale) => sale.paymentMethod === method);
      const methodReturns = returns.filter(
        (saleReturn) => saleReturn.refundMethod === method,
      );
      const gross = methodSales.reduce((total, sale) => total + sale.totalMinor, 0);
      const refunded = methodReturns.reduce(
        (total, saleReturn) => total + saleReturn.totalMinor,
        0,
      );
      return {
        paymentMethod: method,
        salesCount: methodSales.length,
        grossSalesMinor: gross,
        refundsCount: methodReturns.length,
        refundedMinor: refunded,
        netSalesMinor: gross - refunded,
      };
    });

    const saleItemLookup = new Map(
      state.sales.flatMap((sale) =>
        sale.items.map((item) => [item.id, item] as const),
      ),
    );
    const products = new Map<
      string,
      {
        variantId: string;
        productName: string;
        variantName: string;
        sku: string;
        unit: StoredVariant["unit"];
        soldUnits: number;
        refundedUnits: number;
        grossSalesMinor: number;
        refundedMinor: number;
        costMinor: number;
      }
    >();
    for (const sale of sales) {
      for (const item of sale.items) {
        const entry = products.get(item.variantId) ?? {
          variantId: item.variantId,
          productName: item.productName,
          variantName: item.variantName,
          sku: item.sku,
          unit: item.unit,
          soldUnits: 0,
          refundedUnits: 0,
          grossSalesMinor: 0,
          refundedMinor: 0,
          costMinor: 0,
        };
        entry.soldUnits += item.quantity;
        entry.grossSalesMinor += item.totalMinor;
        products.set(item.variantId, entry);
      }
    }
    for (const saleReturn of returns) {
      for (const returned of saleReturn.items) {
        const original = saleItemLookup.get(returned.saleItemId);
        if (!original) {
          continue;
        }
        const entry = products.get(original.variantId) ?? {
          variantId: original.variantId,
          productName: original.productName,
          variantName: original.variantName,
          sku: original.sku,
          unit: original.unit,
          soldUnits: 0,
          refundedUnits: 0,
          grossSalesMinor: 0,
          refundedMinor: 0,
          costMinor: 0,
        };
        entry.refundedUnits += returned.quantity;
        entry.refundedMinor += returned.amountMinor;
        products.set(original.variantId, entry);
      }
    }
    for (const [variantId, entry] of products) {
      const movements = state.inventoryMovements.filter(
        (movement) =>
          movement.variantId === variantId &&
          (movement.movementType === "sale" || movement.movementType === "return") &&
          inRange(movement.createdAt),
      );
      entry.costMinor = -movements.reduce(
        (total, movement) => total + movement.totalCostMinor,
        0,
      );
    }
    const productSales = [...products.values()]
      .map((entry) => ({
        ...entry,
        netUnits: entry.soldUnits - entry.refundedUnits,
        netSalesMinor: entry.grossSalesMinor - entry.refundedMinor,
        marginMinor:
          entry.grossSalesMinor - entry.refundedMinor - entry.costMinor,
      }))
      .sort(
        (left, right) =>
          right.netSalesMinor - left.netSalesMinor ||
          right.netUnits - left.netUnits ||
          left.productName.localeCompare(right.productName),
      );

    const variants = allVariants();
    const adjustments = new Map<
      string,
      {
        variantId: string;
        productName: string;
        variantName: string;
        sku: string;
        unitsDelta: number;
        costMinor: number;
      }
    >();
    for (const movement of state.inventoryMovements) {
      if (movement.movementType !== "adjustment" || !inRange(movement.createdAt)) {
        continue;
      }
      const match = variants.find((entry) => entry.variant.id === movement.variantId);
      if (!match) {
        continue;
      }
      const entry = adjustments.get(movement.variantId) ?? {
        variantId: movement.variantId,
        productName: match.product.name,
        variantName: match.variant.name,
        sku: match.variant.sku,
        unitsDelta: 0,
        costMinor: 0,
      };
      entry.unitsDelta += movement.quantityDelta;
      entry.costMinor += movement.totalCostMinor;
      adjustments.set(movement.variantId, entry);
    }
    const inventoryAdjustments = [...adjustments.values()].sort((left, right) =>
      left.productName.localeCompare(right.productName),
    );

    const cashSessions = state.cashSessions.filter((session) =>
      inRange(session.openedAt),
    );
    const closedCashSessions = cashSessions.filter(
      (session) => session.status === "closed",
    );
    const cashDifferenceMinor = closedCashSessions.reduce(
      (total, session) => total + (session.differenceMinor ?? 0),
      0,
    );
    const multiItemSales = sales.filter((sale) => sale.items.length >= 2).length;
    const soldUnits = sales.reduce(
      (total, sale) =>
        total + sale.items.reduce((units, item) => units + item.quantity, 0),
      0,
    );
    const costOfGoodsMinor = productSales.reduce(
      (total, product) => total + product.costMinor,
      0,
    );
    const netSalesMinor = grossSalesMinor - refundedMinor;
    const marginMinor = netSalesMinor - costOfGoodsMinor;
    const marginBps =
      netSalesMinor === 0
        ? 0
        : Math.trunc((marginMinor * 10_000) / netSalesMinor);
    const attachRateBps =
      sales.length === 0
        ? 0
        : Math.round((multiItemSales * 10_000) / sales.length);
    const itemsPerSaleMilli =
      sales.length === 0 ? 0 : Math.round((soldUnits * 1_000) / sales.length);
    const inventoryValueAt = (boundaryMs: number) =>
      state.inventoryMovements
        .filter((movement) => Date.parse(movement.createdAt) < boundaryMs)
        .reduce((total, movement) => total + movement.totalCostMinor, 0);
    const inventoryValueStartMinor = inventoryValueAt(fromMs);
    const inventoryValueEndMinor = inventoryValueAt(toMs);
    const inventoryValueAverageMinor =
      (inventoryValueStartMinor + inventoryValueEndMinor) / 2;
    const inventoryTurnoverBps =
      inventoryValueAverageMinor > 0
        ? Math.trunc(
            (costOfGoodsMinor * 10_000) / inventoryValueAverageMinor,
          )
        : 0;
    const adjustmentsUnits = inventoryAdjustments.reduce(
      (total, adjustment) => total + adjustment.unitsDelta,
      0,
    );
    const adjustmentsCostMinor = inventoryAdjustments.reduce(
      (total, adjustment) => total + adjustment.costMinor,
      0,
    );

    return {
      from,
      to,
      generatedAt: now(),
      currency: "MXN",
      salesCount: sales.length,
      grossSalesMinor,
      taxMinor,
      refundsCount: returns.length,
      refundedMinor,
      netSalesMinor,
      averageSaleMinor:
        sales.length > 0 ? Math.round(grossSalesMinor / sales.length) : 0,
      attachRateBps,
      itemsPerSaleMilli,
      costOfGoodsMinor,
      marginMinor,
      marginBps,
      inventoryValueStartMinor,
      inventoryValueEndMinor,
      inventoryTurnoverBps,
      adjustmentsUnits,
      adjustmentsCostMinor,
      paymentMethods,
      cashSessionsCount: cashSessions.length,
      openCashSessions: cashSessions.length - closedCashSessions.length,
      closedCashSessions: closedCashSessions.length,
      cashDifferenceMinor,
      cashDifferenceAbsMinor: closedCashSessions.reduce(
        (total, session) => total + Math.abs(session.differenceMinor ?? 0),
        0,
      ),
      productSales,
      inventoryAdjustments,
    };
  };

  const currentPriceFor = (variantId: string) => {
    const now = nowTime();
    return (
      state.catalogPrices
        .filter(
          (price) =>
            price.variantId === variantId && Date.parse(price.effectiveFrom) <= now,
        )
        .sort((left, right) => {
          const byEffective = Date.parse(right.effectiveFrom) - Date.parse(left.effectiveFrom);
          return byEffective !== 0
            ? byEffective
            : Date.parse(right.createdAt) - Date.parse(left.createdAt);
        })[0] ?? null
    );
  };
  const nowTime = () => Date.now();
  const catalogSnapshot = (product: StoredProduct) => ({
    ...product,
    variants: product.variants.map((variant) => ({
      ...variant,
      currentPrice: currentPriceFor(variant.id),
    })),
  });
  const allVariants = () =>
    state.catalog.flatMap((product) =>
      product.variants.map((variant) => ({ product, variant })),
    );
  const roundDiv = (numerator: number, denominator: number) =>
    Math.floor((numerator * 2 + denominator) / (denominator * 2));
  const lineTotals = (
    unitAmountMinor: number,
    quantity: number,
    taxRateBps: number,
    priceIncludesTax: boolean,
  ) => {
    const gross = unitAmountMinor * quantity;
    if (priceIncludesTax) {
      const subtotal = roundDiv(gross * 10_000, 10_000 + taxRateBps);
      return { subtotal, tax: gross - subtotal, total: gross };
    }
    const tax = roundDiv(gross * taxRateBps, 10_000);
    return { subtotal: gross, tax, total: gross + tax };
  };
  const saleRequestHash = (
    items: StoredSaleItem[],
    paymentMethod: string,
    cashReceivedMinor: number | null,
    paymentReference: string | null,
  ) =>
    JSON.stringify([
      paymentMethod,
      cashReceivedMinor,
      paymentReference,
      items
        .map((item) => [item.variantId, item.quantity])
        .sort(([left], [right]) => String(left).localeCompare(String(right))),
    ]);
  const paymentReferenceFrom = (value: unknown) => {
    const reference = String(value ?? "").trim();
    if (reference.length > 120) {
      throw apiError(
        "invalidRequest",
        "La referencia de pago debe tener como máximo 120 caracteres.",
      );
    }
    return reference || null;
  };
  const parseCatalogPrice = (input: Record<string, unknown>) => {
    const amountMinor = Number(input.amountMinor);
    const taxRateBps = Number(input.taxRateBps);
    if (!Number.isInteger(amountMinor) || amountMinor < 0) {
      throw apiError("invalidRequest", "El precio debe ser cero o mayor.");
    }
    if (!Number.isInteger(taxRateBps) || taxRateBps < 0 || taxRateBps > 10_000) {
      throw apiError("invalidRequest", "El impuesto debe estar entre 0 y 100%.");
    }
    return {
      amountMinor,
      taxRateBps,
      priceIncludesTax: Boolean(input.priceIncludesTax),
      effectiveFrom: String(input.effectiveFrom ?? "") || now(),
    };
  };
  const parseCatalogVariant = (input: Record<string, unknown>) => {
    const name = String(input.name ?? "").trim();
    const sku = String(input.sku ?? "").trim();
    const barcode = String(input.barcode ?? "").trim() || null;
    const unit = String(input.unit ?? "");
    const costMinor = Number(input.costMinor);
    if (name.length < 1) {
      throw apiError("invalidRequest", "El nombre de la variante es obligatorio.");
    }
    if (sku.length < 1) {
      throw apiError("invalidRequest", "El SKU es obligatorio.");
    }
    if (!["piece", "service", "session", "package"].includes(unit)) {
      throw apiError("invalidRequest", "Unidad inválida.");
    }
    if (!Number.isInteger(costMinor) || costMinor < 0) {
      throw apiError("invalidRequest", "El costo debe ser cero o mayor.");
    }
    if (allVariants().some(({ variant }) => variant.sku.toLowerCase() === sku.toLowerCase())) {
      throw apiError("invalidRequest", "Ya existe una variante con ese SKU.");
    }
    if (
      barcode &&
      allVariants().some(({ variant }) => variant.barcode === barcode)
    ) {
      throw apiError("invalidRequest", "Ya existe una variante con ese código de barras.");
    }
    return {
      name,
      sku,
      barcode,
      unit: unit as StoredVariant["unit"],
      costMinor,
      price: parseCatalogPrice((input.price ?? {}) as Record<string, unknown>),
    };
  };

  const internals = {
    invoke: async (command: string, args: Record<string, unknown> = {}) => {
      switch (command) {
        case "health":
          return health();

        case "get_setup_state":
          return {
            configured: state.organizations.length > 0,
            gymName: state.organizations[0]?.name,
            ownerReady: state.owner !== null,
          };

        case "bootstrap_owner": {
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          const gymName = String(args.gymName ?? "").trim();
          if (state.organizations.length === 0) {
            if (!gymName) {
              throw apiError("invalidRequest", "Escribe el nombre del gimnasio.");
            }
            state.organizations.push({ id: uuid(), name: gymName });
          }
          if (state.owner) {
            throw apiError("invalidRequest", "Este gimnasio ya tiene una cuenta de dueño.");
          }
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre del dueño es demasiado corto.");
          }
          if (password.length < 12) {
            throw apiError("invalidRequest", "La contraseña debe tener al menos 12 caracteres.");
          }
          const owner: Owner = {
            id: uuid(),
            name,
            password,
            recoveryCode: makeRecoveryCode(),
          };
          state.owner = owner;
          state.session = {
            userId: owner.id,
            organizationId: state.organizations[0]!.id,
            organizationName: state.organizations[0]!.name,
            name: owner.name,
            role: "owner",
            permissions: permissionsFor("owner"),
          };
          recordAudit({
            actor: state.session,
            action: "auth.owner_created",
            resource: "staff_membership",
          });
          return {
            session: { ...state.session },
            recoveryCode: owner.recoveryCode,
          };
        }

        case "generate_owner_recovery": {
          const session = requirePermission("manageSettings");
          if (session.role !== "owner" || !state.owner) {
            throw apiError(
              "unauthorized",
              "Solo la cuenta del dueño puede generar el código de recuperación.",
            );
          }
          state.owner.recoveryCode = makeRecoveryCode();
          recordAudit({
            actor: session,
            action: "auth.owner_recovery_generated",
            resource: "staff_membership",
          });
          return { recoveryCode: state.owner.recoveryCode };
        }

        case "recover_owner": {
          const code = normalizeRecoveryCode(String(args.recoveryCode ?? ""));
          const newPassword = String(args.newPassword ?? "");
          if (
            !state.owner ||
            !code ||
            code !== normalizeRecoveryCode(state.owner.recoveryCode)
          ) {
            recordAudit({
              action: "auth.owner_recovered",
              resource: "staff_membership",
              result: "failed",
            });
            throw apiError(
              "unauthorized",
              "Código de recuperación incorrecto o no configurado.",
            );
          }
          if (newPassword.length < 12) {
            throw apiError(
              "invalidRequest",
              "La contraseña del dueño debe tener al menos 12 caracteres.",
            );
          }
          state.owner.password = newPassword;
          state.owner.recoveryCode = makeRecoveryCode();
          state.session = {
            userId: state.owner.id,
            organizationId: state.organizations[0]!.id,
            organizationName: state.organizations[0]!.name,
            name: state.owner.name,
            role: "owner",
            permissions: permissionsFor("owner"),
          };
          recordAudit({
            actor: state.session,
            action: "auth.owner_recovered",
            resource: "staff_membership",
          });
          return {
            session: { ...state.session },
            recoveryCode: state.owner.recoveryCode,
          };
        }

        case "login": {
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          const owner =
            state.owner &&
            state.owner.name.toLowerCase() === name.toLowerCase() &&
            state.owner.password === password
              ? state.owner
              : null;
          const member = state.staff.find(
            (item) =>
              item.status === "active" &&
              item.name.toLowerCase() === name.toLowerCase() &&
              item.password === password,
          );
          if (!owner && !member) {
            recordAudit({
              action: "auth.login",
              resource: "session",
              result: "failed",
            });
            throw apiError("unauthorized", "Nombre o contraseña incorrectos.");
          }
          if (owner) {
            state.session = {
              userId: owner.id,
              organizationId: state.organizations[0]!.id,
              organizationName: state.organizations[0]!.name,
              name: owner.name,
              role: "owner",
              permissions: permissionsFor("owner"),
            };
          } else if (member) {
            member.lastLoginAt = now();
            state.session = {
              userId: member.id,
              organizationId: state.organizations[0]!.id,
              organizationName: state.organizations[0]!.name,
              name: member.name,
              role: member.role,
              permissions: permissionsFor(member.role, member.permissionOverrides),
            };
          }
          if (state.session) {
            recordAudit({
              actor: state.session,
              action: "auth.login",
              resource: "session",
            });
          }
          return state.session ? { ...state.session } : null;
        }

        case "logout":
          if (state.session) {
            recordAudit({
              actor: state.session,
              action: "auth.logout",
              resource: "session",
            });
          }
          state.session = null;
          return null;

        case "current_session":
          return state.session ? { ...state.session } : null;

        case "list_staff": {
          const session = requirePermission("manageStaff");
          const all = [...(ownerStaff() ? [ownerStaff()!] : []), ...state.staff.map(staffSummary)];
          return all.filter(
            (member) => session.role !== "manager" || member.role !== "manager",
          );
        }

        case "create_staff": {
          const session = requirePermission("manageStaff");
          const name = String(args.name ?? "").trim();
          const password = String(args.password ?? "");
          const role = String(args.role ?? "");
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre del empleado es demasiado corto.");
          }
          if (password.length < 8) {
            throw apiError("invalidRequest", "La contraseña debe tener al menos 8 caracteres.");
          }
          if (!["manager", "receptionist", "advisor", "trainer"].includes(role)) {
            throw apiError("invalidRequest", "Rol inválido.");
          }
          if (
            state.owner?.name.toLowerCase() === name.toLowerCase() ||
            state.staff.some((member) => member.name.toLowerCase() === name.toLowerCase())
          ) {
            throw apiError("invalidRequest", "Ya existe una persona con ese nombre.");
          }
          if (session.role === "manager" && role === "manager") {
            throw apiError("unauthorized", "Un gerente no puede crear otro gerente.");
          }
          const member: StaffMember = {
            id: uuid(),
            name,
            password,
            role: role as StaffMember["role"],
            status: "active",
            lastLoginAt: null,
            createdAt: now(),
            permissionOverrides: [],
          };
          state.staff.push(member);
          recordAudit({
            actor: state.session,
            action: "staff.created",
            resource: "staff_membership",
          });
          return staffSummary(member);
        }

        case "update_staff": {
          const session = requirePermission("manageStaff");
          const userId = String(args.userId ?? "");
          if (userId === session.userId) {
            throw apiError("invalidRequest", "No puedes modificar tu propia cuenta.");
          }
          const member = state.staff.find((item) => item.id === userId);
          if (!member) {
            throw apiError("invalidRequest", "La cuenta del dueño no se puede modificar.");
          }
          const role = String(args.role ?? "");
          const status = String(args.status ?? "");
          if (!["manager", "receptionist", "advisor", "trainer"].includes(role)) {
            throw apiError("invalidRequest", "Rol inválido.");
          }
          if (status !== "active" && status !== "revoked") {
            throw apiError("invalidRequest", "Estado inválido.");
          }
          if (
            session.role === "manager" &&
            (member.role === "manager" || role === "manager")
          ) {
            throw apiError(
              "unauthorized",
              "Solo puedes administrar recepción, asesoría y entrenamiento.",
            );
          }
          member.role = role as StaffMember["role"];
          member.status = status;
          recordAudit({
            actor: state.session,
            action: "staff.updated",
            resource: "staff_membership",
          });
          return staffSummary(member);
        }

        case "update_staff_permissions": {
          const session = requirePermission("managePermissions");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const userId = String(input.userId ?? "");
          if (userId === session.userId) {
            throw apiError(
              "invalidRequest",
              "No puedes modificar tus propios permisos.",
            );
          }
          const member = state.staff.find((item) => item.id === userId);
          if (!member) {
            throw apiError("invalidRequest", "El colaborador no existe.");
          }
          const allPermissions = Object.keys(permissionLabels) as Permission[];
          const rawOverrides = Array.isArray(input.overrides)
            ? input.overrides
            : [];
          const overrides = rawOverrides.map((entry) => {
            const candidate = entry as Record<string, unknown>;
            const permission = String(candidate.permission ?? "") as Permission;
            const effect = String(candidate.effect ?? "");
            if (!allPermissions.includes(permission)) {
              throw apiError("invalidRequest", "Permiso inválido.");
            }
            if (ownerOnlyPermissions.includes(permission)) {
              throw apiError(
                "invalidRequest",
                "Ese permiso solo puede tenerlo el dueño.",
              );
            }
            if (effect !== "allow" && effect !== "deny") {
              throw apiError("invalidRequest", "Efecto de permiso inválido.");
            }
            return { permission, effect: effect as PermissionEffect };
          });
          member.permissionOverrides = overrides;
          recordAudit({
            actor: state.session,
            action: "staff.permissions_updated",
            resource: "staff_membership",
          });
          return staffSummary(member);
        }

        case "membership_dashboard": {
          requireManageMembers();
          return dashboardSnapshot();
        }

        case "list_membership_plans": {
          requireManageMembers();
          return state.membershipPlans.map((plan) => ({ ...plan }));
        }

        case "create_membership_plan": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const name = String(input.name ?? "").trim();
          const description = String(input.description ?? "").trim() || null;
          const durationDays = Number(input.durationDays);
          const priceMinor = Number(input.priceMinor);
          const graceDays = Number(input.graceDays);
          const accessDays = normalizeAccessDays(String(input.accessDays ?? ""));
          const accessStartMinute = Number(input.accessStartMinute);
          const accessEndMinute = Number(input.accessEndMinute);
          const minAge = optionalAge(input.minAge);
          const maxAge = optionalAge(input.maxAge);
          if (!name) {
            throw apiError("invalidRequest", "El nombre del plan es obligatorio.");
          }
          if (!Number.isInteger(durationDays) || durationDays < 1) {
            throw apiError("invalidRequest", "La duración debe ser de al menos un día.");
          }
          if (!Number.isInteger(priceMinor) || priceMinor < 0) {
            throw apiError("invalidRequest", "El precio debe ser cero o mayor.");
          }
          if (!Number.isInteger(graceDays) || graceDays < 0) {
            throw apiError("invalidRequest", "Los días de gracia deben ser cero o mayor.");
          }
          validatePlanAccess(accessDays, accessStartMinute, accessEndMinute, minAge, maxAge);
          if (
            state.membershipPlans.some(
              (plan) => plan.name.toLowerCase() === name.toLowerCase(),
            )
          ) {
            throw apiError("invalidRequest", "Ya existe un plan con ese nombre.");
          }
          const plan: StoredMembershipPlan = {
            id: uuid(),
            name,
            description,
            durationDays,
            priceMinor,
            currency: "MXN",
            graceDays,
            version: 1,
            accessDays,
            accessStartMinute,
            accessEndMinute,
            minAge,
            maxAge,
            active: true,
            createdAt: now(),
            updatedAt: now(),
          };
          state.membershipPlans.push(plan);
          recordAudit({
            actor: state.session,
            action: "membership_plan.created",
            resource: "membership_plan",
          });
          return { ...plan };
        }

        case "update_membership_plan": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const plan = state.membershipPlans.find(
            (item) => item.id === String(input.planId ?? ""),
          );
          if (!plan) {
            throw apiError("notFound", "El plan de membresía no existe.");
          }
          const name = String(input.name ?? "").trim();
          const durationDays = Number(input.durationDays);
          const priceMinor = Number(input.priceMinor);
          const graceDays = Number(input.graceDays);
          const accessDays = normalizeAccessDays(String(input.accessDays ?? ""));
          const accessStartMinute = Number(input.accessStartMinute);
          const accessEndMinute = Number(input.accessEndMinute);
          const minAge = optionalAge(input.minAge);
          const maxAge = optionalAge(input.maxAge);
          if (!name) {
            throw apiError("invalidRequest", "El nombre del plan es obligatorio.");
          }
          if (!Number.isInteger(durationDays) || durationDays < 1) {
            throw apiError("invalidRequest", "La duración debe ser de al menos un día.");
          }
          if (!Number.isInteger(priceMinor) || priceMinor < 0) {
            throw apiError("invalidRequest", "El precio debe ser cero o mayor.");
          }
          if (!Number.isInteger(graceDays) || graceDays < 0) {
            throw apiError("invalidRequest", "Los días de gracia deben ser cero o mayor.");
          }
          validatePlanAccess(accessDays, accessStartMinute, accessEndMinute, minAge, maxAge);
          if (
            state.membershipPlans.some(
              (item) =>
                item.id !== plan.id &&
                item.name.toLowerCase() === name.toLowerCase(),
            )
          ) {
            throw apiError("invalidRequest", "Ya existe un plan con ese nombre.");
          }
          plan.name = name;
          plan.description = String(input.description ?? "").trim() || null;
          plan.durationDays = durationDays;
          plan.priceMinor = priceMinor;
          plan.graceDays = graceDays;
          plan.version += 1;
          plan.accessDays = accessDays;
          plan.accessStartMinute = accessStartMinute;
          plan.accessEndMinute = accessEndMinute;
          plan.minAge = minAge;
          plan.maxAge = maxAge;
          plan.active = Boolean(input.active);
          plan.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "membership_plan.updated",
            resource: "membership_plan",
          });
          return { ...plan };
        }

        case "list_members": {
          requireManageMembers();
          const query = String(args.query ?? "").trim().toLowerCase();
          return state.members
            .filter((member) => {
              if (!query) {
                return true;
              }
              return [
                member.firstName,
                member.lastName,
                member.memberNumber,
                member.phone ?? "",
                member.email ?? "",
              ]
                .join(" ")
                .toLowerCase()
                .includes(query);
            })
            .map(memberSnapshot);
        }

        case "get_member": {
          requireManageMembers();
          const member = state.members.find(
            (item) => item.id === String(args.memberId ?? ""),
          );
          if (!member) {
            throw apiError("notFound", "El socio no existe.");
          }
          return memberSnapshot(member);
        }

        case "create_member": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const firstName = String(input.firstName ?? "").trim();
          const lastName = String(input.lastName ?? "").trim();
          const phone = String(input.phone ?? "").trim() || null;
          const email = String(input.email ?? "").trim().toLowerCase() || null;
          const birthDate = String(input.birthDate ?? "").trim() || null;
          const notes = String(input.notes ?? "").trim() || null;
          if (!firstName || !lastName) {
            throw apiError("invalidRequest", "Nombre y apellido son obligatorios.");
          }
          if (email && !email.includes("@")) {
            throw apiError("invalidRequest", "El email no es válido.");
          }
          state.memberSequence += 1;
          const member: StoredMember = {
            id: uuid(),
            memberNumber: `SOC-${String(state.memberSequence).padStart(5, "0")}`,
            firstName,
            lastName,
            phone,
            email,
            birthDate,
            status: "active",
            joinedAt: dateOnly(),
            notes,
            createdAt: now(),
            updatedAt: now(),
          };
          state.members.push(member);
          recordAudit({
            actor: state.session,
            action: "member.created",
            resource: "member",
          });
          return memberSnapshot(member);
        }

        case "update_member": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const member = state.members.find(
            (item) => item.id === String(input.memberId ?? ""),
          );
          if (!member) {
            throw apiError("notFound", "El socio no existe.");
          }
          const firstName = String(input.firstName ?? "").trim();
          const lastName = String(input.lastName ?? "").trim();
          const status = String(input.status ?? "");
          if (!firstName || !lastName) {
            throw apiError("invalidRequest", "Nombre y apellido son obligatorios.");
          }
          if (!["active", "inactive", "archived"].includes(status)) {
            throw apiError("invalidRequest", "Estado de socio inválido.");
          }
          member.firstName = firstName;
          member.lastName = lastName;
          member.phone = String(input.phone ?? "").trim() || null;
          member.email = String(input.email ?? "").trim().toLowerCase() || null;
          member.birthDate = String(input.birthDate ?? "").trim() || null;
          member.status = status as StoredMember["status"];
          member.notes = String(input.notes ?? "").trim() || null;
          member.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "member.updated",
            resource: "member",
          });
          return memberSnapshot(member);
        }

        case "list_prospects": {
          requireManageMembers();
          const query = String(args.query ?? "").trim().toLowerCase();
          return state.prospects
            .filter((prospect) =>
              !query
                ? true
                : [prospect.name, prospect.phone ?? "", prospect.email ?? ""]
                    .join(" ")
                    .toLowerCase()
                    .includes(query),
            )
            .map((prospect) => ({ ...prospect }));
        }

        case "create_prospect": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const name = String(input.name ?? "").trim();
          if (!name) {
            throw apiError("invalidRequest", "El nombre del prospecto es obligatorio.");
          }
          const prospect: StoredProspect = {
            id: uuid(),
            name,
            phone: String(input.phone ?? "").trim() || null,
            email: String(input.email ?? "").trim().toLowerCase() || null,
            source: String(input.source ?? "").trim() || null,
            status: "new",
            notes: String(input.notes ?? "").trim() || null,
            convertedMemberId: null,
            createdAt: now(),
            updatedAt: now(),
          };
          state.prospects.push(prospect);
          recordAudit({
            actor: state.session,
            action: "prospect.created",
            resource: "prospect",
          });
          return { ...prospect };
        }

        case "update_prospect": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const prospect = state.prospects.find(
            (item) => item.id === String(input.prospectId ?? ""),
          );
          if (!prospect) {
            throw apiError("notFound", "El prospecto no existe.");
          }
          const name = String(input.name ?? "").trim();
          const status = String(input.status ?? "");
          if (!name) {
            throw apiError("invalidRequest", "El nombre del prospecto es obligatorio.");
          }
          if (!["new", "contacted", "trial", "converted", "lost"].includes(status)) {
            throw apiError("invalidRequest", "Estado de prospecto inválido.");
          }
          if (prospect.status === "converted" && status !== "converted") {
            throw apiError("invalidRequest", "Un prospecto convertido no se puede reabrir.");
          }
          if (status === "converted" && !prospect.convertedMemberId) {
            throw apiError("invalidRequest", "Usa la acción de conversión.");
          }
          prospect.name = name;
          prospect.phone = String(input.phone ?? "").trim() || null;
          prospect.email = String(input.email ?? "").trim().toLowerCase() || null;
          prospect.source = String(input.source ?? "").trim() || null;
          prospect.status = status as StoredProspect["status"];
          prospect.notes = String(input.notes ?? "").trim() || null;
          prospect.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "prospect.updated",
            resource: "prospect",
          });
          return { ...prospect };
        }

        case "convert_prospect": {
          requireManageMembers();
          const prospect = state.prospects.find(
            (item) => item.id === String(args.prospectId ?? ""),
          );
          if (!prospect) {
            throw apiError("notFound", "El prospecto no existe.");
          }
          if (prospect.status === "converted" || prospect.convertedMemberId) {
            throw apiError("invalidRequest", "El prospecto ya fue convertido.");
          }
          const [firstName = prospect.name, ...rest] = prospect.name.split(/\s+/);
          state.memberSequence += 1;
          const member: StoredMember = {
            id: uuid(),
            memberNumber: `SOC-${String(state.memberSequence).padStart(5, "0")}`,
            firstName,
            lastName: rest.join(" "),
            phone: prospect.phone,
            email: prospect.email,
            birthDate: null,
            status: "active",
            joinedAt: dateOnly(),
            notes: prospect.notes,
            createdAt: now(),
            updatedAt: now(),
          };
          state.members.push(member);
          prospect.status = "converted";
          prospect.convertedMemberId = member.id;
          prospect.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "prospect.converted",
            resource: "member",
          });
          return memberSnapshot(member);
        }

        case "list_memberships": {
          requireManageMembers();
          return state.memberships
            .filter((membership) => membership.memberId === String(args.memberId ?? ""))
            .sort((left, right) => right.createdAt.localeCompare(left.createdAt))
            .map(membershipSnapshot);
        }

        case "create_membership": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const member = state.members.find(
            (item) => item.id === String(input.memberId ?? ""),
          );
          if (!member) {
            throw apiError("notFound", "El socio no existe.");
          }
          if (member.status !== "active") {
            throw apiError("invalidRequest", "Solo socios activos pueden recibir membresía.");
          }
          const plan = state.membershipPlans.find(
            (item) => item.id === String(input.planId ?? ""),
          );
          if (!plan || !plan.active) {
            throw apiError("invalidRequest", "El plan no está activo.");
          }
          if (
            state.memberships.some(
              (membership) =>
                membership.memberId === member.id &&
                ["scheduled", "active", "paused"].includes(membership.status),
            )
          ) {
            throw apiError("invalidRequest", "El socio ya tiene una membresía vigente.");
          }
          const startDate = String(input.startDate ?? "").trim() || dateOnly();
          ensurePlanEligibility(plan, member, startDate);
          const endDate = addDays(startDate, plan.durationDays - 1);
          const membership: StoredMembership = {
            id: uuid(),
            memberId: member.id,
            planId: plan.id,
            planName: plan.name,
            planVersion: plan.version,
            planVersionId: uuid(),
            status: startDate > dateOnly() ? "scheduled" : "active",
            startDate,
            endDate,
            priceMinor: plan.priceMinor,
            currency: plan.currency,
            graceDays: plan.graceDays,
            accessDays: plan.accessDays,
            accessStartMinute: plan.accessStartMinute,
            accessEndMinute: plan.accessEndMinute,
            minAge: plan.minAge,
            maxAge: plan.maxAge,
            pausedAt: null,
            pauseUntil: null,
            cancellationReason: null,
            createdAt: now(),
            updatedAt: now(),
          };
          state.memberships.push(membership);
          if (plan.priceMinor > 0) {
            state.membershipCharges.push({
              id: uuid(),
              membershipId: membership.id,
              memberId: member.id,
              description: `Membresía ${plan.name}`,
              amountMinor: plan.priceMinor,
              currency: plan.currency,
              dueDate: startDate,
              status: "pending",
              paidMinor: 0,
              createdAt: now(),
              updatedAt: now(),
            });
          }
          recordAudit({
            actor: state.session,
            action: "membership.created",
            resource: "membership",
          });
          return membershipSnapshot(membership);
        }

        case "renew_membership": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const membership = state.memberships.find(
            (item) => item.id === String(input.membershipId ?? ""),
          );
          if (!membership) {
            throw apiError("notFound", "La membresía no existe.");
          }
          if (["paused", "cancelled"].includes(membership.status)) {
            throw apiError("invalidRequest", "La membresía no permite renovación.");
          }
          const plan = state.membershipPlans.find(
            (item) =>
              item.id ===
              (String(input.planId ?? "").trim() || membership.planId),
          );
          if (!plan || !plan.active) {
            throw apiError("invalidRequest", "El plan no está activo.");
          }
          const periodStart =
            membership.endDate >= dateOnly() ? addDays(membership.endDate, 1) : dateOnly();
          const member = state.members.find((item) => item.id === membership.memberId);
          if (!member) {
            throw apiError("notFound", "El socio no existe.");
          }
          ensurePlanEligibility(plan, member, periodStart);
          const endDate = addDays(periodStart, plan.durationDays - 1);
          membership.planId = plan.id;
          membership.planName = plan.name;
          membership.planVersion = plan.version;
          membership.planVersionId = uuid();
          membership.status = membership.startDate > dateOnly() ? "scheduled" : "active";
          membership.endDate = endDate;
          membership.priceMinor = plan.priceMinor;
          membership.currency = plan.currency;
          membership.graceDays = plan.graceDays;
          membership.accessDays = plan.accessDays;
          membership.accessStartMinute = plan.accessStartMinute;
          membership.accessEndMinute = plan.accessEndMinute;
          membership.minAge = plan.minAge;
          membership.maxAge = plan.maxAge;
          membership.pausedAt = null;
          membership.pauseUntil = null;
          membership.cancellationReason = null;
          membership.updatedAt = now();
          if (plan.priceMinor > 0) {
            state.membershipCharges.push({
              id: uuid(),
              membershipId: membership.id,
              memberId: membership.memberId,
              description: `Renovación ${plan.name}`,
              amountMinor: plan.priceMinor,
              currency: plan.currency,
              dueDate: periodStart,
              status: "pending",
              paidMinor: 0,
              createdAt: now(),
              updatedAt: now(),
            });
          }
          recordAudit({
            actor: state.session,
            action: "membership.renewed",
            resource: "membership",
          });
          return membershipSnapshot(membership);
        }

        case "pause_membership": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const membership = state.memberships.find(
            (item) => item.id === String(input.membershipId ?? ""),
          );
          if (!membership) {
            throw apiError("notFound", "La membresía no existe.");
          }
          if (!["active", "scheduled"].includes(membership.status)) {
            throw apiError("invalidRequest", "La membresía no se puede pausar.");
          }
          const untilDate = String(input.untilDate ?? "").trim();
          if (!untilDate || untilDate < dateOnly()) {
            throw apiError("invalidRequest", "La fecha de pausa no puede estar en el pasado.");
          }
          membership.status = "paused";
          membership.pausedAt = dateOnly();
          membership.pauseUntil = untilDate;
          membership.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "membership.paused",
            resource: "membership",
          });
          return membershipSnapshot(membership);
        }

        case "resume_membership": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const membership = state.memberships.find(
            (item) => item.id === String(input.membershipId ?? ""),
          );
          if (!membership) {
            throw apiError("notFound", "La membresía no existe.");
          }
          if (membership.status !== "paused" || !membership.pausedAt) {
            throw apiError("invalidRequest", "La membresía no está pausada.");
          }
          const pausedDays = Math.max(
            0,
            Math.round(
              (Date.parse(`${dateOnly()}T00:00:00.000Z`) -
                Date.parse(`${membership.pausedAt}T00:00:00.000Z`)) /
                86_400_000,
            ),
          );
          membership.endDate = addDays(membership.endDate, pausedDays);
          membership.status = membership.startDate > dateOnly() ? "scheduled" : "active";
          membership.pausedAt = null;
          membership.pauseUntil = null;
          membership.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "membership.resumed",
            resource: "membership",
          });
          return membershipSnapshot(membership);
        }

        case "cancel_membership": {
          requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const membership = state.memberships.find(
            (item) => item.id === String(input.membershipId ?? ""),
          );
          if (!membership) {
            throw apiError("notFound", "La membresía no existe.");
          }
          if (!["active", "scheduled", "paused"].includes(membership.status)) {
            throw apiError("invalidRequest", "La membresía no se puede cancelar.");
          }
          const reason = String(input.reason ?? "").trim();
          if (!reason) {
            throw apiError("invalidRequest", "El motivo de cancelación es obligatorio.");
          }
          membership.status = "cancelled";
          membership.cancellationReason = reason;
          membership.pausedAt = null;
          membership.pauseUntil = null;
          membership.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "membership.cancelled",
            resource: "membership",
          });
          return membershipSnapshot(membership);
        }

        case "member_account": {
          requireManageMembers();
          const memberId = String(args.memberId ?? "");
          if (!state.members.some((member) => member.id === memberId)) {
            throw apiError("notFound", "El socio no existe.");
          }
          return accountSnapshot(memberId);
        }

        case "record_membership_payment": {
          const session = requireManageMembers();
          const input = (args.input ?? {}) as Record<string, unknown>;
          const charge = state.membershipCharges.find(
            (item) => item.id === String(input.chargeId ?? ""),
          );
          if (!charge) {
            throw apiError("notFound", "El cargo no existe.");
          }
          if (charge.status !== "pending") {
            throw apiError("invalidRequest", "El cargo no está pendiente de pago.");
          }
          const amountMinor = Number(input.amountMinor);
          const paymentMethod = String(input.paymentMethod ?? "");
          const reference = String(input.reference ?? "").trim() || null;
          const idempotencyKey = String(input.idempotencyKey ?? "").trim();
          if (!Number.isInteger(amountMinor) || amountMinor <= 0) {
            throw apiError("invalidRequest", "El pago debe ser mayor a cero.");
          }
          const remaining = charge.amountMinor - charge.paidMinor;
          if (amountMinor > remaining) {
            throw apiError("invalidRequest", "El pago supera el saldo del cargo.");
          }
          if (!["cash", "transfer", "point"].includes(paymentMethod)) {
            throw apiError("invalidRequest", "Método de pago inválido.");
          }
          if (!idempotencyKey) {
            throw apiError("invalidRequest", "La clave de idempotencia es obligatoria.");
          }
          const requestHash = `${charge.id}:${amountMinor}:${paymentMethod}:${reference ?? ""}`;
          const previous = state.membershipPaymentIdempotency.get(idempotencyKey);
          if (previous) {
            if (previous.requestHash !== requestHash) {
              throw apiError(
                "invalidRequest",
                "La clave de idempotencia ya se usó con otro pago.",
              );
            }
            const existing = state.membershipPayments.find(
              (payment) => payment.id === previous.paymentId,
            );
            if (existing) {
              return paymentSnapshot(existing);
            }
          }
          const cashSession = paymentMethod === "cash" ? openCashSession() : null;
          if (paymentMethod === "cash" && !cashSession) {
            throw apiError("invalidRequest", "Abre la caja antes de cobrar en efectivo.");
          }
          state.membershipReceiptSequence += 1;
          const receiptNumber = `REC-${String(state.membershipReceiptSequence).padStart(5, "0")}`;
          const payment: StoredMembershipPayment = {
            id: uuid(),
            chargeId: charge.id,
            membershipId: charge.membershipId,
            memberId: charge.memberId,
            amountMinor,
            paymentMethod: paymentMethod as StoredMembershipPayment["paymentMethod"],
            reference,
            receiptNumber,
            cashSessionId: cashSession?.id ?? null,
            receivedBy: session.userId,
            receivedByName: session.name,
            createdAt: now(),
          };
          charge.paidMinor += amountMinor;
          charge.status = charge.paidMinor >= charge.amountMinor ? "paid" : "pending";
          charge.updatedAt = now();
          state.membershipPayments.push(payment);
          if (cashSession) {
            state.cashMovements.unshift({
              id: uuid(),
              cashSessionId: cashSession.id,
              movementType: "income",
              category: "membership_payment",
              amountMinor,
              paymentMethod: "cash",
              evidence: `Pago membresía ${receiptNumber}`,
              authorizedBy: session.userId,
              authorizedByName: session.name,
              createdAt: payment.createdAt,
            });
          }
          state.membershipPaymentIdempotency.set(idempotencyKey, {
            requestHash,
            paymentId: payment.id,
          });
          recordAudit({
            actor: session,
            action: "membership.payment_recorded",
            resource: "membership_payment",
          });
          return paymentSnapshot(payment);
        }

        case "membership_payment_receipt": {
          requireManageMembers();
          const payment = state.membershipPayments.find(
            (item) => item.id === String(args.paymentId ?? ""),
          );
          if (!payment) {
            throw apiError("notFound", "El pago no existe.");
          }
          return receiptSnapshot(payment);
        }

        case "list_catalog": {
          requirePermission("operatePos");
          return state.catalog.map(catalogSnapshot);
        }

        case "list_sales": {
          requirePermission("operatePos");
          return state.sales.slice(0, 50);
        }

        case "list_payment_charges": {
          requirePermission("viewFinance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const saleId = input.saleId ? String(input.saleId) : null;
          const reference = input.externalReference
            ? String(input.externalReference)
            : null;
          const limit = Math.min(Math.max(Number(input.limit ?? 50), 1), 100);
          return state.paymentCharges
            .filter((charge) => !saleId || charge.saleId === saleId)
            .filter(
              (charge) =>
                !reference || charge.externalReference === reference,
            )
            .slice(0, limit)
            .map((charge) => ({ ...charge }));
        }

        case "financial_report": {
          requirePermission("viewFinance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          return financialReportSnapshot(
            String(input.from ?? ""),
            String(input.to ?? ""),
          );
        }

        case "export_financial_report_csv": {
          requirePermission("viewFinance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const report = financialReportSnapshot(
            String(input.from ?? ""),
            String(input.to ?? ""),
          );
          const fileName = `reporte-${report.from.slice(0, 10)}-${report.to.slice(0, 10)}-${Date.now()}.csv`;
          recordAudit({
            actor: state.session,
            action: "report.exported",
            resource: "financial_report",
            correlationId: fileName,
          });
          return {
            fileName,
            path: `/tmp/fitness-gym-e2e/exports/${fileName}`,
            bytes: 2_048,
          };
        }

        case "list_inventory": {
          requirePermission("manageInventory");
          return inventorySnapshot();
        }

        case "list_inventory_movements": {
          requirePermission("manageInventory");
          const variantId = args.variantId ? String(args.variantId) : null;
          const limit = Number(args.limit ?? 50);
          return state.inventoryMovements
            .filter((movement) => !variantId || movement.variantId === variantId)
            .slice(0, Math.min(Math.max(limit, 1), 200))
            .map(inventoryMovementSnapshot);
        }

        case "record_inventory_entry": {
          const session = requirePermission("manageInventory");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const variantId = String(input.variantId ?? "");
          const match = allVariants().find((entry) => entry.variant.id === variantId);
          if (!match) {
            throw apiError("notFound", "La variante no existe.");
          }
          if (match.variant.unit !== "piece") {
            throw apiError("invalidRequest", "El inventario solo aplica a piezas.");
          }
          const quantity = Number(input.quantity);
          if (!Number.isInteger(quantity) || quantity < 1 || quantity > 999_999) {
            throw apiError("invalidRequest", "La cantidad debe estar entre 1 y 999999.");
          }
          const unitCostMinor = Number(input.unitCostMinor);
          if (
            !Number.isInteger(unitCostMinor) ||
            unitCostMinor < 0 ||
            unitCostMinor > 1_000_000_000
          ) {
            throw apiError("invalidRequest", "El costo unitario no es válido.");
          }
          const reference = String(input.reference ?? "").trim() || null;
          const note = String(input.note ?? "").trim() || null;
          if (reference && reference.length > 120) {
            throw apiError("invalidRequest", "La referencia es demasiado larga.");
          }
          if (note && note.length > 300) {
            throw apiError("invalidRequest", "La nota es demasiado larga.");
          }
          const movement = postInventoryMovement({
            variantId,
            movementType: "entry",
            quantityDelta: quantity,
            unitCostMinor,
            sourceType: "manual",
            reference,
            note,
            authorizedBy: session.userId,
            authorizedByName: session.name,
          });
          recordAudit({
            actor: state.session,
            action: "inventory.entry",
            resource: "inventory_movement",
            correlationId: movement.id,
          });
          return inventoryMovementSnapshot(movement);
        }

        case "adjust_inventory": {
          const session = requirePermission("manageInventory");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const variantId = String(input.variantId ?? "");
          const match = allVariants().find((entry) => entry.variant.id === variantId);
          if (!match) {
            throw apiError("notFound", "La variante no existe.");
          }
          if (match.variant.unit !== "piece") {
            throw apiError("invalidRequest", "El inventario solo aplica a piezas.");
          }
          const quantityDelta = Number(input.quantityDelta);
          if (
            !Number.isInteger(quantityDelta) ||
            quantityDelta === 0 ||
            Math.abs(quantityDelta) > 999_999
          ) {
            throw apiError(
              "invalidRequest",
              "El ajuste debe estar entre -999999 y 999999 y no ser cero.",
            );
          }
          const reason = String(input.reason ?? "").trim();
          if (reason.length < 1 || reason.length > 300) {
            throw apiError(
              "invalidRequest",
              "El motivo del ajuste debe tener entre 1 y 300 caracteres.",
            );
          }
          const providedCost =
            input.unitCostMinor == null ? null : Number(input.unitCostMinor);
          if (
            providedCost != null &&
            (!Number.isInteger(providedCost) ||
              providedCost < 0 ||
              providedCost > 1_000_000_000)
          ) {
            throw apiError("invalidRequest", "El costo unitario no es válido.");
          }
          const unitCostMinor = providedCost ?? averageCostFor(variantId);
          const movement = postInventoryMovement({
            variantId,
            movementType: "adjustment",
            quantityDelta,
            unitCostMinor,
            sourceType: "adjustment",
            note: reason,
            authorizedBy: session.userId,
            authorizedByName: session.name,
          });
          recordAudit({
            actor: state.session,
            action: "inventory.adjusted",
            resource: "inventory_movement",
            correlationId: movement.id,
          });
          return inventoryMovementSnapshot(movement);
        }

        case "update_inventory_threshold": {
          requirePermission("manageInventory");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const variantId = String(input.variantId ?? "");
          const match = allVariants().find((entry) => entry.variant.id === variantId);
          if (!match) {
            throw apiError("notFound", "La variante no existe.");
          }
          if (match.variant.unit !== "piece") {
            throw apiError("invalidRequest", "El inventario solo aplica a piezas.");
          }
          const minStock = Number(input.minStock);
          if (!Number.isInteger(minStock) || minStock < 0 || minStock > 999_999) {
            throw apiError("invalidRequest", "El mínimo debe estar entre 0 y 999999.");
          }
          state.inventoryThresholds.set(variantId, minStock);
          recordAudit({
            actor: state.session,
            action: "inventory.threshold.updated",
            resource: "inventory_policy",
            correlationId: variantId,
          });
          return inventorySnapshot().find((item) => item.variantId === variantId);
        }

        case "current_cash_session": {
          requirePermission("operatePos");
          const active = openCashSession();
          return active ? cashSessionSnapshot(active) : null;
        }

        case "list_cash_sessions": {
          requirePermission("viewFinance");
          return state.cashSessions.map(cashSessionSnapshot);
        }

        case "open_cash_session": {
          const session = requirePermission("operatePos");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const openingAmountMinor = Number(input.openingAmountMinor);
          if (
            !Number.isInteger(openingAmountMinor) ||
            openingAmountMinor < 0 ||
            openingAmountMinor > 1_000_000_000
          ) {
            throw apiError("invalidRequest", "El monto inicial no es válido.");
          }
          if (openCashSession()) {
            throw apiError("invalidRequest", "Ya hay un turno de caja abierto.");
          }
          const openedAt = now();
          const cashSession: StoredCashSession = {
            id: uuid(),
            status: "open",
            currency: "MXN",
            openingAmountMinor,
            countedAmountMinor: null,
            differenceMinor: null,
            differenceReason: null,
            openedBy: session.userId,
            openedByName: session.name,
            openedAt,
            closedBy: null,
            closedByName: null,
            closedAt: null,
          };
          state.cashSessions.unshift(cashSession);
          recordAudit({
            actor: state.session,
            action: "cash.opened",
            resource: "cash_session",
            correlationId: cashSession.id,
          });
          return cashSessionSnapshot(cashSession);
        }

        case "close_cash_session": {
          const session = requirePermission("operatePos");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const sessionId = String(input.sessionId ?? "");
          const countedAmountMinor = Number(input.countedAmountMinor);
          const cashSession = state.cashSessions.find((item) => item.id === sessionId);
          if (!cashSession) {
            throw apiError("notFound", "El turno de caja no existe.");
          }
          if (cashSession.status !== "open") {
            throw apiError("invalidRequest", "El turno de caja ya está cerrado.");
          }
          if (
            !Number.isInteger(countedAmountMinor) ||
            countedAmountMinor < 0 ||
            countedAmountMinor > 1_000_000_000
          ) {
            throw apiError("invalidRequest", "El efectivo contado no es válido.");
          }
          const snapshot = cashSessionSnapshot(cashSession);
          const differenceMinor = countedAmountMinor - snapshot.expectedCashMinor;
          const reason =
            typeof input.differenceReason === "string"
              ? input.differenceReason.trim()
              : "";
          if (differenceMinor !== 0 && reason.length === 0) {
            throw apiError(
              "invalidRequest",
              "Registra el motivo de la diferencia para cerrar la caja.",
            );
          }
          if (reason.length > 300) {
            throw apiError("invalidRequest", "El motivo de la diferencia es demasiado largo.");
          }
          cashSession.status = "closed";
          cashSession.countedAmountMinor = countedAmountMinor;
          cashSession.differenceMinor = differenceMinor;
          cashSession.differenceReason = differenceMinor === 0 ? null : reason;
          cashSession.closedBy = session.userId;
          cashSession.closedByName = session.name;
          cashSession.closedAt = now();
          recordAudit({
            actor: state.session,
            action: "cash.closed",
            resource: "cash_session",
            correlationId: cashSession.id,
          });
          return cashSessionSnapshot(cashSession);
        }

        case "list_cash_movements": {
          requireAnyPermission(["operatePos", "viewFinance"]);
          const sessionId = String(args.sessionId ?? "");
          return state.cashMovements.filter(
            (movement) => movement.cashSessionId === sessionId,
          );
        }

        case "record_cash_movement": {
          const session = requirePermission("operatePos");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const sessionId = String(input.sessionId ?? "");
          const movementType = String(input.movementType ?? "");
          const category = String(input.category ?? "").trim();
          const amountMinor = Number(input.amountMinor);
          const paymentMethod = String(input.paymentMethod ?? "");
          const evidence = String(input.evidence ?? "").trim();
          const cashSession = state.cashSessions.find((item) => item.id === sessionId);
          if (!cashSession) {
            throw apiError("notFound", "El turno de caja no existe.");
          }
          if (cashSession.status !== "open") {
            throw apiError("invalidRequest", "El turno de caja ya está cerrado.");
          }
          if (!["income", "expense"].includes(movementType)) {
            throw apiError("invalidRequest", "El tipo de movimiento no es válido.");
          }
          if (!["cash", "transfer", "point"].includes(paymentMethod)) {
            throw apiError("invalidRequest", "El método de pago no es válido.");
          }
          if (
            !Number.isInteger(amountMinor) ||
            amountMinor < 1 ||
            amountMinor > 1_000_000_000
          ) {
            throw apiError("invalidRequest", "El monto del movimiento no es válido.");
          }
          if (category.length === 0 || category.length > 80) {
            throw apiError("invalidRequest", "La categoría no es válida.");
          }
          if (evidence.length === 0 || evidence.length > 200) {
            throw apiError("invalidRequest", "La evidencia no es válida.");
          }
          const movement: StoredCashMovement = {
            id: uuid(),
            cashSessionId: sessionId,
            movementType: movementType as StoredCashMovement["movementType"],
            category,
            amountMinor,
            paymentMethod: paymentMethod as StoredCashMovement["paymentMethod"],
            evidence,
            authorizedBy: session.userId,
            authorizedByName: session.name,
            createdAt: now(),
          };
          state.cashMovements.unshift(movement);
          recordAudit({
            actor: state.session,
            action: "cash.movement_recorded",
            resource: "cash_movement",
            correlationId: movement.id,
          });
          return movement;
        }

        case "create_sale": {
          const session = requirePermission("operatePos");
          const activeCashSession = openCashSession();
          if (!activeCashSession) {
            throw apiError("invalidRequest", "Abre la caja antes de registrar una venta.");
          }
          const input = (args.input ?? {}) as Record<string, unknown>;
          const rawItems = Array.isArray(input.items) ? input.items : [];
          if (rawItems.length === 0) {
            throw apiError("invalidRequest", "La venta necesita al menos un artículo.");
          }
          if (rawItems.length > 50) {
            throw apiError("invalidRequest", "La venta admite como máximo 50 artículos.");
          }
          const paymentMethod = String(input.paymentMethod ?? "");
          if (!["cash", "transfer", "point"].includes(paymentMethod)) {
            throw apiError("invalidRequest", "Método de pago inválido.");
          }
          const idempotencyKey = String(input.idempotencyKey ?? "").trim();
          if (idempotencyKey.length < 1 || idempotencyKey.length > 128) {
            throw apiError(
              "invalidRequest",
              "La clave de idempotencia debe tener entre 1 y 128 caracteres.",
            );
          }
          const seen = new Set<string>();
          const items: StoredSaleItem[] = rawItems.map((rawItem) => {
            const item = (rawItem ?? {}) as Record<string, unknown>;
            const variantId = String(item.variantId ?? "");
            const quantity = Number(item.quantity);
            if (!Number.isInteger(quantity) || quantity < 1 || quantity > 999) {
              throw apiError("invalidRequest", "La cantidad debe estar entre 1 y 999.");
            }
            if (seen.has(variantId)) {
              throw apiError(
                "invalidRequest",
                "La misma variante no puede repetirse en la venta.",
              );
            }
            seen.add(variantId);
            const match = allVariants().find((entry) => entry.variant.id === variantId);
            if (!match) {
              throw apiError("notFound", "La variante no existe.");
            }
            if (match.product.status !== "active" || match.variant.status !== "active") {
              throw apiError(
                "invalidRequest",
                "No puedes vender productos o variantes archivadas.",
              );
            }
            const price = currentPriceFor(variantId);
            if (!price) {
              throw apiError("invalidRequest", "La variante no tiene un precio vigente.");
            }
            const totals = lineTotals(
              price.amountMinor,
              quantity,
              price.taxRateBps,
              price.priceIncludesTax,
            );
            return {
              id: uuid(),
              variantId,
              priceVersionId: price.id,
              productName: match.product.name,
              variantName: match.variant.name,
              sku: match.variant.sku,
              unit: match.variant.unit,
              quantity,
              unitAmountMinor: price.amountMinor,
              taxRateBps: price.taxRateBps,
              priceIncludesTax: price.priceIncludesTax,
              subtotalMinor: totals.subtotal,
              taxMinor: totals.tax,
              totalMinor: totals.total,
              refundedQuantity: 0,
              refundedMinor: 0,
            };
          });
          const subtotalMinor = items.reduce((total, item) => total + item.subtotalMinor, 0);
          const taxMinor = items.reduce((total, item) => total + item.taxMinor, 0);
          const totalMinor = items.reduce((total, item) => total + item.totalMinor, 0);
          if (totalMinor > 1_000_000_000) {
            throw apiError("invalidRequest", "El total de la venta es demasiado grande.");
          }
          let cashReceivedMinor: number | null = null;
          let changeMinor: number | null = null;
          if (paymentMethod === "cash") {
            const received = Number(input.cashReceivedMinor ?? totalMinor);
            if (!Number.isInteger(received) || received < totalMinor) {
              throw apiError(
                "invalidRequest",
                "El efectivo recibido debe cubrir el total.",
              );
            }
            if (received > 1_000_000_000) {
              throw apiError("invalidRequest", "El efectivo recibido es demasiado grande.");
            }
            cashReceivedMinor = received;
            changeMinor = received - totalMinor;
          } else if (input.cashReceivedMinor != null) {
            throw apiError(
              "invalidRequest",
              "El efectivo recibido solo aplica a pagos en efectivo.",
            );
          }
          const paymentReference = paymentReferenceFrom(input.paymentReference);
          const requestHash = saleRequestHash(
            items,
            paymentMethod,
            cashReceivedMinor,
            paymentReference,
          );
          const previousAttempt = state.saleIdempotency.get(idempotencyKey);
          if (previousAttempt) {
            if (previousAttempt.requestHash !== requestHash) {
              throw apiError(
                "invalidRequest",
                "La clave de idempotencia ya se usó con otra operación.",
              );
            }
            const previousSale = state.sales.find(
              (entry) => entry.id === previousAttempt.saleId,
            );
            if (previousSale) {
              return previousSale;
            }
          }
          const sale: StoredSale = {
            id: uuid(),
            currency: "MXN",
            subtotalMinor,
            taxMinor,
            totalMinor,
            refundedMinor: 0,
            paymentMethod: paymentMethod as StoredSale["paymentMethod"],
            cashReceivedMinor,
            changeMinor,
            soldBy: session.userId,
            soldByName: session.name,
            cashSessionId: activeCashSession.id,
            paymentReference,
            items,
            createdAt: now(),
          };
          state.sales.unshift(sale);
          state.paymentCharges.unshift({
            id: uuid(),
            saleId: sale.id,
            saleReturnId: null,
            kind: "charge",
            method: sale.paymentMethod,
            provider: sale.paymentMethod === "point" ? "point" : "manual",
            status: "approved",
            amountMinor: sale.totalMinor,
            currency: sale.currency,
            externalReference: paymentReference,
            createdAt: sale.createdAt,
          });
          state.saleIdempotency.set(idempotencyKey, {
            requestHash,
            saleId: sale.id,
          });
          for (const item of sale.items) {
            if (item.unit === "piece") {
              postInventoryMovement({
                variantId: item.variantId,
                movementType: "sale",
                quantityDelta: -item.quantity,
                unitCostMinor: averageCostFor(item.variantId),
                sourceType: "sale",
                sourceId: item.id,
                reference: sale.id,
                authorizedBy: session.userId,
                authorizedByName: session.name,
              });
            }
          }
          recordAudit({
            actor: state.session,
            action: "sale.completed",
            resource: "sale",
            correlationId: sale.id,
          });
          return sale;
        }

        case "list_sale_returns": {
          requirePermission("operatePos");
          const saleId = args.saleId ? String(args.saleId) : null;
          return state.saleReturns
            .filter((saleReturn) => !saleId || saleReturn.saleId === saleId)
            .slice(0, 50)
            .map((saleReturn) => ({
              ...saleReturn,
              items: saleReturn.items.map((item) => ({ ...item })),
            }));
        }

        case "create_sale_return": {
          const session = requirePermission("operatePos");
          const activeCashSession = openCashSession();
          if (!activeCashSession) {
            throw apiError(
              "invalidRequest",
              "Abre la caja antes de registrar una devolución.",
            );
          }
          const input = (args.input ?? {}) as Record<string, unknown>;
          const saleId = String(input.saleId ?? "");
          const sale = state.sales.find((entry) => entry.id === saleId);
          if (!sale) {
            throw apiError("notFound", "La venta no existe.");
          }
          const reason = String(input.reason ?? "").trim();
          if (reason.length < 1 || reason.length > 300) {
            throw apiError(
              "invalidRequest",
              "El motivo de la devolución debe tener entre 1 y 300 caracteres.",
            );
          }
          const rawItems = Array.isArray(input.items) ? input.items : [];
          if (rawItems.length === 0) {
            throw apiError(
              "invalidRequest",
              "La devolución necesita al menos un artículo.",
            );
          }
          if (rawItems.length > 50) {
            throw apiError(
              "invalidRequest",
              "La devolución admite como máximo 50 artículos.",
            );
          }
          const idempotencyKey = String(input.idempotencyKey ?? "").trim();
          if (idempotencyKey.length < 1 || idempotencyKey.length > 128) {
            throw apiError(
              "invalidRequest",
              "La clave de idempotencia debe tener entre 1 y 128 caracteres.",
            );
          }
          const paymentReference = paymentReferenceFrom(input.paymentReference);
          const requestHash = JSON.stringify([
            saleId,
            reason,
            paymentReference,
            rawItems
              .map((rawItem) => {
                const item = (rawItem ?? {}) as Record<string, unknown>;
                return [String(item.saleItemId ?? ""), Number(item.quantity)];
              })
              .sort(([left], [right]) => String(left).localeCompare(String(right))),
          ]);
          const previousAttempt = state.saleReturnIdempotency.get(idempotencyKey);
          if (previousAttempt) {
            if (previousAttempt.requestHash !== requestHash) {
              throw apiError(
                "invalidRequest",
                "La clave de idempotencia ya se usó con otra operación.",
              );
            }
            const previousReturn = state.saleReturns.find(
              (entry) => entry.id === previousAttempt.returnId,
            );
            if (previousReturn) {
              return {
                ...previousReturn,
                items: previousReturn.items.map((item) => ({ ...item })),
              };
            }
          }
          const seen = new Set<string>();
          let totalMinor = 0;
          const returnItems: StoredSaleReturnItem[] = [];
          for (const rawItem of rawItems) {
            const item = (rawItem ?? {}) as Record<string, unknown>;
            const saleItemId = String(item.saleItemId ?? "");
            const quantity = Number(item.quantity);
            if (!Number.isInteger(quantity) || quantity < 1 || quantity > 999) {
              throw apiError(
                "invalidRequest",
                "La cantidad a devolver debe estar entre 1 y 999.",
              );
            }
            if (seen.has(saleItemId)) {
              throw apiError(
                "invalidRequest",
                "El mismo artículo no puede repetirse en la devolución.",
              );
            }
            seen.add(saleItemId);
            const saleItem = sale.items.find((entry) => entry.id === saleItemId);
            if (!saleItem) {
              throw apiError(
                "invalidRequest",
                "El artículo no pertenece a esta venta.",
              );
            }
            const remainingQuantity =
              saleItem.quantity - saleItem.refundedQuantity;
            if (remainingQuantity <= 0) {
              throw apiError(
                "invalidRequest",
                "El artículo ya fue devuelto por completo.",
              );
            }
            if (quantity > remainingQuantity) {
              throw apiError(
                "invalidRequest",
                "La cantidad excede lo disponible por devolver.",
              );
            }
            const remainingAmount = saleItem.totalMinor - saleItem.refundedMinor;
            const amountMinor =
              quantity === remainingQuantity
                ? remainingAmount
                : Math.floor((remainingAmount * quantity) / remainingQuantity);
            if (amountMinor < 0) {
              throw apiError("invalidRequest", "El importe de devolución no es válido.");
            }
            totalMinor += amountMinor;
            returnItems.push({
              id: uuid(),
              saleItemId,
              productName: saleItem.productName,
              variantName: saleItem.variantName,
              sku: saleItem.sku,
              quantity,
              amountMinor,
            });
          }
          if (totalMinor <= 0) {
            throw apiError(
              "invalidRequest",
              "La devolución debe tener un importe mayor que cero.",
            );
          }
          if (totalMinor > 1_000_000_000) {
            throw apiError(
              "invalidRequest",
              "El importe de la devolución es demasiado grande.",
            );
          }
          const saleReturn: StoredSaleReturn = {
            id: uuid(),
            saleId: sale.id,
            cashSessionId: activeCashSession.id,
            currency: sale.currency,
            refundMethod: sale.paymentMethod,
            totalMinor,
            reason,
            authorizedBy: session.userId,
            authorizedByName: session.name,
            paymentReference,
            createdAt: now(),
            items: returnItems,
          };
          state.saleReturns.unshift(saleReturn);
          state.saleReturnIdempotency.set(idempotencyKey, {
            requestHash,
            returnId: saleReturn.id,
          });
          state.paymentCharges.unshift({
            id: uuid(),
            saleId: sale.id,
            saleReturnId: saleReturn.id,
            kind: "refund",
            method: saleReturn.refundMethod,
            provider: saleReturn.refundMethod === "point" ? "point" : "manual",
            status: "approved",
            amountMinor: saleReturn.totalMinor,
            currency: saleReturn.currency,
            externalReference: paymentReference,
            createdAt: saleReturn.createdAt,
          });
          for (const item of returnItems) {
            const saleItem = sale.items.find((entry) => entry.id === item.saleItemId);
            if (!saleItem || saleItem.unit !== "piece") {
              continue;
            }
            const original = state.inventoryMovements.find(
              (movement) =>
                movement.sourceType === "sale" && movement.sourceId === item.saleItemId,
            );
            postInventoryMovement({
              variantId: saleItem.variantId,
              movementType: "return",
              quantityDelta: item.quantity,
              unitCostMinor: original?.unitCostMinor ?? averageCostFor(saleItem.variantId),
              sourceType: "sale_return",
              sourceId: saleReturn.id,
              reference: sale.id,
              note: reason,
              authorizedBy: session.userId,
              authorizedByName: session.name,
            });
          }
          sale.refundedMinor += totalMinor;
          for (const item of returnItems) {
            const saleItem = sale.items.find((entry) => entry.id === item.saleItemId);
            if (saleItem) {
              saleItem.refundedQuantity += item.quantity;
              saleItem.refundedMinor += item.amountMinor;
            }
          }
          recordAudit({
            actor: state.session,
            action: "sale.returned",
            resource: "sale_return",
            correlationId: saleReturn.id,
          });
          return {
            ...saleReturn,
            items: saleReturn.items.map((item) => ({ ...item })),
          };
        }

        case "create_catalog_item": {
          requirePermission("manageCatalog");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const name = String(input.name ?? "").trim();
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre del producto es demasiado corto.");
          }
          if (
            state.catalog.some(
              (product) => product.name.toLowerCase() === name.toLowerCase(),
            )
          ) {
            throw apiError("invalidRequest", "Ya existe un producto con ese nombre.");
          }
          const category = String(input.category ?? "").trim() || null;
          const variantInput = (input.variant ?? {}) as Record<string, unknown>;
          const variant = parseCatalogVariant(variantInput);
          const productId = uuid();
          const variantId = uuid();
          const createdAt = now();
          const price: StoredPrice = {
            id: uuid(),
            variantId,
            currency: "MXN",
            ...variant.price,
            createdAt,
          };
          const product: StoredProduct = {
            id: productId,
            name,
            category,
            status: "active",
            variants: [
              {
                id: variantId,
                name: variant.name,
                sku: variant.sku,
                barcode: variant.barcode,
                unit: variant.unit,
                costMinor: variant.costMinor,
                status: "active",
                createdAt,
                updatedAt: createdAt,
              },
            ],
            createdAt,
            updatedAt: createdAt,
          };
          state.catalog.push(product);
          state.catalogPrices.push(price);
          recordAudit({
            actor: state.session,
            action: "catalog.product_created",
            resource: "catalog_product",
            correlationId: product.id,
          });
          return catalogSnapshot(product);
        }

        case "add_catalog_variant": {
          requirePermission("manageCatalog");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const product = state.catalog.find(
            (item) => item.id === String(input.productId ?? ""),
          );
          if (!product) {
            throw apiError("notFound", "El producto no existe.");
          }
          if (product.status !== "active") {
            throw apiError("invalidRequest", "No puedes agregar variantes a un producto archivado.");
          }
          const variant = parseCatalogVariant(
            (input.variant ?? {}) as Record<string, unknown>,
          );
          const variantId = uuid();
          const createdAt = now();
          product.variants.push({
            id: variantId,
            name: variant.name,
            sku: variant.sku,
            barcode: variant.barcode,
            unit: variant.unit,
            costMinor: variant.costMinor,
            status: "active",
            createdAt,
            updatedAt: createdAt,
          });
          state.catalogPrices.push({
            id: uuid(),
            variantId,
            currency: "MXN",
            ...variant.price,
            createdAt,
          });
          product.updatedAt = createdAt;
          recordAudit({
            actor: state.session,
            action: "catalog.variant_created",
            resource: "catalog_variant",
            correlationId: variantId,
          });
          return catalogSnapshot(product);
        }

        case "update_catalog_product": {
          requirePermission("manageCatalog");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const product = state.catalog.find(
            (item) => item.id === String(input.productId ?? ""),
          );
          if (!product) {
            throw apiError("notFound", "El producto no existe.");
          }
          const name = String(input.name ?? "").trim();
          const status = String(input.status ?? "");
          if (name.length < 2) {
            throw apiError("invalidRequest", "El nombre del producto es demasiado corto.");
          }
          if (status !== "active" && status !== "archived") {
            throw apiError("invalidRequest", "Estado de producto inválido.");
          }
          if (
            state.catalog.some(
              (item) =>
                item.id !== product.id &&
                item.name.toLowerCase() === name.toLowerCase(),
            )
          ) {
            throw apiError("invalidRequest", "Ya existe un producto con ese nombre.");
          }
          product.name = name;
          product.category = String(input.category ?? "").trim() || null;
          product.status = status;
          product.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "catalog.product_updated",
            resource: "catalog_product",
            correlationId: product.id,
          });
          return catalogSnapshot(product);
        }

        case "update_catalog_variant": {
          requirePermission("manageCatalog");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const match = allVariants().find(
            ({ variant }) => variant.id === String(input.variantId ?? ""),
          );
          if (!match) {
            throw apiError("notFound", "La variante no existe.");
          }
          const name = String(input.name ?? "").trim();
          const sku = String(input.sku ?? "").trim();
          const barcode = String(input.barcode ?? "").trim() || null;
          const unit = String(input.unit ?? "");
          const status = String(input.status ?? "");
          const costMinor = Number(input.costMinor);
          if (name.length < 1 || sku.length < 1) {
            throw apiError("invalidRequest", "Nombre y SKU son obligatorios.");
          }
          if (!["piece", "service", "session", "package"].includes(unit)) {
            throw apiError("invalidRequest", "Unidad inválida.");
          }
          if (status !== "active" && status !== "archived") {
            throw apiError("invalidRequest", "Estado de variante inválido.");
          }
          if (!Number.isInteger(costMinor) || costMinor < 0) {
            throw apiError("invalidRequest", "El costo debe ser cero o mayor.");
          }
          if (
            allVariants().some(
              ({ variant }) =>
                variant.id !== match.variant.id &&
                variant.sku.toLowerCase() === sku.toLowerCase(),
            )
          ) {
            throw apiError("invalidRequest", "Ya existe una variante con ese SKU.");
          }
          if (
            barcode &&
            allVariants().some(
              ({ variant }) => variant.id !== match.variant.id && variant.barcode === barcode,
            )
          ) {
            throw apiError("invalidRequest", "Ya existe una variante con ese código de barras.");
          }
          match.variant.name = name;
          match.variant.sku = sku;
          match.variant.barcode = barcode;
          match.variant.unit = unit as StoredVariant["unit"];
          match.variant.costMinor = costMinor;
          match.variant.status = status;
          match.variant.updatedAt = now();
          match.product.updatedAt = match.variant.updatedAt;
          recordAudit({
            actor: state.session,
            action: "catalog.variant_updated",
            resource: "catalog_variant",
            correlationId: match.variant.id,
          });
          return catalogSnapshot(match.product);
        }

        case "add_catalog_price": {
          requirePermission("manageCatalog");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const match = allVariants().find(
            ({ variant }) => variant.id === String(input.variantId ?? ""),
          );
          if (!match) {
            throw apiError("notFound", "La variante no existe.");
          }
          if (match.product.status !== "active" || match.variant.status !== "active") {
            throw apiError(
              "invalidRequest",
              "No puedes agregar precios a un producto o variante archivado.",
            );
          }
          const price = parseCatalogPrice((input.price ?? {}) as Record<string, unknown>);
          const createdAt = now();
          state.catalogPrices.push({
            id: uuid(),
            variantId: match.variant.id,
            currency: "MXN",
            ...price,
            createdAt,
          });
          match.variant.updatedAt = createdAt;
          match.product.updatedAt = createdAt;
          recordAudit({
            actor: state.session,
            action: "catalog.price_added",
            resource: "catalog_variant",
            correlationId: match.variant.id,
          });
          return catalogSnapshot(match.product);
        }

        case "simulate_access": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          if (state.organizations.length === 0) {
            throw apiError(
              "invalidRequest",
              "Configura el gimnasio antes de simular un acceso.",
            );
          }
          const request = (args.request ?? {}) as Record<string, unknown>;
          const credentialId = String(request.credentialId ?? "").trim();
          const deviceId = String(request.deviceId ?? "").trim();
          const idempotencyKey = String(request.idempotencyKey ?? "").trim();
          if (!credentialId || !deviceId || !idempotencyKey) {
            throw apiError("invalidRequest", "La solicitud está incompleta.");
          }

          const requestHash = `${credentialId}\n${deviceId}`;
          const existing = state.accessEntries.get(idempotencyKey);
          if (existing) {
            if (existing.requestHash !== requestHash) {
              throw apiError(
                "invalidRequest",
                "La clave de idempotencia ya se usó para otra solicitud.",
              );
            }
            return {
              eventId: existing.event.eventId,
              allowed: existing.event.allowed,
              reason: "evento ya registrado; se devuelve el resultado original",
              duplicate: true,
              idempotencyKey,
              latencyMs: 3,
              decidedAt: existing.event.occurredAt,
            };
          }

          const credential = state.credentials.find(
            (item) => item.code === credentialId && item.status === "active",
          );
          const member = credential
            ? state.members.find((item) => item.id === credential.memberId)
            : undefined;
          const membership = member
            ? state.memberships.find(
                (item) =>
                  item.memberId === member.id &&
                  ["active", "scheduled"].includes(item.status) &&
                  item.startDate <= dateOnly() &&
                  item.endDate >= dateOnly(),
              )
            : undefined;
          const simulatedAllowed = credentialId === "CRED-ALLOW";
          const allowed = credential
            ? Boolean(member && member.status === "active" && membership)
            : simulatedAllowed;
          const reason = credential
            ? allowed
              ? "credencial válida con membresía vigente"
              : "credencial sin membresía vigente"
            : credentialId === "CRED-DENY"
              ? "credencial simulada sin acceso"
              : "credencial desconocida";
          const event: AccessEvent = {
            eventId: uuid(),
            allowed,
            idempotencyKey,
            occurredAt: now(),
          };
          state.accessEvents.unshift(event);
          state.accessEntries.set(idempotencyKey, { event, requestHash });
          state.accessLog.unshift({
            id: event.eventId,
            credentialCode: credentialId,
            memberId: member?.id ?? null,
            memberNumber: member?.memberNumber ?? null,
            memberName: member
              ? `${member.firstName} ${member.lastName}`
              : null,
            deviceId,
            zone: null,
            allowed,
            reason,
            snapshotRevision: state.snapshots.at(-1)?.revision ?? 0,
            idempotencyKey,
            payloadHash: `sha256:${credentialId}:${deviceId}`,
            reconciliationStatus: "applied",
            occurredAt: event.occurredAt,
          });
          recordAudit({
            actor: state.session,
            action: "access.simulated",
            resource: "access_decision",
            result: allowed ? "allowed" : "denied",
            correlationId: event.eventId,
          });
          return {
            eventId: event.eventId,
            allowed,
            reason,
            duplicate: false,
            idempotencyKey,
            latencyMs: 3,
            decidedAt: event.occurredAt,
          };
        }

        case "list_access_events": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          const limit = Math.max(1, Math.min(100, Number(args.limit ?? 20)));
          return state.accessEvents.slice(0, limit);
        }

        case "list_member_credentials": {
          requirePermission("manageMembers");
          const memberId = String(args.memberId ?? "");
          return state.credentials
            .filter((credential) => credential.memberId === memberId)
            .map((credential) => ({ ...credential }));
        }

        case "issue_member_credential": {
          requirePermission("manageMembers");
          const memberId = String(args.memberId ?? "");
          const member = state.members.find((item) => item.id === memberId);
          if (!member) {
            throw apiError("notFound", "El socio no existe.");
          }
          const credential: StoredCredential = {
            id: uuid(),
            memberId,
            memberNumber: member.memberNumber,
            memberName: `${member.firstName} ${member.lastName}`,
            code: `GYM-${member.memberNumber.replace(/[^A-Z0-9]/g, "").slice(-8)}-${String(
              state.credentials.length + 1,
            ).padStart(2, "0")}`,
            status: "active",
            revision: 1,
            issuedAt: now(),
            revokedAt: null,
          };
          state.credentials.push(credential);
          recordAudit({
            actor: state.session,
            action: "credential.issued",
            resource: "member_credential",
            correlationId: credential.id,
          });
          return { ...credential };
        }

        case "revoke_member_credential": {
          requirePermission("manageMembers");
          const credentialId = String(args.credentialId ?? "");
          const credential = state.credentials.find(
            (item) => item.id === credentialId,
          );
          if (!credential) {
            throw apiError("notFound", "La credencial no existe.");
          }
          if (credential.status !== "revoked") {
            credential.status = "revoked";
            credential.revision += 1;
            credential.revokedAt = now();
          }
          recordAudit({
            actor: state.session,
            action: "credential.revoked",
            resource: "member_credential",
            correlationId: credential.id,
          });
          return { ...credential };
        }

        case "publish_access_snapshot": {
          requirePermission("manageMembers");
          const revision = state.snapshots.length + 1;
          const members = state.members
            .filter((member) => member.status === "active")
            .map((member) => ({
              memberId: member.id,
              memberNumber: member.memberNumber,
              memberName: `${member.firstName} ${member.lastName}`,
              memberStatus: member.status,
              credentials: state.credentials
                .filter((credential) => credential.memberId === member.id)
                .map((credential) => ({
                  code: credential.code,
                  status: credential.status,
                  revision: credential.revision,
                })),
            }));
          const payloadJson = JSON.stringify({
            revision,
            organizationId: state.session?.organizationId ?? null,
            members,
          });
          const snapshot: StoredSnapshot = {
            revision,
            generatedAt: now(),
            memberCount: members.length,
            credentialCount: members.reduce(
              (total, member) => total + member.credentials.length,
              0,
            ),
            checksum: `sha256:${payloadJson.length}`,
            signature: `hmac:${payloadJson.length}`,
            payloadJson,
          };
          state.snapshots.push(snapshot);
          recordAudit({
            actor: state.session,
            action: "snapshot.published",
            resource: "access_snapshot",
            correlationId: String(revision),
          });
          return { ...snapshot };
        }

        case "latest_access_snapshot": {
          requirePermission("manageMembers");
          const snapshot = state.snapshots.at(-1);
          return snapshot ? { ...snapshot } : null;
        }

        case "record_access_event": {
          requirePermission("operatePos");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const idempotencyKey = String(input.idempotencyKey ?? "").trim();
          const existing = state.accessLog.find(
            (event) => event.idempotencyKey === idempotencyKey,
          );
          if (existing) {
            if (
              existing.allowed !== Boolean(input.allowed) ||
              existing.reason !== String(input.reason ?? "")
            ) {
              throw apiError(
                "invalidRequest",
                "La clave de idempotencia ya se usó con otro contenido.",
              );
            }
            return { ...existing };
          }
          const code = String(input.credentialCode ?? "");
          const credential = state.credentials.find(
            (item) => item.code === code,
          );
          const member = credential
            ? state.members.find((item) => item.id === credential.memberId)
            : undefined;
          const event: StoredAccessLog = {
            id: uuid(),
            credentialCode: code,
            memberId: member?.id ?? null,
            memberNumber: member?.memberNumber ?? null,
            memberName: member ? `${member.firstName} ${member.lastName}` : null,
            deviceId: String(input.deviceId ?? ""),
            zone: String(input.zone ?? "").trim() || null,
            allowed: Boolean(input.allowed),
            reason: String(input.reason ?? ""),
            snapshotRevision: Number(input.snapshotRevision ?? 0),
            idempotencyKey,
            payloadHash: `sha256:${code}:${String(input.deviceId ?? "")}`,
            reconciliationStatus:
              Number(input.snapshotRevision ?? 0) <
              (state.snapshots.at(-1)?.revision ?? 0)
                ? "stale"
                : "applied",
            occurredAt: String(input.occurredAt ?? now()),
          };
          state.accessLog.unshift(event);
          return { ...event };
        }

        case "list_access_event_log": {
          requirePermission("manageMembers");
          const limit = Math.max(1, Math.min(100, Number(args.limit ?? 50)));
          return state.accessLog.slice(0, limit).map((event) => ({ ...event }));
        }

        case "reconcile_access_events": {
          requirePermission("operatePos");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const snapshotRevision = Number(input.snapshotRevision ?? 0);
          const events = Array.isArray(input.events) ? input.events : [];
          let inserted = 0;
          let duplicates = 0;
          let stale = 0;
          const conflicts: Array<{ idempotencyKey: string; reason: string }> = [];
          for (const raw of events) {
            const event = raw as Record<string, unknown>;
            const key = String(event.idempotencyKey ?? "");
            const existing = state.accessLog.find(
              (item) => item.idempotencyKey === key,
            );
            if (existing) {
              if (
                existing.allowed !== Boolean(event.allowed) ||
                existing.reason !== String(event.reason ?? "")
              ) {
                conflicts.push({
                  idempotencyKey: key,
                  reason:
                    "la clave de idempotencia ya se usó con otro contenido",
                });
              } else {
                duplicates += 1;
              }
              continue;
            }
            const code = String(event.credentialCode ?? "");
            const credential = state.credentials.find(
              (item) => item.code === code,
            );
            const member = credential
              ? state.members.find((item) => item.id === credential.memberId)
              : undefined;
            const isStale =
              snapshotRevision < (state.snapshots.at(-1)?.revision ?? 0);
            state.accessLog.unshift({
              id: uuid(),
              credentialCode: code,
              memberId: member?.id ?? null,
              memberNumber: member?.memberNumber ?? null,
              memberName: member
                ? `${member.firstName} ${member.lastName}`
                : null,
              deviceId: String(event.deviceId ?? ""),
              zone: String(event.zone ?? "").trim() || null,
              allowed: Boolean(event.allowed),
              reason: String(event.reason ?? ""),
              snapshotRevision,
              idempotencyKey: key,
              payloadHash: `sha256:${code}:${String(event.deviceId ?? "")}`,
              reconciliationStatus: isStale ? "stale" : "applied",
              occurredAt: String(event.occurredAt ?? now()),
            });
            inserted += 1;
            if (isStale) {
              stale += 1;
            }
          }
          return {
            snapshotRevision,
            inserted,
            duplicates,
            stale,
            conflicts,
          };
        }

        case "attendance_report": {
          requirePermission("manageMembers");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const from = String(input.from ?? "");
          const to = String(input.to ?? "");
          const allowed = state.accessLog.filter(
            (event) =>
              event.allowed &&
              event.reconciliationStatus !== "legacy" &&
              event.occurredAt >= from &&
              event.occurredAt < to,
          );
          const hours = Array.from({ length: 24 }, (_, hour) => {
            const visits = allowed.filter(
              (event) =>
                new Date(event.occurredAt).getUTCHours() === hour,
            );
            return {
              hour,
              visits: visits.length,
              uniqueMembers: new Set(
                visits.map((event) => event.memberId).filter(Boolean),
              ).size,
            };
          });
          let peakHour = 0;
          let peakVisits = -1;
          for (const hour of hours) {
            if (hour.visits > peakVisits) {
              peakVisits = hour.visits;
              peakHour = hour.hour;
            }
          }
          const lowDemandHours = hours.filter(
            (hour) => peakVisits > 0 && hour.visits <= peakVisits * 0.5,
          );
          const days = Array.from(
            new Set(allowed.map((event) => event.occurredAt.slice(0, 10))),
          ).map((date) => {
            const visits = allowed.filter(
              (event) => event.occurredAt.slice(0, 10) === date,
            );
            return {
              date,
              visits: visits.length,
              uniqueMembers: new Set(
                visits.map((event) => event.memberId).filter(Boolean),
              ).size,
            };
          });
          const zones = Array.from(
            new Set(allowed.map((event) => event.zone ?? "Sin zona")),
          ).map((zone) => {
            const visits = allowed.filter(
              (event) => (event.zone ?? "Sin zona") === zone,
            );
            return {
              zone,
              visits: visits.length,
              uniqueMembers: new Set(
                visits.map((event) => event.memberId).filter(Boolean),
              ).size,
            };
          });
          const members = Array.from(
            new Map(
              allowed
                .filter((event) => event.memberId)
                .map((event) => [event.memberId, event]),
            ).values(),
          ).map((event) => ({
            memberId: event.memberId!,
            memberNumber: event.memberNumber ?? "",
            memberName: event.memberName ?? "",
            visits: allowed.filter(
              (item) => item.memberId === event.memberId,
            ).length,
            lastVisitAt: event.occurredAt,
          }));
          return {
            from,
            to,
            generatedAt: now(),
            totalVisits: allowed.length,
            uniqueMembers: new Set(
              allowed.map((event) => event.memberId).filter(Boolean),
            ).size,
            peakHour,
            lowDemandHours,
            hours,
            days,
            plans: [],
            zones,
            members,
          };
        }

        case "create_report_schedule": {
          requirePermission("viewFinance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const cadence = String(input.cadence ?? "");
          if (!["daily", "weekly", "monthly"].includes(cadence)) {
            throw apiError("invalidRequest", "Cadencia inválida.");
          }
          const schedule: StoredReportSchedule = {
            id: uuid(),
            reportType: "financial",
            cadence: cadence as StoredReportSchedule["cadence"],
            lookbackDays: Math.max(
              1,
              Math.min(366, Number(input.lookbackDays ?? 7)),
            ),
            enabled: true,
            nextRunAt: now(),
            lastRunAt: null,
            createdAt: now(),
            updatedAt: now(),
          };
          state.reportSchedules.push(schedule);
          return { ...schedule };
        }

        case "list_report_schedules": {
          requirePermission("viewFinance");
          return state.reportSchedules.map((schedule) => ({ ...schedule }));
        }

        case "set_report_schedule_enabled": {
          requirePermission("viewFinance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const schedule = state.reportSchedules.find(
            (item) => item.id === String(input.scheduleId ?? ""),
          );
          if (!schedule) {
            throw apiError("notFound", "La programación no existe.");
          }
          schedule.enabled = Boolean(input.enabled);
          schedule.updatedAt = now();
          return { ...schedule };
        }

        case "list_report_runs": {
          requirePermission("viewFinance");
          const limit = Math.max(1, Math.min(100, Number(args.limit ?? 50)));
          return state.reportRuns.slice(0, limit).map((run) => ({ ...run }));
        }

        case "run_due_report_schedules": {
          requirePermission("viewFinance");
          const current = now();
          const due = state.reportSchedules.filter(
            (schedule) => schedule.enabled && schedule.nextRunAt <= current,
          );
          const generated: StoredReportRun[] = [];
          for (const schedule of due) {
            const periodTo = current;
            const periodFrom = new Date(
              Date.now() - schedule.lookbackDays * 24 * 60 * 60 * 1_000,
            ).toISOString();
            const fileName = `reporte-${periodFrom.slice(0, 10)}-${periodTo.slice(0, 10)}-${Date.now()}.csv`;
            const run: StoredReportRun = {
              id: uuid(),
              scheduleId: schedule.id,
              reportType: schedule.reportType,
              periodFrom,
              periodTo,
              generatedAt: current,
              fileName,
              checksum: `sha256:${fileName.length}`,
              status: "completed",
              error: null,
            };
            state.reportRuns.unshift(run);
            generated.push(run);
            schedule.lastRunAt = current;
            const days =
              schedule.cadence === "daily"
                ? 1
                : schedule.cadence === "weekly"
                  ? 7
                  : 30;
            schedule.nextRunAt = new Date(
              Date.now() + days * 24 * 60 * 60 * 1_000,
            ).toISOString();
            schedule.updatedAt = current;
          }
          return generated.map((run) => ({ ...run }));
        }

        case "list_backups":
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          return state.backups.map((backup) => ({ ...backup }));

        case "create_backup": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          const passphrase = String(args.passphrase ?? "");
          if (passphrase.trim().length < 8) {
            throw apiError("backup", "La frase debe tener al menos 8 caracteres.");
          }
          const createdAt = now();
          const fileName = `fitness-gym-${createdAt.replace(/[-:.]/g, "")}.gymbackup`;
          const backup = {
            id: uuid(),
            fileName,
            createdAt,
            sizeBytes: 118784,
            checksum: "9f2c4e7a1b8d3f5061728394a5b6c7d8e9f00112233445566778899aabbccddee",
            schemaVersion: 21,
          };
          state.backups.unshift(backup);
          state.secrets.set(fileName, passphrase);
          recordAudit({
            actor: state.session,
            action: "backup.created",
            resource: "backup",
            correlationId: backup.id,
          });
          return { ...backup };
        }

        case "restore_backup": {
          if (!state.session) {
            throw apiError("unauthorized", "Inicia sesión para continuar.");
          }
          const fileName = String(args.fileName ?? "");
          const backup = state.backups.find((item) => item.fileName === fileName);
          if (!backup) {
            throw apiError("notFound", `No existe el backup ${fileName}.`);
          }
          if (state.secrets.get(fileName) !== String(args.passphrase ?? "")) {
            throw apiError(
              "backup",
              "No se pudo descifrar el backup: frase incorrecta o archivo dañado.",
            );
          }
          state.session = null;
          return {
            backupId: backup.id,
            restoredAt: now(),
            schemaVersion: backup.schemaVersion,
            integrityOk: true,
          };
        }

        case "list_audit_events": {
          requirePermission("viewAudit");
          const limit = Math.max(1, Math.min(200, Number(args.limit ?? 100)));
          return state.auditEvents.slice(0, limit).map((event) => ({ ...event }));
        }

        case "verify_audit_chain": {
          requirePermission("viewAudit");
          const ordered = [...state.auditEvents].sort((left, right) => left.sequence - right.sequence);
          const valid = ordered.every(
            (event, index) => index === 0 || event.sequence === ordered[index - 1]!.sequence + 1,
          );
          return {
            valid,
            checkedEvents: ordered.length,
            firstBrokenSequence: valid
              ? null
              : ordered.find(
                  (event, index) =>
                    index > 0 && event.sequence !== ordered[index - 1]!.sequence + 1,
                )?.sequence ?? null,
          };
        }

        case "get_gym_settings": {
          requirePermission("viewSettings");
          if (!state.gymSettings) {
            state.gymSettings = {
              id: uuid(),
              businessName: state.organizations[0]?.name ?? "Fitness Gym",
              legalName: null,
              taxId: null,
              currency: "MXN",
              timezone: "America/Mexico_City",
              locale: "es-MX",
              openingTime: "06:00",
              closingTime: "22:00",
              slotMinutes: 60,
              graceMinutes: 10,
              inactivityDays: 30,
              frozenDays: 0,
              updatedAt: now(),
            };
          }
          return { ...state.gymSettings };
        }

        case "update_gym_settings": {
          requirePermission("manageSettings");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const businessName = String(input.businessName ?? "").trim();
          if (!businessName) {
            throw apiError("invalidRequest", "El nombre comercial es obligatorio.");
          }
          const slotMinutes = Number(input.slotMinutes ?? 60);
          const graceMinutes = Number(input.graceMinutes ?? 10);
          const inactivityDays = Number(input.inactivityDays ?? 30);
          const frozenDays = Number(input.frozenDays ?? 0);
          if (slotMinutes < 15 || slotMinutes > 240) {
            throw apiError("invalidRequest", "La duración de turno debe estar entre 15 y 240 minutos.");
          }
          if (graceMinutes < 0 || graceMinutes > 120) {
            throw apiError("invalidRequest", "La tolerancia debe estar entre 0 y 120 minutos.");
          }
          if (inactivityDays < 1 || inactivityDays > 365) {
            throw apiError("invalidRequest", "La inactividad debe estar entre 1 y 365 días.");
          }
          if (frozenDays < 0 || frozenDays > 365) {
            throw apiError("invalidRequest", "Los días congelados deben estar entre 0 y 365.");
          }
          state.gymSettings = {
            id: state.gymSettings?.id ?? uuid(),
            businessName,
            legalName: String(input.legalName ?? "").trim() || null,
            taxId: String(input.taxId ?? "").trim() || null,
            currency: String(input.currency ?? "MXN").trim() || "MXN",
            timezone: String(input.timezone ?? "America/Mexico_City").trim(),
            locale: String(input.locale ?? "es-MX").trim(),
            openingTime: String(input.openingTime ?? "06:00").trim(),
            closingTime: String(input.closingTime ?? "22:00").trim(),
            slotMinutes,
            graceMinutes,
            inactivityDays,
            frozenDays,
            updatedAt: now(),
          };
          recordAudit({
            actor: state.session,
            action: "settings.updated",
            resource: "gym_settings",
          });
          return { ...state.gymSettings };
        }

        case "list_integration_links": {
          requirePermission("viewIntegrations");
          return state.integrationLinks.map((link) => ({
            ...link,
            capabilities: [...link.capabilities],
          }));
        }

        case "upsert_integration_link": {
          requirePermission("manageIntegrations");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const moduleId = String(input.moduleId ?? "").trim();
          const displayName = String(input.displayName ?? "").trim();
          const contractVersion = String(input.contractVersion ?? "").trim();
          const status = String(input.status ?? "connected").trim();
          if (!moduleId || !displayName || !contractVersion) {
            throw apiError("invalidRequest", "Los datos de la integración están incompletos.");
          }
          if (!["connected", "degraded", "disconnected"].includes(status)) {
            throw apiError("invalidRequest", "Estado de integración inválido.");
          }
          const capabilities = Array.isArray(input.capabilities)
            ? input.capabilities.map((value) => String(value).trim()).filter(Boolean).slice(0, 12)
            : [];
          const existing = state.integrationLinks.find((link) => link.moduleId === moduleId);
          const updatedAt = now();
          const link: IntegrationLink = {
            id: existing?.id ?? uuid(),
            moduleId,
            displayName,
            contractVersion,
            status,
            capabilities,
            lastSeenAt: existing?.lastSeenAt ?? updatedAt,
            lastEventAt: existing?.lastEventAt ?? null,
            lastSnapshotAt: existing?.lastSnapshotAt ?? null,
            notes: String(input.notes ?? "").trim() || null,
            createdAt: existing?.createdAt ?? updatedAt,
            updatedAt,
          };
          if (existing) {
            Object.assign(existing, link);
          } else {
            state.integrationLinks.push(link);
          }
          recordAudit({
            actor: state.session,
            action: "integration.linked",
            resource: "integration_link",
            correlationId: link.id,
          });
          return { ...link, capabilities: [...link.capabilities] };
        }

        case "integration_journal_summary": {
          requirePermission("viewIntegrations");
          return {
            pending: 0,
            sent: 0,
            deadLetter: 0,
            inboxEvents: 0,
            lastServerSequence: 0,
          };
        }

        case "list_classes": {
          requirePermission("viewClasses");
          return state.classes.map((item) => ({ ...item }));
        }

        case "create_class": {
          requirePermission("manageClasses");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const name = String(input.name ?? "").trim();
          const weekday = Number(input.weekday ?? 0);
          const startTime = String(input.startTime ?? "").trim();
          const durationMinutes = Number(input.durationMinutes ?? 60);
          const capacity = Number(input.capacity ?? 1);
          if (!name) {
            throw apiError("invalidRequest", "El nombre de la clase es obligatorio.");
          }
          if (weekday < 0 || weekday > 6) {
            throw apiError("invalidRequest", "El día de la semana es inválido.");
          }
          if (!startTime) {
            throw apiError("invalidRequest", "La hora de inicio es obligatoria.");
          }
          if (durationMinutes < 15 || durationMinutes > 480) {
            throw apiError("invalidRequest", "La duración debe estar entre 15 y 480 minutos.");
          }
          if (capacity < 1 || capacity > 500) {
            throw apiError("invalidRequest", "La capacidad debe estar entre 1 y 500.");
          }
          const coachUserId = String(input.coachUserId ?? "").trim() || null;
          const coachName = coachUserId
            ? state.staff.find((member) => member.id === coachUserId)?.name ?? null
            : null;
          const createdAt = now();
          const session: ClassSession = {
            id: uuid(),
            name,
            coachUserId,
            coachName,
            weekday,
            startTime,
            durationMinutes,
            capacity,
            room: String(input.room ?? "").trim() || null,
            status: "active",
            notes: String(input.notes ?? "").trim() || null,
            booked: 0,
            attended: 0,
            createdAt,
            updatedAt: createdAt,
          };
          state.classes.push(session);
          recordAudit({
            actor: state.session,
            action: "class.created",
            resource: "class",
            correlationId: session.id,
          });
          return { ...session };
        }

        case "update_class": {
          requirePermission("manageClasses");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const session = state.classes.find(
            (item) => item.id === String(input.classId ?? ""),
          );
          if (!session) {
            throw apiError("notFound", "La clase no existe.");
          }
          const name = String(input.name ?? "").trim();
          const status = String(input.status ?? "");
          const capacity = Number(input.capacity ?? session.capacity);
          if (!name) {
            throw apiError("invalidRequest", "El nombre de la clase es obligatorio.");
          }
          if (!["active", "paused", "cancelled"].includes(status)) {
            throw apiError("invalidRequest", "Estado de clase inválido.");
          }
          if (capacity < session.booked) {
            throw apiError("invalidRequest", "La capacidad no puede ser menor a las reservas actuales.");
          }
          const coachUserId = String(input.coachUserId ?? "").trim() || null;
          session.name = name;
          session.coachUserId = coachUserId;
          session.coachName = coachUserId
            ? state.staff.find((member) => member.id === coachUserId)?.name ?? null
            : null;
          session.weekday = Number(input.weekday ?? session.weekday);
          session.startTime = String(input.startTime ?? session.startTime).trim();
          session.durationMinutes = Number(input.durationMinutes ?? session.durationMinutes);
          session.capacity = capacity;
          session.room = String(input.room ?? "").trim() || null;
          session.status = status;
          session.notes = String(input.notes ?? "").trim() || null;
          session.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "class.updated",
            resource: "class",
            correlationId: session.id,
          });
          return { ...session };
        }

        case "list_class_bookings": {
          requirePermission("viewClasses");
          const classId = String(args.classId ?? "");
          return state.classBookings
            .filter((booking) => booking.classId === classId)
            .map((booking) => ({ ...booking }));
        }

        case "book_class": {
          requirePermission("manageReservations");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const classId = String(input.classId ?? "");
          const memberId = String(input.memberId ?? "");
          const session = state.classes.find((item) => item.id === classId);
          if (!session) {
            throw apiError("notFound", "La clase no existe.");
          }
          if (session.status !== "active") {
            throw apiError("invalidRequest", "Solo las clases activas aceptan reservas.");
          }
          const member = state.members.find((item) => item.id === memberId);
          if (!member) {
            throw apiError("notFound", "El socio no existe.");
          }
          const existing = state.classBookings.find(
            (booking) => booking.classId === classId && booking.memberId === memberId,
          );
          if (existing && existing.status !== "cancelled") {
            throw apiError("invalidRequest", "El socio ya tiene una reserva en esta clase.");
          }
          const activeBookings = state.classBookings.filter(
            (booking) =>
              booking.classId === classId &&
              (booking.status === "booked" || booking.status === "attended"),
          ).length;
          if (activeBookings >= session.capacity) {
            throw apiError("invalidRequest", "La clase no tiene cupo disponible.");
          }
          const bookedAt = now();
          if (existing) {
            existing.status = "booked";
            existing.note = String(input.note ?? "").trim() || null;
            existing.bookedAt = bookedAt;
            existing.updatedAt = bookedAt;
            session.booked += 1;
          } else {
            const booking: ClassBooking = {
              id: uuid(),
              classId,
              className: session.name,
              memberId,
              memberName: `${member.firstName} ${member.lastName}`.trim(),
              status: "booked",
              note: String(input.note ?? "").trim() || null,
              bookedAt,
              updatedAt: bookedAt,
            };
            state.classBookings.push(booking);
            session.booked += 1;
          }
          recordAudit({
            actor: state.session,
            action: "class.booking.created",
            resource: "class_booking",
          });
          const booking = state.classBookings.find(
            (item) => item.classId === classId && item.memberId === memberId,
          )!;
          return { ...booking };
        }

        case "update_class_booking": {
          requirePermission("manageReservations");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const booking = state.classBookings.find(
            (item) => item.id === String(input.bookingId ?? ""),
          );
          if (!booking) {
            throw apiError("notFound", "La reserva no existe.");
          }
          const status = String(input.status ?? "");
          if (!["booked", "attended", "cancelled", "no_show"].includes(status)) {
            throw apiError("invalidRequest", "Estado de reserva inválido.");
          }
          const previous = booking.status;
          booking.status = status;
          booking.updatedAt = now();
          const session = state.classes.find((item) => item.id === booking.classId);
          if (session) {
            const counted = (value: string) => value === "booked" || value === "attended";
            if (counted(previous) && !counted(status)) {
              session.booked = Math.max(0, session.booked - 1);
            }
            if (!counted(previous) && counted(status)) {
              session.booked += 1;
            }
            if (status === "attended" && previous !== "attended") {
              session.attended += 1;
            } else if (previous === "attended" && status !== "attended") {
              session.attended = Math.max(0, session.attended - 1);
            }
          }
          recordAudit({
            actor: state.session,
            action: "class.booking.updated",
            resource: "class_booking",
            correlationId: booking.id,
          });
          return { ...booking };
        }

        case "list_campaigns": {
          requirePermission("viewMarketing");
          return state.campaigns.map((campaign) => ({ ...campaign }));
        }

        case "create_campaign": {
          requirePermission("manageMarketing");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const name = String(input.name ?? "").trim();
          const channel = String(input.channel ?? "").trim();
          const audience = String(input.audience ?? "").trim();
          const status = String(input.status ?? "draft").trim();
          if (!name || !audience) {
            throw apiError("invalidRequest", "El nombre y la audiencia son obligatorios.");
          }
          if (!["whatsapp", "sms", "email", "in_app", "referral"].includes(channel)) {
            throw apiError("invalidRequest", "Canal de campaña inválido.");
          }
          if (!["draft", "active", "paused", "completed"].includes(status)) {
            throw apiError("invalidRequest", "Estado de campaña inválido.");
          }
          const createdAt = now();
          const campaign: Campaign = {
            id: uuid(),
            name,
            channel,
            audience,
            status,
            startsOn: String(input.startsOn ?? "").trim() || null,
            endsOn: String(input.endsOn ?? "").trim() || null,
            notes: String(input.notes ?? "").trim() || null,
            createdAt,
            updatedAt: createdAt,
          };
          state.campaigns.push(campaign);
          recordAudit({
            actor: state.session,
            action: "campaign.created",
            resource: "campaign",
            correlationId: campaign.id,
          });
          return { ...campaign };
        }

        case "update_campaign": {
          requirePermission("manageMarketing");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const campaign = state.campaigns.find(
            (item) => item.id === String(input.campaignId ?? ""),
          );
          if (!campaign) {
            throw apiError("notFound", "La campaña no existe.");
          }
          const name = String(input.name ?? "").trim();
          const channel = String(input.channel ?? "").trim();
          const audience = String(input.audience ?? "").trim();
          const status = String(input.status ?? "");
          if (!name || !audience) {
            throw apiError("invalidRequest", "El nombre y la audiencia son obligatorios.");
          }
          if (!["whatsapp", "sms", "email", "in_app", "referral"].includes(channel)) {
            throw apiError("invalidRequest", "Canal de campaña inválido.");
          }
          if (!["draft", "active", "paused", "completed"].includes(status)) {
            throw apiError("invalidRequest", "Estado de campaña inválido.");
          }
          campaign.name = name;
          campaign.channel = channel;
          campaign.audience = audience;
          campaign.status = status;
          campaign.startsOn = String(input.startsOn ?? "").trim() || null;
          campaign.endsOn = String(input.endsOn ?? "").trim() || null;
          campaign.notes = String(input.notes ?? "").trim() || null;
          campaign.updatedAt = now();
          recordAudit({
            actor: state.session,
            action: "campaign.updated",
            resource: "campaign",
            correlationId: campaign.id,
          });
          return { ...campaign };
        }

        case "marketing_funnel": {
          requirePermission("viewMarketing");
          const inactivityDays = Math.max(
            1,
            Math.min(365, Number(args.inactivityDays ?? state.gymSettings?.inactivityDays ?? 30)),
          );
          const expiryDays = Math.max(1, Math.min(365, Number(args.expiryDays ?? 30)));
          const stageOrder = ["new", "contacted", "trial", "converted", "lost"];
          const stages = stageOrder.map((status) => ({
            status,
            count: state.prospects.filter((prospect) => prospect.status === status).length,
          }));
          const total = stages.reduce((sum, stage) => sum + stage.count, 0);
          const converted = stages.find((stage) => stage.status === "converted")?.count ?? 0;
          const conversionBps = total === 0 ? 0 : Math.round((converted * 10000) / total);
          const today = new Date();
          const expiryLimit = new Date(today);
          expiryLimit.setDate(expiryLimit.getDate() + expiryDays);
          const expiryLimitIso = expiryLimit.toISOString().slice(0, 10);
          const todayIso = today.toISOString().slice(0, 10);
          const expiringMemberships = state.memberships.filter(
            (membership) =>
              membership.status === "active" &&
              membership.endDate >= todayIso &&
              membership.endDate <= expiryLimitIso,
          ).length;
          const inactivityCutoff = new Date(today);
          inactivityCutoff.setDate(inactivityCutoff.getDate() - inactivityDays);
          const cutoffIso = inactivityCutoff.toISOString();
          const activeMembers = state.members.filter((member) => member.status === "active");
          const inactiveMembers = activeMembers.filter(
            (member) =>
              !state.accessLog.some(
                (entry) =>
                  entry.memberId === member.id &&
                  entry.allowed &&
                  entry.occurredAt >= cutoffIso,
              ),
          ).length;
          const funnel: MarketingFunnel = {
            prospectsTotal: total,
            prospectsConverted: converted,
            conversionBps,
            stages,
            activeCampaigns: state.campaigns.filter((campaign) => campaign.status === "active")
              .length,
            expiringMemberships,
            inactiveMembers,
          };
          return funnel;
        }

        case "list_equipment": {
          requirePermission("viewMaintenance");
          return state.equipment.map((item) => ({ ...item }));
        }

        case "create_equipment": {
          requirePermission("manageMaintenance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const name = String(input.name ?? "").trim();
          if (!name) {
            throw apiError("invalidRequest", "El nombre del equipo es obligatorio.");
          }
          const createdAt = now();
          const item: Equipment = {
            id: uuid(),
            name,
            code: String(input.code ?? "").trim() || null,
            area: String(input.area ?? "").trim() || null,
            status: "operational",
            acquiredOn: String(input.acquiredOn ?? "").trim() || null,
            notes: String(input.notes ?? "").trim() || null,
            openOrders: 0,
            createdAt,
            updatedAt: createdAt,
          };
          state.equipment.push(item);
          recordAudit({
            actor: state.session,
            action: "equipment.created",
            resource: "equipment",
            correlationId: item.id,
          });
          return { ...item };
        }

        case "update_equipment": {
          requirePermission("manageMaintenance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const item = state.equipment.find(
            (equipment) => equipment.id === String(input.equipmentId ?? ""),
          );
          if (!item) {
            throw apiError("notFound", "El equipo no existe.");
          }
          const status = String(input.status ?? "");
          if (!["operational", "maintenance", "out_of_service"].includes(status)) {
            throw apiError("invalidRequest", "Estado de equipo inválido.");
          }
          item.name = String(input.name ?? "").trim() || item.name;
          item.code = String(input.code ?? "").trim() || null;
          item.area = String(input.area ?? "").trim() || null;
          item.status = status;
          item.acquiredOn = String(input.acquiredOn ?? "").trim() || null;
          item.notes = String(input.notes ?? "").trim() || null;
          item.updatedAt = now();
          item.openOrders = state.maintenanceOrders.filter(
            (order) =>
              order.equipmentId === item.id &&
              (order.status === "open" || order.status === "in_progress"),
          ).length;
          recordAudit({
            actor: state.session,
            action: "equipment.updated",
            resource: "equipment",
            correlationId: item.id,
          });
          return { ...item };
        }

        case "list_maintenance_orders": {
          requirePermission("viewMaintenance");
          const status = String(args.status ?? "").trim();
          return state.maintenanceOrders
            .filter((order) => !status || order.status === status)
            .map((order) => ({ ...order }));
        }

        case "create_maintenance_order": {
          requirePermission("manageMaintenance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const kind = String(input.kind ?? "").trim();
          const priority = String(input.priority ?? "normal").trim();
          const summary = String(input.summary ?? "").trim();
          if (!["preventive", "corrective"].includes(kind)) {
            throw apiError("invalidRequest", "Tipo de orden inválido.");
          }
          if (!["low", "normal", "high"].includes(priority)) {
            throw apiError("invalidRequest", "Prioridad de orden inválida.");
          }
          if (!summary) {
            throw apiError("invalidRequest", "El resumen de la orden es obligatorio.");
          }
          const equipmentId = String(input.equipmentId ?? "").trim() || null;
          const equipment = equipmentId
            ? state.equipment.find((item) => item.id === equipmentId)
            : null;
          if (equipmentId && !equipment) {
            throw apiError("notFound", "El equipo no existe.");
          }
          const createdAt = now();
          const order: MaintenanceOrder = {
            id: uuid(),
            equipmentId,
            equipmentName: equipment?.name ?? null,
            kind,
            status: "open",
            priority,
            summary,
            detail: String(input.detail ?? "").trim() || null,
            dueOn: String(input.dueOn ?? "").trim() || null,
            reportedBy: String(input.reportedBy ?? "").trim() || state.session?.name || null,
            completedAt: null,
            createdAt,
            updatedAt: createdAt,
          };
          state.maintenanceOrders.push(order);
          if (equipment) {
            equipment.openOrders += 1;
          }
          recordAudit({
            actor: state.session,
            action: "maintenance.order.created",
            resource: "maintenance_order",
            correlationId: order.id,
          });
          return { ...order };
        }

        case "update_maintenance_order": {
          requirePermission("manageMaintenance");
          const input = (args.input ?? {}) as Record<string, unknown>;
          const order = state.maintenanceOrders.find(
            (item) => item.id === String(input.orderId ?? ""),
          );
          if (!order) {
            throw apiError("notFound", "La orden de mantenimiento no existe.");
          }
          const status = String(input.status ?? "");
          const priority = String(input.priority ?? "");
          const summary = String(input.summary ?? "").trim();
          if (!["open", "in_progress", "completed", "cancelled"].includes(status)) {
            throw apiError("invalidRequest", "Estado de orden inválido.");
          }
          if (!["low", "normal", "high"].includes(priority)) {
            throw apiError("invalidRequest", "Prioridad de orden inválida.");
          }
          if (!summary) {
            throw apiError("invalidRequest", "El resumen de la orden es obligatorio.");
          }
          const wasOpen = order.status === "open" || order.status === "in_progress";
          order.status = status;
          order.priority = priority;
          order.summary = summary;
          order.detail = String(input.detail ?? "").trim() || null;
          order.dueOn = String(input.dueOn ?? "").trim() || null;
          order.completedAt = status === "completed" ? now() : null;
          order.updatedAt = now();
          const isOpen = status === "open" || status === "in_progress";
          const equipment = order.equipmentId
            ? state.equipment.find((item) => item.id === order.equipmentId)
            : null;
          if (equipment) {
            equipment.openOrders = Math.max(
              0,
              equipment.openOrders + (isOpen ? 1 : 0) - (wasOpen ? 1 : 0),
            );
          }
          recordAudit({
            actor: state.session,
            action: "maintenance.order.updated",
            resource: "maintenance_order",
            correlationId: order.id,
          });
          return { ...order };
        }

        default:
          throw apiError("internal", `Comando no simulado: ${command}`);
      }
    },
    transformCallback: () => 1,
    unregisterCallback: () => {},
    convertFileSrc: (path: string) => path,
  };

  (window as unknown as { __TAURI_INTERNALS__: typeof internals }).__TAURI_INTERNALS__ =
    internals;
})();

export {};
