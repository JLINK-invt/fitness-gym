# Arquitectura de la fundación

## Flujo

```text
React (apps/gym-desktop/src)
  │ invoke() tipado con @gym/contracts
  ▼
Comandos Tauri (apps/gym-desktop/src-tauri)
  │
  ├── gym-core      reglas puras, RBAC y bus de eventos
  ├── gym-data      SQLCipher + migraciones + key provider
  └── gym-backup    snapshots cifrados + restore verificado
```

La nube no participa en la operación. Todo el flujo anterior es local.

## Límites de dependencia

- `gym-core` no depende de Tauri, SQLite ni del sistema operativo.
- `gym-contracts` solo contiene DTOs serializables y errores de API.
- `gym-data` depende de `gym-core` y `gym-contracts`; nunca al revés.
- `gym-backup` depende de `gym-data`; no expone detalles de SQLCipher a la UI.
- La app Tauri es la única capa que conoce comandos, ventanas y rutas del SO.
- `packages/contracts` es el espejo TypeScript de `gym-contracts`; un cambio
  incompatible exige subir `CONTRACT_VERSION`.

## Persistencia

- Cada base local es un archivo SQLCipher con WAL.
- La clave se inyecta con `PRAGMA key` antes de cualquier lectura.
- `schema_migrations` guarda versión, nombre, checksum y fecha.
- Las tablas operativas llevan `organization_id NOT NULL`; las de sede llevan
  además `site_id`.
- `module_events` impone `UNIQUE (organization_id, idempotency_key)`.
- El acceso simulado usa `access_decision` como agregado y
  `access.allowed`/`access.denied` como tipos de evento.

## Identidad y sesión

- `users` guarda nombre, hash Argon2id y estado; `staff_memberships` guarda rol y
  alcance de sede. Solo puede existir un `owner` activo por instalación.
- El primer arranque crea gimnasio y dueño en una sola operación; no hay usuario
  ni contraseña por defecto.
- La contraseña nunca cruza al webview después del formulario; el hash queda en
  SQLCipher y la sesión vive en memoria del proceso Tauri.
- Cinco fallos bloquean el nombre durante cinco minutos; 15 minutos de
  inactividad cierran la sesión.
- Los comandos operativos reciben el actor desde la sesión Rust, nunca desde un
  `userId` enviado por la UI.
- Restaurar un backup cierra la sesión porque la base restaurada puede tener otro
  dueño o personal revocado.

## Acceso simulado

La UI de Acceso llama a `simulate_access`; `gym-core` decide sin I/O y
`gym-data` inserta una sola vez en `module_events`. La misma clave con el mismo
lector y credencial devuelve el evento original; con datos distintos se rechaza.
El contrato completo está en
[`contracts/access-simulated.md`](contracts/access-simulated.md).

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
