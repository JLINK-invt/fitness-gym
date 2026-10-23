# Contrato local · ventas y cobro (Phase 3)

- **Versión de contrato:** 2.1.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v7 (`0007_sales.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_sales` | — (máximo 50 recientes) | `Sale[]` | `OperatePos` |
| `create_sale` | `items`, `paymentMethod`, `cashReceivedMinor?` | `Sale` | `OperatePos` |

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
}

interface Sale {
  id: string;
  currency: string;
  subtotalMinor: number;
  taxMinor: number;
  totalMinor: number;
  paymentMethod: PaymentMethod;
  cashReceivedMinor: number | null;
  changeMinor: number | null;
  soldBy: string | null;
  soldByName: string | null;
  items: SaleItem[];
  createdAt: string;
}
```

## Reglas

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
- La moneda proviene de `organizations.currency`.
- Los totales no pueden superar 1 000 000 000 centavos.
- `sales` y `sale_items` son append-only por trigger: no admiten `UPDATE` ni
  `DELETE`. Una devolución futura requerirá un documento compensatorio, no la
  edición de la venta.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Carrito vacío o demasiado grande; cantidad, pago o efectivo inválido; variante repetida, archivada o sin precio; total excesivo |
| `unauthorized` | Sesión ausente o permiso insuficiente |
| `notFound` | Variante inexistente en el gimnasio |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

Cada venta registra `sale.completed` con recurso `sale`, resultado `allowed`,
origen `tauri` y `correlationId` igual al id de la venta. El evento incluye al
actor de la sesión. No se guardan secretos ni datos personales adicionales.

## Fuera de alcance

Devoluciones, cancelaciones, descuentos, propinas, pagos mixtos, facturación
(CFDI), arqueo de caja y turnos. El ticket es no fiscal y se imprime bajo
demanda.
