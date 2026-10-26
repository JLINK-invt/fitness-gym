# Contrato local · socios, membresías y cobros (Fase 4)

- **Versión de contrato:** 2.12.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v17 (`0017_members.sql`)

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

interface Membership {
  id: string;
  memberId: string;
  memberName: string;
  planId: string;
  planName: string;
  status: MembershipStatus;
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
  receivedBy: string | null;
  receivedByName: string | null;
  createdAt: string;
}
```

## Reglas

- Solo puede existir una membresía `scheduled`, `active` o `paused` por socio.
- Crear o renovar una membresía con precio mayor a cero genera un cargo pendiente.
- La renovación continúa después del `endDate` vigente o desde hoy si ya venció.
- Pausar exige una fecha futura; reanudar extiende el `endDate` por los días pausados.
- Los pagos aceptan abonos parciales y no pueden superar el saldo del cargo.
- `record_membership_payment` es idempotente por `idempotencyKey`; reutilizarla con
  otro contenido devuelve conflicto.
- `membership_events`, `membership_payments` y los términos de los cargos son
  append-only.
- Cada alta, renovación, pausa, reanudación, cancelación y pago se registra en
  auditoría y en `sync_outbox`.
