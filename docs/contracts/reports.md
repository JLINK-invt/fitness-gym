# Contrato local · reportes y analítica (Phase 3–4)

- **Versión de contrato:** 2.13.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v15 (`0015_payment_charges.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `financial_report` | `from`, `to` (RFC3339) | `FinancialReport` | `ViewFinance` |
| `export_financial_report_csv` | `from`, `to` (RFC3339) | `FinancialReportExport` | `ViewFinance` |

## DTOs

```ts
interface FinancialReportInput {
  from: string;
  to: string;
}

interface PaymentMethodTotal {
  paymentMethod: PaymentMethod;
  salesCount: number;
  grossSalesMinor: number;
  refundsCount: number;
  refundedMinor: number;
  netSalesMinor: number;
}

interface ProductSalesTotal {
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

interface InventoryAdjustmentTotal {
  variantId: string;
  productName: string;
  variantName: string;
  sku: string;
  unitsDelta: number;
  costMinor: number;
}

interface FinancialReport {
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

interface FinancialReportExport {
  fileName: string;
  path: string;
  bytes: number;
}
```

## Reglas

- `from` y `to` deben ser fechas RFC3339 válidas, con `from < to` y un máximo
  de 366 días. Los límites se comparan con `julianday()` para tolerar offsets.
- Las ventas, devoluciones, ajustes y turnos se filtran por `created_at` u
  `opened_at` dentro del rango; `to` es exclusivo.
- `grossSalesMinor` y `taxMinor` suman el ledger de ventas; `refundedMinor`
  suma el ledger de devoluciones. `netSalesMinor = grossSalesMinor -
  refundedMinor`.
- `averageSaleMinor` es el promedio entero de la venta bruta entre las ventas
  del rango, o cero si no hay ventas.
- `attachRateBps` es el porcentaje (en puntos base) de ventas con dos o más
  líneas distintas; `itemsPerSaleMilli` es el promedio de unidades por venta
  expresado en milésimas.
- `costOfGoodsMinor` suma el costo de las ventas por producto;
  `marginMinor = netSalesMinor - costOfGoodsMinor` y `marginBps` expresa el
  margen sobre la venta neta.
- `inventoryValueStartMinor` y `inventoryValueEndMinor` son el valor acumulado
  de `inventory_movements.total_cost_minor` antes de `from` y de `to`.
  `inventoryTurnoverBps = costOfGoodsMinor / promedio de inventario`, en puntos
  base; si el promedio no es positivo, queda en cero.
- `adjustmentsUnits` y `adjustmentsCostMinor` son la suma firmada de los ajustes
  del rango; se desglosan en `inventoryAdjustments`.
- `productSales` se ordena por venta neta y unidades descendentes; el primer
  elemento es el top de productos.
- `paymentMethods` siempre incluye `cash`, `transfer` y `point`, aun en cero.
  Cada método separa ventas y devoluciones, y su neto reconcilia con el total.
- Las ventas por producto agrupan por variante con snapshot de nombre, SKU,
  unidad y precios. `marginMinor = netSalesMinor - costMinor`.
- `costMinor` usa el costo de `inventory_movements` de tipo `sale` y `return`
  del rango; los servicios y variantes sin movimientos quedan en costo cero.
- Los ajustes agrupan `inventory_movements` de tipo `adjustment` por variante,
  con unidades y costo firmados.
- Los turnos se cuentan por `opened_at`; `cashDifferenceMinor` y
  `cashDifferenceAbsMinor` suman solo turnos cerrados.
- La exportación escribe un CSV UTF-8 con saltos CRLF en
  `<app_data>/exports/reporte-<desde>-<hasta>-<marca>.csv`. Los campos de texto
  se escapan y se neutralizan fórmulas de hoja de cálculo.
- La moneda proviene de `organizations.currency`.
- Solo `ViewFinance` (dueño, encargado y contabilidad) puede consultar o
  exportar. El cajero no accede a reportes.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Rango inválido, invertido o mayor a 366 días |
| `unauthorized` | Sesión ausente o permiso insuficiente |
| `storage` | Fallo de SQLCipher, escritura del CSV o auditoría |

## Auditoría

- `report.exported` con recurso `financial_report`, resultado `allowed` y
  `correlationId` igual al nombre del archivo.
- Si la auditoría falla, el archivo recién escrito se elimina y el comando
  devuelve `storage`; no queda una exportación sin rastro.
- La consulta del reporte no escribe auditoría para no inflar la cadena en cada
  cambio de rango.

## Fuera de alcance

Reportes fiscales, declaraciones, conciliación bancaria automática, envío por
correo, programación de cierres y exportación a Excel/PDF.
