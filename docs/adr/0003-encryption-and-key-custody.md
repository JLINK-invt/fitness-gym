# ADR-0003 · Cifrado en reposo y custodia de clave

- **Estado:** Accepted (2026-10-20)
- **Fecha:** 2026-10-20
- **Decide:** Cifrado de la base y manejo de secretos
- **Relacionado:** Phase 1, Phase 5

## Contexto

La base contiene PII, pagos y accesos. El modelo de amenaza incluye robo del
equipo, copia del archivo y backups exfiltrados. La app debe operar sin pedir
una passphrase en cada arranque, pero sin dejar la clave junto a la base.

## Decisión

1. **SQLCipher 4** con cifrado de páginas AES-256-CBC y HMAC-SHA512. La clave es
   aleatoria de 256 bits generada con CSPRNG, no derivada de una passphrase.
2. **Custodia en el keychain del sistema operativo** mediante el crate `keyring`:
   Secret Service/libsecret en Linux, Keychain en macOS y Credential Manager en
   Windows. El servicio identifica la app (`com.fitnessgym.desktop`) y la
   entrada queda aislada por usuario y dispositivo del SO.
3. **Fallback de desarrollo/CI:** archivo `0600` o variable `GYM_DB_KEY`, nunca
   en producción. El archivo `keys/db.key` es solo bootstrap; se migra al
   keychain en el arranque y se elimina (implementado el 2026-10-21).
4. **La clave nunca se escribe en logs, errores, telemetría ni backups.** Se
   mantiene en memoria el menor tiempo posible y se zeroiza al soltar.
5. **SQLCipher endurecido:** `cipher_memory_security = ON`, WAL, `synchronous =
   NORMAL`, `foreign_keys = ON`, `busy_timeout = 5000`. La clave se aplica antes
   de cualquier lectura.
6. **Rotación de clave:** `PRAGMA rekey` dentro de una ventana de mantenimiento,
   con backup verificado previo. Se implementará después del vertical slice.
7. **Passphrase de backups separada** de la clave de la base y nunca almacenada
   por la app. Ver ADR-0004.
8. **Cifrado de disco del SO recomendado** (BitLocker, LUKS, FileVault) como
   defensa en profundidad, no como sustituto.

## Implementación (2026-10-21)

- `KeyringKeyProvider` usa `keyring` con Secret Service en Linux, Keychain en
  macOS y Credential Manager en Windows.
- Al arrancar, si el keychain no tiene la clave y existe `keys/db.key`, la clave
  se importa, se guarda en el keychain y el archivo se elimina.
- `GYM_DB_KEY` tiene prioridad para CI. En builds de desarrollo, si el keychain
  no está disponible se usa el archivo `0600`; un build release falla cerrado.
- El identificador usado es servicio `com.fitnessgym.desktop` y cuenta
  `database-key`. Al ser una organización por instalación, el alcance es el
  usuario y el dispositivo.

## Consecuencias

- Si el keychain no está disponible (servidor sin sesión, CI), la app degrada a
  modo desarrollo y lo advierte; no crea una base sin cifrar.
- Perder el keychain y no tener backups con passphrase implica pérdida de datos.
- El usuario que controla la sesión desbloqueada puede leer la base; el cifrado
  protege el disco en reposo, no una sesión comprometida.
- La migración de `db.key` al keychain es idempotente; el archivo se elimina
  solo después de guardar la clave en el keychain.

## Alternativas descartadas

- **SQLite en claro + permisos de archivo:** insuficiente ante robo o copia.
- **Passphrase en cada arranque:** fricción inaceptable en recepción.
- **Clave derivada solo de la contraseña del usuario:** bloquea el arranque
  desatendido y complica backups/restore.
- **Stronghold:** vault con passphrase propia; añade UX y estado que hoy no se
  justifica. Se puede reevaluar si el keychain no cubre un requisito.

## Criterios de revisión

- Si el keychain no está disponible o falla en un sistema objetivo.
- Antes de implementar rotación de clave o sincronización multi-dispositivo.
- Si se requiere arranque desatendido en un servidor sin sesión de usuario.
- Después de una auditoría criptográfica o de un incidente de clave expuesta.

## Decisión pendiente

Ninguna. La implementación del keychain queda como criterio de salida de Phase 1.
