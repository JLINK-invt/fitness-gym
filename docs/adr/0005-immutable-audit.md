# ADR-0005 · Auditoría append-only con hash chain local

- **Estado:** Accepted
- **Fecha:** 2026-10-22
- **Supersede:** riesgo residual de ADR-0002 sobre auditoría manipulable

## Contexto

Las operaciones sensibles ya registraban actor, acción, recurso y resultado en
`audit_events`, pero una fila podía editarse o borrarse con SQL. El threat model
marcaba la manipulación de auditoría como pendiente y la UI no ofrecía consulta.

## Decisión

1. `audit_events` es append-only mediante triggers que abortan `UPDATE` y
   `DELETE`.
2. Cada evento lleva un `sequence` por gimnasio, el hash del evento anterior y su
   propio hash SHA-256 sobre campos canónicos.
3. La escritura calcula consecutivo y hash dentro de la misma transacción.
4. La UI expone una pantalla de solo lectura con `ViewAudit`; el dueño ve todo el
   gimnasio y los roles con sede solo ven sus eventos.
5. `verify_audit_chain` recalcula la cadena y reporta el primer `sequence` roto.
6. Los eventos heredados de esquema 3 se conservan sin hash; la cadena empieza en
   el primer evento posterior a la migración 4.

## Consecuencias

- Una modificación de contenido o un borrado intermedio se detecta al verificar.
- El rendimiento de escritura es aceptable porque el acceso a SQLite ya está
  serializado y la cadena se calcula por evento.
- Un atacante con la clave puede eliminar triggers o truncar el final de la
  cadena. La detección fuerte requiere una copia externa o un ancla monotónica.
- La exportación firmada y el colector externo quedan para una fase posterior.

## Alternativas descartadas

- **Solo permisos de aplicación:** no protege ante SQL directo.
- **Tabla de solo inserción sin hash:** detecta edición accidental, no cambios
  deliberados.
- **Bloquear SQLite en solo lectura:** impide la operación normal del gimnasio.
