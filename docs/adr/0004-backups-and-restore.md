# ADR-0004 · Backups y restauración

- **Estado:** Accepted (2026-10-20)
- **Fecha:** 2026-10-20
- **Decide:** Formato, verificación y operación de backups
- **Relacionado:** Phase 1, Phase 4

## Contexto

El gimnasio no tiene DBA. Un backup que no se puede restaurar no es un backup.
El archivo de la base ya está cifrado con la clave del keychain, pero una copia
del equipo o del backup no debe ser legible sin un secreto adicional.

## Decisión

1. **Snapshot completo, no incremental**, con `VACUUM INTO` sobre la base
   abierta. Produce una copia SQLCipher consistente sin exponer texto plano.
2. **Sobre cifrado propio** con XChaCha20-Poly1305. La clave se deriva de una
   passphrase con Argon2id v19 (m=19 MiB, t=2, p=1, salida 32 bytes).
   Estos parámetros quedan fijados para el formato v1.
3. **Formato versionado:** magia `FGYMBAK1`, salt de 16 bytes, nonce de 24 bytes
   y ciphertext. Sidecar JSON con id, nombre, fecha, tamaño, checksum SHA-256,
   versión de esquema y algoritmos.
4. **Escritura atómica** del sobre y del sidecar. Nunca se sobrescribe un backup
   existente.
5. **Restore verificado:** checksum del sobre, descifrado, `PRAGMA
   integrity_check` con la clave de la base y reemplazo atómico con rollback.
   Antes del swap se cierra la conexión, se hace checkpoint del WAL y se
   eliminan `-wal`/`-shm` obsoletos.
6. **Passphrase mínima de 8 caracteres**, nunca almacenada. Se recomienda
   guardarla en un gestor de contraseñas y probar el restore cada trimestre.
7. **Phase 1 solo backup local manual.** La copia a nube, la programación y la
   retención automática se deciden en Phase 4 con cifrado de extremo a extremo.
8. **Cada backup se valida al crearlo** con el checksum del sobre y un
   `quick_check` de la base de origen; la integridad completa del snapshot se
   revalida en el restore. Se registrará en `backup_snapshots` cuando el módulo
   de auditoría esté activo.

## Consecuencias

- Perder la passphrase equivale a perder los backups; no hay escrow en Phase 1.
- El espacio requerido es el tamaño de la base por snapshot; la UI debe mostrar
  el uso y permitir borrar copias antiguas con auditoría.
- El restore reemplaza toda la base: no hay merge ni restauración parcial.
- Los parámetros de Argon2id no se pueden cambiar sin subir la versión del
  formato y conservar el lector anterior.

## Alternativas descartadas

- **Copiar el archivo SQLCipher directamente:** puede capturar un estado
  inconsistente con WAL activo.
- **Backup en claro comprimido:** expone PII.
- **Cifrar con la misma clave de la base:** un backup filtrado compromete la
  base y viceversa.
- **Incremental desde el inicio:** complejidad y riesgo de cadenas rotas sin
  necesidad en Phase 1.

## Criterios de revisión

- Antes de añadir backup a nube, programación o retención automática.
- Si el tamaño o el tiempo de backup afecta la operación diaria.
- Al cambiar parámetros de KDF o el formato (requiere versión nueva).
- Tras un simulacro de restore fallido o un incidente de pérdida de datos.

## Decisión pendiente

Definir en Phase 4 el destino cloud y la política de retención; no bloquea
Phase 1.
