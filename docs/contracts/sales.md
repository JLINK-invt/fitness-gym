# Contrato local · ventas y cobro (Phase 3)

- **Versión de contrato:** 2.12.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v15 (`0015_payment_charges.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_sales` | — (máximo 50 recientes) | `Sale[]` | `OperatePos` |
| `create_sale` | `items`, `paymentMethod`, `cashReceivedMinor?`, `paymentReference?`, `idempotencyKey` | `Sale` | `OperatePos` |
| `list_sale_returns` | `saleId?` (máximo 50 recientes) | `SaleReturn[]` | `OperatePos` |
| `create_sale_return` | `saleId`, `items`, `reason`, `paymentReference?`, `idempotencyKey` | `SaleReturn` | `OperatePos` |
| `list_payment_charges` | `saleId?`, `externalReference?`, `limit?` | `PaymentCharge[]` | `ViewFinance` |

## DTOs

```ts
type PaymentMethod = "cash" | "transfer" | "point";

interface SaleItemInput {
  variantId: string;
  quantity: number;
}

interface CreateSaleInput {
  items: SaleItemInput[];
  paymentMethod: PaymentMethod;
  cashReceivedMinor?: number;
  paymentReference?: string;
  idempotencyKey: string;
}

interface SaleItem {
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

interface SaleReturnItemInput {
  saleItemId: string;
  quantity: number;
}

interface CreateSaleReturnInput {
  saleId: string;
  items: SaleReturnItemInput[];
  reason: string;
  paymentReference?: string;
  idempotencyKey: string;
}

interface SaleReturnItem {
  id: string;
  saleItemId: string;
  productName: string;
  variantName: string;
  sku: string;
  quantity: number;
  amountMinor: number;
}

interface SaleReturn {
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

interface PaymentCharge {
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

interface Sale {
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
```

## Reglas

- Toda venta exige un turno de caja abierto y queda enlazada con
  `cashSessionId`. Las ventas históricas anteriores al esquema v9 conservan
  `null`.
- Una venta tiene entre 1 y 50 líneas; cada variante aparece una sola vez y la
  cantidad está entre 1 y 999.
- Solo se venden productos y variantes con estado `active`.
- El precio se resuelve en Rust al confirmar la venta con la versión vigente
  (`effective_from <= now`). La UI nunca envía importes.
- La línea guarda `price_version_id` y un snapshot de nombre de producto,
  variante, SKU, unidad, monto unitario, tasa e indicador de impuesto incluido.
  Editar el catálogo después no altera la venta.
- Si el precio incluye impuesto, `subtotal = round(total × 10000 / (10000 + bps))`
  y `tax = total − subtotal`. Si no lo incluye,
  `tax = round(subtotal × bps / 10000)` y `total = subtotal + tax`. El redondeo
  es half-up a centavos.
- `cash`: `cashReceivedMinor` es opcional y por defecto igual al total; debe
  cubrir el total y `changeMinor = recibido − total`.
- `transfer` y `point`: `cashReceivedMinor` debe omitirse; ambos quedan `null`.
- `paymentReference` es opcional, de hasta 120 caracteres, para el folio de la
  terminal o la clave bancaria. Se guarda en la venta y en `payment_charges`
  para conciliación; no se admiten datos de tarjeta.
- La moneda proviene de `organizations.currency`.
- `idempotencyKey` es obligatoria, de 1 a 128 caracteres y se genera por
  intento de venta. Reintentar con la misma clave y el mismo contenido devuelve
  la venta original sin cobrar ni descontar stock dos veces; reutilizarla con
  otro contenido responde conflicto de idempotencia.
- Los totales no pueden superar 1 000 000 000 centavos.
- Cada línea de una variante `piece` registra un movimiento de inventario
  negativo, valuado al costo promedio vigente.
  Vender sin stock está permitido y deja la existencia en negativo.
- `sales` y `sale_items` son append-only por trigger: no admiten `UPDATE` ni
  `DELETE`. Las devoluciones se registran como documentos compensatorios.
- Cada venta crea un cargo aprobado en `payment_charges` con proveedor
  `manual` (efectivo/transferencia) o `point`. Cada devolución crea un
  reembolso aprobado; `listPaymentCharges` permite filtrar por venta o
  referencia externa con permiso `ViewFinance`.

## Devoluciones

- Una devolución referencia una venta confirmada y uno o más `saleItemId`; no
  edita ni borra la venta original.
- `idempotencyKey` es obligatoria (1–128 caracteres). Reintentar con la misma
  clave y contenido devuelve la devolución original; reutilizarla con otro
  contenido responde conflicto de idempotencia.
- `paymentReference` es opcional (máximo 120 caracteres) y se guarda en el
  reembolso para conciliación.
- La cantidad se valida contra lo disponible por línea:
  `cantidad − refundedQuantity`. Una línea agotada ya no admite devoluciones.
- `quantity` está entre 1 y 999 por línea; una venta admite hasta 50 líneas por
  devolución y un mismo `saleItemId` aparece una sola vez.
- El importe se calcula en Rust desde el snapshot de la venta:
  `floor(remainingAmount × quantity / remainingQuantity)`. Si se devuelve todo
  lo restante de la línea, se usa el remanente exacto para no perder centavos.
- El método de reembolso se deriva de la venta original (`refundMethod`); la UI
  no puede enviar uno distinto.
- Toda devolución exige un turno de caja abierto y guarda `cashSessionId`.
  `reason` es obligatorio, de 1 a 300 caracteres.
- `totalMinor` debe ser mayor que cero y no superar 1 000 000 000 centavos.
- `list_sale_returns` filtra opcionalmente por `saleId` y devuelve las más
  recientes primero.
- Cada devolución de una variante `piece` repone la cantidad y usa el costo
  promedio del movimiento de venta original.

## Ledger inmutable

- `ledger_entries` registra cada evento económico como asiento nuevo:
  venta positiva (`sale`), devolución negativa (`sale_return`).
- Cada asiento referencia `source_id` (venta o devolución), guarda moneda,
  importe, tipo y fecha; no se actualiza ni se borra.
- Triggers bloquean `UPDATE` y `DELETE` sobre `ledger_entries`.
- Los totales se reconstruyen con `SUM(amount_minor)` por organización y deben
  coincidir con ventas menos devoluciones. Las ventas previas al esquema v11 se
  migraron al ledger durante la migración.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Carrito vacío o demasiado grande; cantidad, pago o efectivo inválido; variante repetida, archivada o sin precio; total excesivo |
| `unauthorized` | Sesión ausente o permiso insuficiente |
| `notFound` | Variante, venta o devolución inexistente en el gimnasio |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

Cada venta registra `sale.completed` con recurso `sale` y `correlationId` igual
al id de la venta. Cada devolución registra `sale.returned` con recurso
`sale_return` y `correlationId` igual al id de la devolución. Ambos eventos
incluyen al actor de la sesión, resultado `allowed` y origen `tauri`. No se
guardan secretos ni datos personales adicionales.

## Fuera de alcance

Cancelaciones sin devolución, descuentos, propinas, pagos mixtos, facturación
(CFDI) y compras a proveedores. El arqueo de caja y los turnos se documentan en
[`cash.md`](cash.md). El ticket es no fiscal y se imprime bajo demanda.
