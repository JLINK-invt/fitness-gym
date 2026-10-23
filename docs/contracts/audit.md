# Contrato local · auditoría inmutable (Phase 2)

- **Versión de contrato:** 1.5.0 (sin cambios de DTO; compatible con 1.4.0)
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v4 (`0004_immutable_audit.sql`)

## Objetivo

Registrar actor, acción, recurso, resultado, sede, origen y correlación para
operaciones sensibles. Cada evento se encadena con el hash del anterior y la
tabla rechaza `UPDATE` y `DELETE` por trigger.

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_audit_events` | `limit?` (1–200, default 100) | `AuditEventSummary[]` | `ViewAudit` |
| `verify_audit_chain` | — | `AuditIntegrity` | `ViewAudit` |

`list_audit_events` devuelve los eventos más recientes primero. El dueño ve todo
el gimnasio; encargado y contabilidad solo ven eventos de su sede. Los eventos
sin sede (por ejemplo un login fallido) no se muestran a roles con alcance de
sede.

## DTOs

```ts
type AuditResult = "allowed" | "denied" | "failed";

interface AuditEventSummary {
  id: string;
  sequence: number;
  actorId: string | null;
  actorName: string | null;
  siteId: string | null;
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
```

## Esquema

`audit_events` conserva las columnas originales y agrega:

- `sequence`: consecutivo por gimnasio, índice único `(organization_id, sequence)`.
- `site_id`: sede del recurso o del actor cuando aplica.
- `origin`: fuente del evento (`tauri` para comandos locales).
- `prev_hash`: hash del evento anterior de la cadena.
- `event_hash`: SHA-256 de los campos canónicos, incluido `prev_hash`.

La migración 4 renumera los eventos heredados y los conserva con `origin =
'legacy'` sin hash. La cadena verificable empieza en el primer evento posterior a
la migración; los eventos heredados siguen siendo inmutables por trigger.

## Acciones registradas

- `auth.owner_created`, `auth.login`, `auth.logout`.
- `site.created`, `site.updated`.
- `staff.created`, `staff.updated`.
- `access.simulated` solo cuando se inserta un evento nuevo (no en replay
  idempotente).
- `backup.created`.

Las exportaciones y operaciones de finanzas/inventario se agregarán cuando esos
módulos existan.

## Inmutabilidad y límites

- `audit_events_no_update` y `audit_events_no_delete` abortan cualquier intento de
  modificar o borrar una fila.
- La UI es solo lectura: no hay comandos de edición o borrado.
- `verify_audit_chain` recalcula el hash de cada evento y valida `prev_hash` y el
  consecutivo. Devuelve el primer `sequence` roto.
- Un atacante con acceso directo al archivo y la clave puede eliminar el trigger
  o truncar el final de la cadena; la copia externa y el anclaje monotónico se
  difieren a una fase posterior. El threat model registra ese riesgo residual.

## Errores

| Código | Caso |
| --- | --- |
| `unauthorized` | Sesión ausente o rol sin `ViewAudit` |
| `storage` | Fallo al leer o verificar la auditoría |
| `internal` | Error inesperado del core |

## Fuera de alcance

Exportación firmada, envío a un colector externo, retención automática y
búsqueda avanzada. La consulta actual es por sede y límite, sin filtros por
acción o fecha.
