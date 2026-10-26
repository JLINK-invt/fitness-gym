type Organization = { id: string; name: string };
type Owner = { id: string; name: string; password: string; recoveryCode: string };
type Session = {
  userId: string;
  organizationId: string;
  name: string;
  role: "owner" | "manager" | "cashier" | "accounting";
  permissions: string[];
};
type AccessEvent = {
  eventId: string;
  allowed: boolean;
  idempotencyKey: string;
  occurredAt: string;
};
type AccessEntry = { event: AccessEvent; requestHash: string };
type StaffMember = {
  id: string;
  name: string;
  password: string;
  role: "manager" | "cashier" | "accounting";
  status: "active" | "revoked";
  lastLoginAt: string | null;
  createdAt: string;
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
  status: "scheduled" | "active" | "paused" | "expired" | "cancelled";
  startDate: string;
  endDate: string;
  priceMinor: number;
  currency: string;
  graceDays: number;
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
    memberSequence: number;
  } = {
    organizations: [],
    owner: null,
    staff: [],
    session: null,
    accessEvents: [],
    accessEntries: new Map(),
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
    memberSequence: 0,
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
  const permissionsFor = (role: Session["role"]) => {
    switch (role) {
      case "owner":
        return [
          "manageSettings",
          "manageStaff",
          "viewFinance",
          "operatePos",
          "manageInventory",
          "manageCatalog",
          "manageBackups",
          "manageMembers",
          "viewAudit",
        ];
      case "manager":
        return [
          "manageStaff",
          "viewFinance",
          "operatePos",
          "manageInventory",
          "manageCatalog",
          "manageBackups",
          "manageMembers",
          "viewAudit",
        ];
      case "cashier":
        return ["operatePos", "manageInventory", "manageMembers"];
      case "accounting":
        return ["viewFinance", "viewAudit"];
    }
  };
  const requireSession = () => {
    if (!state.session) {
      throw apiError("unauthorized", "Inicia sesión para continuar.");
    }
    return state.session;
  };
  const requirePermission = (permission: string) => {
    const session = requireSession();
    if (!session.permissions.includes(permission)) {
      throw apiError("unauthorized", "Tu rol no permite realizar esta acción.");
    }
    return session;
  };
  const requireAnyPermission = (permissions: string[]) => {
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
        }
      : null;
  const staffSummary = (member: StaffMember) => ({
    userId: member.id,
    name: member.name,
    role: member.role,
    status: member.status,
    lastLoginAt: member.lastLoginAt,
    createdAt: member.createdAt,
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
  const dashboardSnapshot = () => {
    const today = dateOnly();
    const monthStart = `${today.slice(0, 7)}-01`;
    const soon = addDays(today, 7);
    return {
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
      newMembersThisMonth: state.members.filter((member) => member.joinedAt >= monthStart).length,
    };
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
    contractVersion: "2.12.0",
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
      schemaVersion: 17,
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
              name: owner.name,
              role: "owner",
              permissions: permissionsFor("owner"),
            };
          } else if (member) {
            member.lastLoginAt = now();
            state.session = {
              userId: member.id,
              organizationId: state.organizations[0]!.id,
              name: member.name,
              role: member.role,
              permissions: permissionsFor(member.role),
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
            (member) =>
              session.role !== "manager" ||
              member.role === "cashier" ||
              member.role === "accounting",
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
          if (!["manager", "cashier", "accounting"].includes(role)) {
            throw apiError("invalidRequest", "Rol inválido.");
          }
          if (
            state.owner?.name.toLowerCase() === name.toLowerCase() ||
            state.staff.some((member) => member.name.toLowerCase() === name.toLowerCase())
          ) {
            throw apiError("invalidRequest", "Ya existe una persona con ese nombre.");
          }
          if (session.role === "manager" && role === "manager") {
            throw apiError("unauthorized", "Un encargado no puede crear otro encargado.");
          }
          const member: StaffMember = {
            id: uuid(),
            name,
            password,
            role: role as StaffMember["role"],
            status: "active",
            lastLoginAt: null,
            createdAt: now(),
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
          if (!["manager", "cashier", "accounting"].includes(role)) {
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
              "Solo puedes administrar cajeros y contabilidad.",
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
          const endDate = addDays(startDate, plan.durationDays - 1);
          const membership: StoredMembership = {
            id: uuid(),
            memberId: member.id,
            planId: plan.id,
            planName: plan.name,
            status: startDate > dateOnly() ? "scheduled" : "active",
            startDate,
            endDate,
            priceMinor: plan.priceMinor,
            currency: plan.currency,
            graceDays: plan.graceDays,
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
          const endDate = addDays(periodStart, plan.durationDays - 1);
          membership.planId = plan.id;
          membership.planName = plan.name;
          membership.status = membership.startDate > dateOnly() ? "scheduled" : "active";
          membership.endDate = endDate;
          membership.priceMinor = plan.priceMinor;
          membership.currency = plan.currency;
          membership.graceDays = plan.graceDays;
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
          const payment: StoredMembershipPayment = {
            id: uuid(),
            chargeId: charge.id,
            membershipId: charge.membershipId,
            memberId: charge.memberId,
            amountMinor,
            paymentMethod: paymentMethod as StoredMembershipPayment["paymentMethod"],
            reference,
            receivedBy: session.userId,
            receivedByName: session.name,
            createdAt: now(),
          };
          charge.paidMinor += amountMinor;
          charge.status = charge.paidMinor >= charge.amountMinor ? "paid" : "pending";
          charge.updatedAt = now();
          state.membershipPayments.push(payment);
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

          const allowed = credentialId === "CRED-ALLOW";
          const reason = allowed
            ? "credencial simulada activa"
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
            schemaVersion: 17,
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
