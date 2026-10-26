# Arquitectura de la fundación

## Flujo

```text
React (apps/gym-desktop/src)
  │ invoke() tipado con @gym/contracts
  ▼
Comandos Tauri (apps/gym-desktop/src-tauri)
  │
  ├── gym-core      reglas puras, RBAC y bus de eventos
  ├── gym-data      SQLCipher + migraciones + key provider + journal de sync
  ├── gym-sync      motor outbox/inbox y frontera SyncTransport
  └── gym-backup    snapshots cifrados + restore verificado
```

La nube no participa en la operación. Todo el flujo anterior es local; el
journal de sync solo se ejercita contra un servidor falso en pruebas.

## Límites de dependencia

- `gym-core` no depende de Tauri, SQLite ni del sistema operativo.
- `gym-contracts` solo contiene DTOs serializables y errores de API.
- `gym-data` depende de `gym-core` y `gym-contracts`; nunca al revés.
- `gym-backup` depende de `gym-data`; no expone detalles de SQLCipher a la UI.
- `gym-sync` depende de `gym-data`; no conoce Tauri ni detalles de SQLCipher y
  recibe el transporte por un trait.
- La app Tauri es la única capa que conoce comandos, ventanas y rutas del SO.
- `packages/contracts` es el espejo TypeScript de `gym-contracts`; un cambio
  incompatible exige subir `CONTRACT_VERSION`.

## Persistencia

- Cada base local es un archivo SQLCipher con WAL.
- La clave se inyecta con `PRAGMA key` antes de cualquier lectura.
- `schema_migrations` guarda versión, nombre, checksum y fecha.
- Cada instalación tiene una sola organización (`organization_id NOT NULL`); el
  esquema no modela sedes.
- `staff_memberships` impone una membresía por usuario e índice único por
  gimnasio y usuario.
- `module_events` impone `UNIQUE (organization_id, idempotency_key)`.
- `audit_events` es append-only por trigger, con `sequence` por gimnasio y hash
  chain SHA-256 (`prev_hash` + `event_hash`). La migración 4 conserva los eventos
  heredados sin hash y la cadena verificable empieza después.
- `catalog_products` conserva productos y variantes con archivo lógico;
  `catalog_price_versions` es append-only.
- `sales` y `sale_items` son append-only y guardan snapshot de nombres, SKU,
  unidad y precio; cada línea referencia la versión de precio cobrada. La
  migración 14 agrega `idempotency_key` y `request_hash` por venta para que un
  reintento devuelva la operación original sin duplicar cobro ni stock.
- `cash_sessions` guarda cada turno con fondo inicial, conteo, diferencia,
  motivo y usuarios de apertura/cierre. Un índice único parcial permite un solo
  turno abierto y las ventas se enlazan con `cash_session_id`.
- `cash_movements` guarda ingresos y gastos con método, categoría, evidencia y
  autorizante. Es append-only y el cierre recalcula el efectivo esperado con los
  movimientos en efectivo.
- `sale_returns` y `sale_return_items` son documentos compensatorios
  append-only; referencian la venta y el turno donde sale el dinero. La
  migración 15 agrega idempotencia por devolución para que un reintento no
  reembolse dos veces.
- `payment_charges` registra cargos y reembolsos aprobados con método,
  proveedor (`manual` o `point`), importe, referencia externa e idempotencia.
  Es append-only y nunca almacena PAN, CVV ni datos de tarjeta.
- `ledger_entries` es el journal inmutable: ventas positivas y devoluciones
  negativas, sin `UPDATE` ni `DELETE`. Los reportes reconstruyen totales desde
  él y la migración 11 hizo backfill de las ventas previas.
- `inventory_movements` es el único origen del stock: entradas, ventas,
  devoluciones y ajustes son filas append-only. La migración 13 elimina el
  modelo multi-ubicación y la vista `inventory_balances` agrega cantidad y
  valor por variante en la única ubicación lógica.
- `inventory_policies` guarda el mínimo por variante; el estado `lowStock` se
  calcula como `stock <= minStock` al consultar.
- El acceso simulado usa `access_decision` como agregado y
  `access.allowed`/`access.denied` como tipos de evento.
