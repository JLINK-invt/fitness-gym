# Runbook · migraciones SQLite

## Aplicar

Las migraciones corren al abrir la base (`Db::open`) dentro de una transacción.
Cada versión se registra en `schema_migrations` con checksum SHA-256.

## Verificar

```sql
PRAGMA user_version;
SELECT version, name, checksum, applied_at FROM schema_migrations ORDER BY version;
PRAGMA integrity_check;
```

La app expone el mismo estado en la pantalla de diagnóstico. Versión actual: 16
(`0016_sync_journal.sql`). La migración 6 elimina las sedes, limpia los hashes
de auditoría y reinicia la cadena; la migración 7 agrega ventas y líneas
append-only; la migración 8 agrega `users.recovery_code_hash`; la migración 9
agrega `cash_sessions` y enlaza las ventas nuevas con `cash_session_id`; la
migración 10 agrega `cash_movements`, el motivo de diferencia y sus triggers de
inmutabilidad; la migración 11 agrega devoluciones, sus líneas, el ledger
inmutable y el backfill de ventas existentes al ledger; la migración 12 agrega
movimientos de inventario, políticas de mínimo y la vista de balances; la
migración 13 elimina ubicaciones y transferencias, conserva entradas, ventas,
devoluciones y ajustes, y recrea los balances por variante; la migración 14
agrega la clave de idempotencia y el hash de contenido por venta; la migración
15 agrega idempotencia a devoluciones y la tabla append-only `payment_charges`
para cargos y reembolsos conciliables; la migración 16 agrega el journal local
de sincronización (`sync_outbox`, `sync_inbox` y `sync_state`), sin habilitar
transporte de red. Ninguna admite rollback automático: se revierten restaurando
un snapshot anterior.

## Rollback

No hay rollback automático: es forward-only. Para revertir la migración 1 en un
entorno de desarrollo, con la base cerrada y respaldo previo:

```sql
BEGIN;
DROP TABLE IF EXISTS module_events;
DROP TABLE IF EXISTS audit_events;
DROP TABLE IF EXISTS backup_snapshots;
DROP TABLE IF EXISTS staff_memberships;
DROP TABLE IF EXISTS organizations;
DELETE FROM schema_migrations WHERE version = 1;
PRAGMA user_version = 0;
COMMIT;
```

En producción el rollback se ejecuta restaurando un snapshot verificado, no
editando la base a mano. Ver `docs/architecture.md` y el flujo de restore.

## Drift

Si el checksum de una migración ya aplicada no coincide, la app rechaza la
apertura con `MigrationDrift`. Nunca edites una migración publicada: crea una
nueva versión.
