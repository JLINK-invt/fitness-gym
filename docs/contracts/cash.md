# Contrato local · caja y turnos (Phase 3)

- **Versión de contrato:** 2.13.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v15 (`0015_payment_charges.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `current_cash_session` | — | `CashSession \| null` | `OperatePos` |
| `open_cash_session` | `openingAmountMinor` | `CashSession` | `OperatePos` |
| `close_cash_session` | `sessionId`, `countedAmountMinor`, `differenceReason?` | `CashSession` | `OperatePos` |
| `list_cash_sessions` | — (máximo 50 recientes) | `CashSession[]` | `ViewFinance` |
| `list_cash_movements` | `sessionId` (máximo 100 recientes) | `CashMovement[]` | `OperatePos` o `ViewFinance` |
| `record_cash_movement` | `sessionId`, `movementType`, `category`, `amountMinor`, `paymentMethod`, `evidence` | `CashMovement` | `OperatePos` |

## DTOs

```ts
type CashSessionStatus = "open" | "closed";
type CashMovementType = "income" | "expense";

interface CashSession {
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

interface CashMovement {
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

interface OpenCashSessionInput {
  openingAmountMinor: number;
}

interface CloseCashSessionInput {
  sessionId: string;
  countedAmountMinor: number;
  differenceReason?: string | null;
}

interface RecordCashMovementInput {
  sessionId: string;
  movementType: CashMovementType;
  category: string;
  amountMinor: number;
  paymentMethod: PaymentMethod;
  evidence: string;
}
```

## Reglas

- Solo puede existir un turno `open` por instalación; un índice único parcial lo
  impone y la API devuelve `invalidRequest` si ya hay uno.
- Toda venta y todo movimiento exigen un turno abierto. La venta guarda
  `cashSessionId`; las ventas históricas anteriores al esquema v9 conservan
  `null`. Los movimientos existen desde el esquema v10.
- `openingAmountMinor` y `countedAmountMinor` son enteros entre 0 y
  1 000 000 000 centavos.
- Los movimientos aceptan montos entre 1 y 1 000 000 000 centavos, categoría de
  1 a 80 caracteres y evidencia de 1 a 200 caracteres.
- `movementType` distingue ingreso de gasto. `paymentMethod` puede ser efectivo,
  transferencia o Point manual.
- `cashIncomeMinor` y `cashExpenseMinor` suman solo movimientos en efectivo;
  `incomeMinor` y `expenseMinor` suman todos los métodos.
- `cashRefundsMinor` suma las devoluciones cuyo método de reembolso es efectivo;
  `refundedMinor` suma todas las devoluciones del turno y `refundsCount` las
  cuenta, sin importar el método.
- `expectedCashMinor = openingAmountMinor + Σ ventas en efectivo +
  Σ ingresos en efectivo − Σ gastos en efectivo − Σ devoluciones en efectivo`.
- Devoluciones por transferencia o Point no alteran el efectivo esperado.
- Transferencia y Point cuentan para totales y `movementsCount`, pero no alteran
  el efectivo esperado.
- El cierre recibe el `sessionId` abierto para evitar cerrar un turno distinto
  por una vista obsoleta. Un turno cerrado no se puede reabrir ni cerrar dos
  veces.
- `differenceMinor = countedAmountMinor − expectedCashMinor`; puede ser negativo
  y queda registrado tal cual.
- Si `differenceMinor` no es cero, `differenceReason` es obligatorio (máximo 300
  caracteres). Si la diferencia es cero, el motivo se guarda como `null`.
- La identidad del turno y los movimientos son inmutables. `UPDATE` y `DELETE`
  están bloqueados por trigger; no se permite borrar movimientos ni corregirlos
  en sitio.
- La moneda proviene de `organizations.currency`.
- El historial y los movimientos requieren `ViewFinance` o `OperatePos`; el
  cajero puede operar el turno actual, pero no listar los turnos históricos del
  gimnasio. Contabilidad puede consultar historial y movimientos en modo lectura.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Montos fuera de rango; turno ya abierto; turno ya cerrado; movimiento sin turno abierto; diferencia sin motivo; categoría o evidencia inválidas |
| `notFound` | El turno o movimiento no existe en el gimnasio |
| `unauthorized` | Sesión ausente o permiso insuficiente |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

- `cash.opened` con recurso `cash_session` y `correlationId` igual al id del
  turno al abrir.
- `cash.movement_recorded` con recurso `cash_movement` y `correlationId` igual
  al id del movimiento.
- `cash.closed` con recurso `cash_session` y `correlationId` igual al id del
  turno al cerrar.
- `sale.completed` conserva el id de la venta; la venta enlaza el turno.
- `sale.returned` conserva el id de la devolución; la devolución enlaza el turno
  y descuenta el efectivo del turno cuando corresponde.

## Fuera de alcance

Cierres parciales, aprobación de dos personas para faltantes/sobrantes,
múltiples cajas simultáneas, arqueo por denominaciones y conciliación bancaria
automática. Las devoluciones se documentan en [`sales.md`](sales.md).
