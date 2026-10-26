# Contrato local · inventario y valoración (Phase 3)

- **Versión de contrato:** 2.12.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v15 (`0015_payment_charges.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_inventory` | — | `InventoryItem[]` | `ManageInventory` |
| `list_inventory_movements` | `variantId?`, `limit?` | `InventoryMovement[]` | `ManageInventory` |
| `record_inventory_entry` | `variantId`, `quantity`, `unitCostMinor`, `reference?`, `note?` | `InventoryMovement` | `ManageInventory` |
| `adjust_inventory` | `variantId`, `quantityDelta`, `unitCostMinor?`, `reason` | `InventoryMovement` | `ManageInventory` |
| `update_inventory_threshold` | `variantId`, `minStock` | `InventoryItem` | `ManageInventory` |

`limit` se acota entre 1 y 200; la UI usa 40 por defecto.

## DTOs

```ts
interface InventoryItem {
  variantId: string;
  productName: string;
  variantName: string;
  sku: string;
  unit: string;
  stock: number;
  valueMinor: number;
  avgUnitCostMinor: number;
  minStock: number;
  lowStock: boolean;
}

type InventoryMovementType = "entry" | "sale" | "return" | "adjustment";
type InventorySourceType = "manual" | "sale" | "sale_return" | "adjustment";

interface InventoryMovement {
  id: string;
  variantId: string;
  movementType: InventoryMovementType;
  quantityDelta: number;
  unitCostMinor: number;
  totalCostMinor: number;
  sourceType: InventorySourceType;
  sourceId: string | null;
  reference: string | null;
  note: string | null;
  authorizedBy: string | null;
  authorizedByName: string | null;
  createdAt: string;
}

interface RecordInventoryEntryInput {
  variantId: string;
  quantity: number;
  unitCostMinor: number;
  reference?: string;
  note?: string;
}

interface AdjustInventoryInput {
  variantId: string;
  quantityDelta: number;
  unitCostMinor?: number;
  reason: string;
}

interface UpdateInventoryThresholdInput {
  variantId: string;
  minStock: number;
}
```

## Reglas

- Solo las variantes con unidad `piece` participan en inventario. Las variantes
  de servicio se rechazan con `invalidRequest`.
- El stock nunca se edita: la única fuente es `inventory_movements`. Entradas,
  ventas, devoluciones y ajustes agregan filas nuevas.
- `quantityDelta` es distinto de cero y su valor absoluto está entre 1 y
  999 999. `unitCostMinor` está entre 0 y 1 000 000 000 centavos.
- `totalCostMinor = quantityDelta × unitCostMinor`; en una salida ambos valores
  son negativos para conservar el valor económico del movimiento.
- La instalación opera una sola ubicación lógica de inventario. El modelo
  multi-ubicación y las transferencias se eliminaron en la migración 13.
- La entrada suma cantidad al costo indicado. `reference` admite hasta 120
  caracteres y `note` hasta 300; ambos se recortan y los vacíos se guardan como
  `null`.
- El ajuste usa `sourceType = "adjustment"`, exige motivo de 1 a 300 caracteres
  y calcula el costo con `unitCostMinor` o, si se omite, con el costo promedio
  global de la variante.
- El umbral `minStock` está entre 0 y 999 999. `lowStock` es
  `stock <= minStock`; por defecto el mínimo es 0, así que un SKU sin existencias
  figura bajo mínimo.
- `list_inventory` devuelve todas las variantes pieza activas o archivadas del
  gimnasio, con stock y valor derivados de los movimientos.
- `avgUnitCostMinor` es `valueMinor / stock` redondeado cuando hay existencia y
  `0` en caso contrario. El valor puede ser negativo si las ventas superan las
  entradas.
- Las ventas de variantes pieza agregan un movimiento `sale` negativo y las
  devoluciones agregan un movimiento `return` positivo al costo del movimiento
  de venta original. Vender sin stock está permitido.
- `inventory_movements` es append-only por trigger: `UPDATE` y `DELETE` fallan.
- La moneda proviene de `organizations.currency`.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Variante de servicio; cantidad, costo, umbral o motivo fuera de rango; cambio de umbral inválido |
| `notFound` | Variante o movimiento inexistente en el gimnasio |
| `unauthorized` | Sesión ausente o permiso insuficiente |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

- `inventory.entry` con recurso `inventory_movement`.
- `inventory.adjusted` con recurso `inventory_movement`.
- `inventory.threshold.updated` con recurso `inventory_policy`.

Cada evento incluye al actor de la sesión, resultado `allowed`, origen `tauri` y
un `correlationId` con el id del movimiento o política afectada.

## Fuera de alcance

Compras a proveedores, costeo FIFO/LIFO, números de lote, caducidad, conteos
físicos programados y multi-sede. La valoración usa costo promedio ponderado
global por variante.
