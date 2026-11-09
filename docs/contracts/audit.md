# Contrato local · auditoría inmutable (Phase 2)

- **Versión de contrato:** 3.1.0
- **Estado:** implementado (Phase 5: export + restore audit)
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v22 (`0022_critical_settings_approval.sql`)

## Objetivo

Registrar actor, acción, recurso, resultado, origen y correlación para
operaciones sensibles. Cada evento se encadena con el hash del anterior y la
tabla rechaza `UPDATE` y `DELETE` por trigger.

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_audit_events` | `limit?` (1–200, default 100) | `AuditEventSummary[]` | `ViewAudit` |
| `verify_audit_chain` | — | `AuditIntegrity` | `ViewAudit` |
| `export_audit_csv` | — | `AuditExport` | `ViewAudit` |

`list_audit_events` devuelve los eventos más recientes primero. Todos los roles
con `ViewAudit` ven los eventos de la instalación.

`export_audit_csv` genera un CSV con todos los eventos en orden de cadena
(ASC por `sequence`), incluye metadatos del gimnasio, resumen de integridad y
un archivo `.sha256` sidecar para verificación fuera de la app.

## DTOs

```ts
type AuditResult = "allowed" | "denied" | "failed";

interface AuditEventSummary {
  id: string;
  sequence: number;
  actorId: string | null;
  actorName: string | null;
  action: string;
  resource: string;
  result: AuditResult;
  origin: string;
  correlationId: string | null;
  createdAt: string;
}

interface AuditIntegrity {
  valid: boolean;
  checkedEvents: number;
  firstBrokenSequence: number | null;
}

interface AuditExport {
  fileName: string;
  path: string;
  bytes: number;
  checksum: string;
  events: number;
  integrityValid: boolean;
}
```

## Esquema

`audit_events` conserva las columnas originales y agrega:

- `sequence`: consecutivo por gimnasio, índice único `(organization_id, sequence)`.
- `origin`: fuente del evento (`tauri` para comandos locales).
- `prev_hash`: hash del evento anterior de la cadena.
- `event_hash`: SHA-256 de los campos canónicos, incluido `prev_hash`.
- `export_audit_csv` exporta también `prev_hash` y `event_hash` por evento.

La migración 4 renumera los eventos heredados y los conserva con `origin =
'legacy'` sin hash. La migración 6 elimina la columna `site_id`, limpia los hashes
existentes y reinicia la cadena con el formato canónico
`fitness-gym-audit-v2`. La cadena verificable empieza en el primer evento posterior
a esa migración; los eventos heredados siguen siendo inmutables por trigger.

## Acciones registradas

- `auth.owner_created`, `auth.login`, `auth.logout`.
- `auth.owner_recovery_generated`, `auth.owner_recovered`.
- `staff.created`, `staff.updated`.
- `access.simulated` solo cuando se inserta un evento nuevo (no en replay
  idempotente).
- `catalog.product_created`, `catalog.variant_created`,
  `catalog.product_updated`, `catalog.variant_updated` y `catalog.price_added`.
- `sale.completed`, con `correlationId` igual al id de la venta.
- `backup.created`, `backup.restored`, `backup.restore_failed`, `backup.pruned`.
- `settings.change_requested`, `settings.change_approved`, `settings.change_rejected`.
- `report.exported` (finanzas), `report.exported` (reportes programados), `audit.exported`.
- `sync.completed`, `sync.settings_saved`.

Las exportaciones y operaciones de finanzas/inventario se registran cuando esos
módulos existen.

## Inmutabilidad y límites

- `audit_events_no_update` y `audit_events_no_delete` abortan cualquier intento de
  modificar o borrar una fila.
- La UI es solo lectura: no hay comandos de edición o borrado.
- `verify_audit_chain` recalcula el hash de cada evento y valida `prev_hash` y el
  consecutivo. Devuelve el primer `sequence` roto.
- `export_audit_csv` produce un CSV inmutable con checksum SHA-256 y archivo
  `.sha256` sidecar para verificación fuera de la app.
- Un atacante con acceso directo al archivo y la clave puede eliminar el trigger
  o truncar el final de la cadena; la exportación firmada y el anclaje monotónico
  mitigan ese riesgo residual.

## Errores

| Código | Caso |
| --- | --- |
| `unauthorized` | Sesión ausente o rol sin `ViewAudit` |
| `storage` | Fallo al leer o verificar la auditoría |
| `internal` | Error inesperado del core |

## Fuera de alcance

Envío a un colector externo, retención automática y búsqueda avanzada. La
consulta actual es por límite, sin filtros por acción o fecha. La exportación
firmada está implementada (`export_audit_csv`).
