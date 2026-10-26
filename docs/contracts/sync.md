# Contrato local · sincronización (Phase 3)

- **Versión de contrato:** 2.12.0
- **Estado:** journal local implementado; transporte de red pendiente
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v16 (`0016_sync_journal.sql`)

## Alcance

La app registra los cambios sincronizables en un journal local duradero. El
outbox se escribe en la misma transacción que el cambio de negocio; el inbox
guarda eventos remotos de forma idempotente y mantiene un cursor por secuencia
de servidor. El motor `gym-sync` puede empujar y jalar contra cualquier
`SyncTransport`; las pruebas usan un servidor falso en memoria.

No hay comandos Tauri, pantalla de sincronización, enrolamiento de dispositivo
ni transporte de red habilitado. La sincronización multi-dispositivo real queda
pendiente de un servidor, mTLS y resolución de conflictos.

## Tablas

| Tabla | Propósito |
| --- | --- |
| `sync_outbox` | Eventos locales pendientes, enviados o en `dead_letter` |
| `sync_inbox` | Eventos remotos recibidos, deduplicados por `event_id` |
| `sync_state` | Último `server_sequence` aplicado por gimnasio |

`sync_outbox` impone `UNIQUE (organization_id, idempotency_key)` y guarda
`payload_json`, `payload_hash`, `attempts`, `last_error`, `next_attempt_at` y
`sent_at`. `sync_inbox` impone `UNIQUE (organization_id, server_sequence)`.
Ambas tablas usan claves foráneas con `ON DELETE CASCADE` a `organizations`.

## Eventos locales

| Tipo | Agregado | Clave de idempotencia |
| --- | --- | --- |
| `sale.created` | `sale` | `sale:{saleId}:created` |
| `sale.returned` | `sale_return` | `sale-return:{returnId}:created` |
| `cash.session_opened` | `cash_session` | `cash-session:{sessionId}:opened` |
| `cash.session_closed` | `cash_session` | `cash-session:{sessionId}:closed` |
| `cash.movement_recorded` | `cash_movement` | `cash-movement:{movementId}:created` |
| `inventory.entry_recorded` | `inventory_movement` | `inventory-movement:{movementId}:recorded` |
| `inventory.adjustment_recorded` | `inventory_movement` | `inventory-movement:{movementId}:recorded` |

El payload usa camelCase y contiene los importes en centavos y las referencias
necesarias para reconstruir el hecho. Las ventas incluyen líneas con variante,
SKU, cantidad, precio, impuesto y totales; las devoluciones incluyen líneas
compensatorias; caja e inventario incluyen turno, método, cantidades y actor.

## API Rust

| Método | Entrada | Salida |
| --- | --- | --- |
| `pending_sync_events` | `organizationId`, `dueAt`, `limit` | `SyncOutboxRecord[]` pendientes y vencidos |
| `list_sync_outbox` | `organizationId`, `limit` | `SyncOutboxRecord[]` recientes |
| `mark_sync_event_sent` | `organizationId`, `eventId`, `sentAt` | `bool` si cambió de `pending` a `sent` |
| `fail_sync_event` | `organizationId`, `eventId`, `error`, `nextAttemptAt`, `maxAttempts` | `SyncFailureRecord` |
| `sync_cursor` | `organizationId` | `lastServerSequence` (0 si no existe) |
| `apply_inbox_event` | `organizationId`, `InboundSyncEvent` | `bool` si insertó y avanzó el cursor |
| `list_sync_inbox` | `organizationId`, `limit` | `SyncInboxRecord[]` recientes |

`SyncEngine::sync_once(organizationId, now)` ejecuta un ciclo:

1. Toma hasta `batchSize` eventos pendientes con `next_attempt_at <= now`.
2. Llama `SyncTransport::push`; cada ACK aceptado marca el evento como `sent`.
3. Un ACK rechazado, ausente o un error de transporte registra el fallo.
4. Lee el cursor y llama `SyncTransport::pull` desde esa secuencia.
5. Aplica cada evento al inbox; los ya vistos se cuentan como duplicados.

Valores por defecto: `batchSize = 50` (1 a 100), `maxAttempts = 5` y
`retryDelaySeconds = 30`. El backoff es `30 s × (attempts + 1)`. Al alcanzar
`maxAttempts` el evento pasa a `dead_letter` y deja de reintentarse
automáticamente. El error se trunca a 500 caracteres.

## Reglas

- El outbox se inserta en la misma transacción que la venta, devolución, turno
  o movimiento; si la transacción falla, no queda evento huérfano.
- Un evento de negocio y su fila de outbox son atómicos; un crash no pierde el
  hecho ni deja media publicación.
- La clave de idempotencia es única por gimnasio. Un reintento del mismo hecho
  no crea otra fila.
- `mark_sync_event_sent` solo afecta filas `pending`; es idempotente.
- `fail_sync_event` solo incrementa `pending`; si el evento ya está `sent` o
  `dead_letter`, devuelve su estado sin cambiar nada.
- El inbox rechaza eventos sin `event_id`, sin `idempotency_key` o con
  `server_sequence < 1`.
- Si `server_sequence <= cursor`, el evento no se aplica y se reporta como
  duplicado. Si el `event_id` ya existe, `ON CONFLICT DO NOTHING` evita duplicar.
- El cursor avanza solo cuando se inserta un evento nuevo, usando el máximo
  entre el cursor actual y la secuencia recibida.
- El inbox guarda el payload y su hash SHA-256; todavía no aplica los eventos a
  las tablas de negocio. Esa aplicación y la resolución de conflictos quedan
  para la integración multi-dispositivo.

## Errores

| Código | Caso |
| --- | --- |
| `invalidValue` | Evento entrante sin identificadores; `maxAttempts < 1`; evento de outbox inexistente |
| `databaseClosed` | La base está cerrada |
| `storage` | Fallo de SQLCipher o de persistencia |
| `invalidPayload` | Payload de outbox o inbox que no es JSON válido |

Los errores de transporte no se persisten como error de base: se registran como
fallo del evento y se reflejan en `SyncReport.transportError`.

## Seguridad

- El journal vive dentro de la misma base SQLCipher; no hay archivos de sync en
  claro.
- No se registran claves, tokens ni payloads en logs.
- El transporte real deberá usar HTTPS con mTLS por dispositivo, enrolamiento y
  rotación de certificados; el `SyncTransport` actual es solo una frontera.
- El servidor deberá deduplicar por `idempotency_key` y validar la organización
  del certificado antes de aceptar un evento.

## Fuera de alcance

Transporte HTTP real, enrolamiento/rotación mTLS, servidor multi-tenant,
resolución de conflictos, aplicación de eventos remotos al negocio, UI de estado
de sync y sincronización de catálogo o personal.
