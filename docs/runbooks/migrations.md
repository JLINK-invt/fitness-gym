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

La app expone el mismo estado en la pantalla de diagnóstico. Versión actual: 4
(`0004_immutable_audit.sql`).

## Rollback

No hay rollback automático: es forward-only. Para revertir la migración 1 en un
entorno de desarrollo, con la base cerrada y respaldo previo:

```sql
BEGIN;
DROP TABLE IF EXISTS module_events;
DROP TABLE IF EXISTS audit_events;
DROP TABLE IF EXISTS backup_snapshots;
DROP TABLE IF EXISTS staff_memberships;
DROP TABLE IF EXISTS sites;
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
