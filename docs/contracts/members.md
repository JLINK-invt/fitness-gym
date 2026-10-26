# Contrato local · socios, membresías y cobros (Fase 4)

- **Versión de contrato:** 2.14.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v19 (`0019_access_attendance_reports.sql`)

## Permisos

Todas las operaciones usan `ManageMembers`. En V1 lo tienen dueño, encargado y
cajero. Contabilidad no lo tiene.

## Comandos

| Comando | Entrada | Salida |
| --- | --- | --- |
| `membership_dashboard` | — | `MembershipDashboard` |
| `list_membership_plans` | — | `MembershipPlan[]` |
| `create_membership_plan` | `CreateMembershipPlanInput` | `MembershipPlan` |
| `update_membership_plan` | `UpdateMembershipPlanInput` | `MembershipPlan` |
| `list_members` | `query?` | `Member[]` |
| `get_member` | `memberId` | `Member` |
| `create_member` | `CreateMemberInput` | `Member` |
| `update_member` | `UpdateMemberInput` | `Member` |
| `list_prospects` | `query?` | `Prospect[]` |
| `create_prospect` | `CreateProspectInput` | `Prospect` |
| `update_prospect` | `UpdateProspectInput` | `Prospect` |
| `convert_prospect` | `prospectId` | `Member` |
| `list_memberships` | `memberId` | `Membership[]` |
| `create_membership` | `CreateMembershipInput` | `Membership` |
| `renew_membership` | `RenewMembershipInput` | `Membership` |
| `pause_membership` | `PauseMembershipInput` | `Membership` |
| `resume_membership` | `ResumeMembershipInput` | `Membership` |
| `cancel_membership` | `CancelMembershipInput` | `Membership` |
| `member_account` | `memberId` | `MemberAccount` |
| `record_membership_payment` | `RecordMembershipPaymentInput` | `MembershipPayment` |
| `membership_payment_receipt` | `paymentId` | `MembershipPaymentReceipt` |
| `list_member_credentials` | `memberId` | `MemberCredential[]` |
| `issue_member_credential` | `memberId` | `MemberCredential` |
| `revoke_member_credential` | `credentialId` | `MemberCredential` |

## DTOs

```ts
type MemberStatus = "active" | "inactive" | "archived";
type ProspectStatus = "new" | "contacted" | "trial" | "converted" | "lost";
type MembershipStatus = "scheduled" | "active" | "paused" | "expired" | "cancelled";
type MembershipChargeStatus = "pending" | "paid" | "void";

interface MembershipPlan {
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

interface Member {
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

interface MemberCredential {
  id: string;
  memberId: string;
  memberNumber: string;
  memberName: string;
  code: string;
  status: "active" | "revoked";
  revision: number;
  issuedAt: string;
  revokedAt: string | null;
}

interface Membership {
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

interface MembershipCharge {
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

interface MembershipPayment {
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
}

interface MembershipPaymentReceipt {
  paymentId: string;
  receiptNumber: string;
  gymName: string;
  memberNumber: string;
  memberName: string;
  chargeDescription: string;
  amountMinor: number;
  currency: string;
  paymentMethod: "cash" | "transfer" | "point";
  reference: string | null;
  receivedByName: string | null;
  createdAt: string;
}

interface MembershipDashboardTrendPoint {
  month: string;
  newMembers: number;
  renewals: number;
  cancellations: number;
  incomeMinor: number;
}

interface MembershipDashboard {
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
```

## Reglas

- Los planes son versionados: editar un plan crea una versión inmutable y no
  cambia los términos de membresías existentes. Renovar toma la versión vigente.
- `accessDays` usa `mon,tue,wed,thu,fri,sat,sun`; `accessStartMinute` va de 0 a
  1439 y `accessEndMinute` de 1 a 1440.
- `minAge`/`maxAge` son opcionales y se validan al crear o renovar contra la
  fecha de nacimiento del socio; sin fecha de nacimiento el alta falla.
- Solo puede existir una membresía `scheduled`, `active` o `paused` por socio.
- Crear o renovar una membresía con precio mayor a cero genera un cargo pendiente.
- La renovación continúa después del `endDate` vigente o desde hoy si ya venció.
- Pausar exige una fecha futura; reanudar extiende el `endDate` por los días pausados.
- Los pagos aceptan abonos parciales y no pueden superar el saldo del cargo.
- Los pagos en efectivo requieren una caja abierta y generan un movimiento de
  caja con categoría `membership_payment` y evidencia con el número de recibo.
- Cada pago recibe un recibo no fiscal `REC-00001`; `membership_payment_receipt`
  devuelve los datos para mostrarlo o imprimirlo.
- `record_membership_payment` es idempotente por `idempotencyKey`; reutilizarla con
  otro contenido devuelve conflicto.
- `membership_events`, `membership_payments` y los términos de los cargos son
  append-only.
- Cada alta, renovación, pausa, reanudación, cancelación y pago se registra en
  auditoría y en `sync_outbox`.
