/**
 * TypeScript mirror of `crates/gym-contracts`.
 * Keep both sides in sync; `CONTRACT_VERSION` guards breaking changes.
 */

export const CONTRACT_VERSION = "1.5.0";

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

export type StaffRole = "owner" | "manager" | "cashier" | "accounting";

export type Permission =
  | "manageSettings"
  | "manageStaff"
  | "viewFinance"
  | "operatePos"
  | "manageInventory"
  | "manageCatalog"
  | "manageBackups"
  | "viewAudit";

export interface SetupState {
  configured: boolean;
  gymName?: string;
  ownerReady: boolean;
}

export interface SessionSummary {
  userId: string;
  organizationId: string;
  siteId?: string;
  name: string;
  role: StaffRole;
  permissions: Permission[];
}

export type SiteStatus = "active" | "closed";

export type StaffStatus = "active" | "revoked";

export interface SiteSummary {
  id: string;
  name: string;
  status: SiteStatus;
  createdAt: string;
}

export interface StaffSummary {
  userId: string;
  name: string;
  role: StaffRole;
  siteId: string | null;
  siteName: string | null;
  status: StaffStatus;
  lastLoginAt: string | null;
  createdAt: string;
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

export type AuditResult = "allowed" | "denied" | "failed";

export interface AuditEventSummary {
  id: string;
  sequence: number;
  actorId: string | null;
  actorName: string | null;
  siteId: string | null;
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