- `sync_outbox` guarda eventos de ventas, devoluciones, caja e inventario en la
  misma transacción del cambio de negocio, con `idempotency_key` única por
  gimnasio. `sync_inbox` deduplica por `event_id` y `sync_state` guarda el
  último `server_sequence`. La migración 16 crea el journal; no hay transporte
  de red ni aplicación de eventos remotos al negocio.

## Identidad y sesión

- `users` guarda nombre, hash Argon2id, hash del código de recuperación y estado;
  `staff_memberships` guarda el rol. Solo puede existir un `owner` activo por
  instalación.
- El primer arranque crea gimnasio y dueño en una sola operación; no hay usuario
  ni contraseña por defecto.
- La contraseña nunca cruza al webview después del formulario; el hash y el hash
  del código de recuperación quedan en SQLCipher y la sesión vive en memoria del
  proceso Tauri. El código en claro se muestra una sola vez.
- El código de recuperación del dueño es de un solo uso: al recuperar la cuenta
  se rota de forma atómica y se abre una sesión nueva. El dueño puede generar
  otro desde **Equipo**.
- Cinco fallos bloquean el nombre o el flujo de recuperación durante cinco
  minutos; 15 minutos de inactividad cierran la sesión.
- Los comandos operativos reciben el actor desde la sesión Rust, nunca desde un
  `userId` enviado por la UI.
- El dueño administra todo el equipo. Un encargado solo ve y administra cajeros
  y contabilidad; no puede crear ni editar encargados ni al dueño.
- El dueño no se edita desde la UI: se crea solo en la configuración inicial. Un
  usuario tampoco puede modificar su propia membresía.
- Revocar una cuenta impide el siguiente login; la sesión en memoria se invalida
  al expirar o al hacer logout. Restaurar un backup cierra la sesión porque la
  base restaurada puede tener otro dueño o personal revocado.
- `auth.owner_created`, `auth.login`, `auth.logout`,
  `auth.owner_recovery_generated`, `auth.owner_recovered`, `staff.created`,
  `staff.updated`, `access.simulated`, `catalog.*`, `sale.completed`,
  `cash.opened`, `cash.closed`, `inventory.*` y `backup.created` se registran
  en `audit_events` con el actor de la sesión.

## Auditoría

Cada evento sensible se agrega en una transacción que calcula el consecutivo y
el hash del evento anterior. La tabla rechaza `UPDATE` y `DELETE`; la UI solo
consulta y puede verificar la cadena. Todos los roles con `ViewAudit` ven los
eventos de la instalación. La cadena detecta alteraciones de contenido, pero no un
truncado del final sin una copia externa; ese riesgo queda documentado.

El contrato completo está en [`contracts/audit.md`](contracts/audit.md).

## Personal

La pantalla **Equipo** requiere `ManageStaff` y permite crear accesos, cambiar
rol y desactivar o reactivar cuentas.

Reglas de autorización aplicadas en Rust:

- No se puede crear ni elevar una cuenta a `owner`.
- Un `manager` solo lista y administra `cashier` y `accounting`.
- Solo puede existir una membresía por usuario.
- Los comandos filtran por `organization_id` y validan el rol actual antes de
  modificarlo.

El contrato completo está en [`contracts/staff.md`](contracts/staff.md).

## Catálogo y precios

La pantalla **Catálogo** requiere `ManageCatalog` y permite crear productos con
una variante inicial, editar producto o variante, archivarlos y agregar precios.
Cada precio es una versión inmutable con monto, tasa de impuesto, indicador de
impuesto incluido y `effective_from`; la UI muestra la versión vigente según la
hora actual. La moneda proviene del gimnasio. Los SKU y códigos de barras son
únicos por gimnasio. El contrato completo está en
[`contracts/catalog.md`](contracts/catalog.md).

## Ventas y cobro

