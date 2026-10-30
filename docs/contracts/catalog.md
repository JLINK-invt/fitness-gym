# Contrato local · catálogo, precios e impuestos (Phase 3)

- **Versión de contrato:** 3.0.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v21 (`0021_gym_operations.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_catalog` | — | `CatalogProduct[]` | `OperatePos` |
| `create_catalog_item` | `name`, `category`, `variant` | `CatalogProduct` | `ManageCatalog` |
| `add_catalog_variant` | `productId`, `variant` | `CatalogProduct` | `ManageCatalog` |
| `update_catalog_product` | `productId`, `name`, `category`, `status` | `CatalogProduct` | `ManageCatalog` |
| `update_catalog_variant` | `variantId`, `name`, `sku`, `barcode`, `unit`, `costMinor`, `status` | `CatalogProduct` | `ManageCatalog` |
| `add_catalog_price` | `variantId`, `price` | `CatalogProduct` | `ManageCatalog` |

## DTOs

```ts
type CatalogStatus = "active" | "archived";
type CatalogUnit = "piece" | "service" | "session" | "package";

interface CatalogPrice {
  id: string;
  currency: string;
  amountMinor: number;
  taxRateBps: number;
  priceIncludesTax: boolean;
  effectiveFrom: string;
  createdAt: string;
}

interface CatalogVariant {
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

interface CatalogProduct {
  id: string;
  name: string;
  category: string | null;
  status: CatalogStatus;
  variants: CatalogVariant[];
  createdAt: string;
  updatedAt: string;
}
```

## Reglas

- Producto: nombre único por gimnasio sin distinguir mayúsculas; 2–80
  caracteres visibles tras `trim`. Categoría opcional de 2–60.
- Variante: nombre de 1–80 caracteres y SKU de 1–40 obligatorios; código de
  barras opcional de 1–64. SKU y código de barras son únicos por gimnasio. La
  unidad y el costo no pueden ser negativos.
- Precio: `amountMinor >= 0`, impuesto entre 0 y 10 000 bps (0–100%).
  `effective_from` se normaliza a RFC3339 UTC y, si falta, usa la hora actual.
- Un producto o variante archivado no acepta nuevas variantes ni precios.
- Archivar un producto no archiva en cascada sus variantes.
- Editar un precio no actualiza la fila anterior: inserta una versión nueva.
- La versión vigente es la de mayor `effective_from <= now`; si no existe,
  `currentPrice` es `null`.
- La moneda se toma de `organizations.currency`; la UI no la envía.
- Las ventas guardan `price_version_id` y los importes aplicados; ver
  [`sales.md`](sales.md).

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Nombre, SKU, unidad, costo, precio o impuesto inválido; duplicado; recurso archivado |
| `unauthorized` | Sesión ausente o permiso insuficiente |
| `notFound` | Producto o variante inexistente en el gimnasio |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

Se registran con el actor de la sesión:

- `catalog.product_created`, `catalog.product_updated`.
- `catalog.variant_created`, `catalog.variant_updated`.
- `catalog.price_added`.

No se guardan secretos ni datos personales en estos eventos.

## Fuera de alcance

Inventario, combos, descuentos, múltiples listas de precios, impuestos por
producto con múltiples tasas y compras. Las ventas consumen este contrato;
las devoluciones quedan para un slice posterior.
