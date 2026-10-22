/**
 * TypeScript mirror of `crates/gym-contracts`.
 * Keep both sides in sync; `CONTRACT_VERSION` guards breaking changes.
 */

export const CONTRACT_VERSION = "1.3.0";

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