La pantalla **Ventas** requiere `OperatePos`. El cajero busca variantes activas
del catálogo, arma el carrito y cobra en efectivo, transferencia o terminal
Point. Rust resuelve la versión de precio vigente y calcula subtotal, impuesto y
total; la UI solo muestra el desglose. La venta guarda un snapshot inmutable y
las tablas rechazan `UPDATE` y `DELETE`. Toda venta exige un turno de caja
abierto y una clave de idempotencia por intento. Transferencias y Point aceptan
una referencia externa opcional para conciliación, guardada también en
`payment_charges`. Las devoluciones se registran como documentos nuevos por
línea, con motivo, método derivado de la venta e idempotencia propia; cada una
genera un asiento negativo en el ledger y un reembolso en `payment_charges`. El ticket es no fiscal y se imprime bajo demanda. El contrato completo
está en [`contracts/sales.md`](contracts/sales.md).

## Reportes financieros

La pantalla **Reportes** requiere `ViewFinance`. Rust reconstruye el cierre desde
el ledger inmutable: ventas, devoluciones, desglose por método de pago, ventas
por producto con margen, ajustes de inventario y diferencias de turnos. El costo
sale de `inventory_movements`; los servicios usan costo cero. La exportación
escribe un CSV en `<app_data>/exports/` y registra `report.exported` con el
nombre del archivo como correlación; si la auditoría falla, el archivo se
elimina. El contrato completo está en
[`contracts/reports.md`](contracts/reports.md).

## Caja y turnos

La pantalla **Caja** requiere `OperatePos` para abrir, consultar el turno actual
y cerrar; solo `ViewFinance` puede listar el historial. El cajero registra el
fondo inicial, registra ingresos y gastos con evidencia y la app impide cobrar
sin un turno abierto. Al cerrar, cuenta el efectivo: Rust calcula el esperado
con las ventas, movimientos y devoluciones en efectivo del turno, exige motivo
cuando hay diferencia y bloquea reaperturas. El contrato completo está en
[`contracts/cash.md`](contracts/cash.md).

## Inventario y valoración

El **Panel** resume existencias, valor, SKU bajo mínimo y rotación de los
últimos 30 días. La pantalla **Inventario** requiere `ManageInventory` y muestra
existencias, valor y costo promedio por variante pieza. El stock nunca se edita:
entradas, ajustes, ventas y devoluciones agregan movimientos inmutables. Las
ventas pueden dejar stock negativo. El modelo multi-ubicación se eliminó en la
migración 13; el contrato completo está en
[`contracts/inventory.md`](contracts/inventory.md).

## Acceso simulado

El backend conserva `simulate_access`: `gym-core` decide sin I/O y `gym-data`
inserta una sola vez en `module_events`. La misma clave con el mismo lector y
credencial devuelve el evento original; con datos distintos se rechaza. La ruta
y la navegación no se exponen en la UI de V1. El contrato completo está en
[`contracts/access-simulated.md`](contracts/access-simulated.md).

## Sincronización local

`gym-data` escribe un evento de outbox en la misma transacción que la venta,
devolución, turno o movimiento de inventario. `gym-sync` toma los pendientes
vencidos, los empuja por `SyncTransport` y marca los aceptados como `sent`; los
rechazos se reintentan con backoff y pasan a `dead_letter` tras cinco intentos.
El inbox guarda eventos remotos, deduplica por `event_id` y avanza un cursor por
secuencia de servidor. Las pruebas usan un servidor falso en memoria; el
transporte HTTPS con mTLS, el enrolamiento y la aplicación de eventos remotos
quedan pendientes. El contrato completo está en
[`contracts/sync.md`](contracts/sync.md).

## Backups

1. `VACUUM INTO` produce una copia SQLCipher consistente.
2. La copia se cifra con XChaCha20-Poly1305 usando una clave derivada de la
   passphrase con Argon2id.
3. El sobre guarda magia, salt y nonce; el sidecar JSON guarda checksum,
   tamaño, versión de esquema e id.
4. Restore verifica checksum, descifra, valida `PRAGMA integrity_check` con la
   clave de la base y reemplaza el archivo de forma atómica con rollback.

## Criterios de salida de Phase 1

- Vertical slice Tauri → Rust → SQLite → evento de acceso → backup → restore
  operando offline.
- Cero eventos duplicados tras reinicio o restauración.
- Datos locales ilegibles sin la clave.
- CI reproduce build, lint, tipos y pruebas en instalación limpia.
