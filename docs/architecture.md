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
- `sites` tiene nombre único por gimnasio sin distinguir mayúsculas. No se borra
  una sede: se cierra, y conserva su historial.
- `staff_memberships` impone una membresía por usuario e índice único por
  gimnasio y usuario.
- `module_events` impone `UNIQUE (organization_id, idempotency_key)`.
- `audit_events` es append-only por trigger, con `sequence` por gimnasio y hash
  chain SHA-256 (`prev_hash` + `event_hash`). La migración 4 conserva los eventos
  heredados sin hash y la cadena verificable empieza después.
- `catalog_products` conserva productos y variantes con archivo lógico;
  `catalog_price_versions` es append-only y una venta futura deberá referenciar
  la versión de precio usada, no solo el producto.
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
- El dueño opera sobre todo el gimnasio (`site_id = NULL`). Un encargado solo ve
  y administra cajeros y contabilidad de su propia sede; no puede crear otro
  encargado ni mover personal a otra sede.
- El dueño no se edita desde la UI: se crea solo en la configuración inicial. Un
  usuario tampoco puede modificar su propia membresía.
- Revocar una cuenta impide el siguiente login; la sesión en memoria se invalida
  al expirar o al hacer logout. Restaurar un backup cierra la sesión porque la
  base restaurada puede tener otro dueño o personal revocado.
- `auth.owner_created`, `auth.login`, `auth.logout`, `site.created`,
  `site.updated`, `staff.created`, `staff.updated`, `access.simulated` y
  `backup.created` se registran en `audit_events` con el actor de la sesión.

## Auditoría

Cada evento sensible se agrega en una transacción que calcula el consecutivo y
el hash del evento anterior. La tabla rechaza `UPDATE` y `DELETE`; la UI solo
consulta y puede verificar la cadena. El dueño ve todo el gimnasio y los roles con
sede solo ven sus propios eventos. La cadena detecta alteraciones de contenido,
pero no un truncado del final sin una copia externa; ese riesgo queda documentado.

El contrato completo está en [`contracts/audit.md`](contracts/audit.md).

## Sedes y personal

La pantalla **Sedes** requiere `ManageSettings` y permite crear, renombrar,
cerrar y reabrir sedes. Cerrar exige que no queden miembros activos asignados.
La pantalla **Equipo** requiere `ManageStaff` y permite crear accesos, cambiar
rol o sede, y desactivar o reactivar cuentas.

Reglas de autorización aplicadas en Rust:

- No se puede crear ni elevar una cuenta a `owner`.
- Un `manager` solo administra `cashier` y `accounting` de su sede.
- No se asigna personal a una sede cerrada.
- Solo puede existir una membresía por usuario.
- Los comandos filtran por `organization_id` y, para `manager`, por `site_id`.

El contrato completo está en
[`contracts/sites-staff.md`](contracts/sites-staff.md).

## Catálogo y precios

La pantalla **Catálogo** requiere `ManageCatalog` y permite crear productos con
una variante inicial, editar producto o variante, archivarlos y agregar precios.
Cada precio es una versión inmutable con monto, tasa de impuesto, indicador de
impuesto incluido y `effective_from`; la UI muestra la versión vigente según la
hora actual. La moneda proviene del gimnasio. Los SKU y códigos de barras son
únicos por gimnasio. El contrato completo está en
[`contracts/catalog.md`](contracts/catalog.md).

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
