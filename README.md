# Fitness Gym · Operación local

Core de escritorio para gimnasios independientes: Tauri v2 + React + TypeScript,
backend Rust modular y SQLite cifrada con SQLCipher.

Las **Fases 1 y 2** están cerradas: fundación desktop Rust/SQLite, identidad
local, sesiones, tenancy, RBAC y auditoría inmutable. La **Fase 3** añade POS,
caja e inventario. El catálogo con precios versionados, las ventas con cobro, la
caja por turnos y el control de inventario ya están operativos. CRM básico,
socios y membresías forman parte de la misma versión desktop. Mobile, growth y
suscripciones separadas quedan fuera de la V1.

## Estado actual

| Capacidad | Estado |
| --- | --- |
| Persistencia SQLCipher, migraciones y backups | Operativo; esquema v15 |
| Acceso simulado con evento idempotente | Oculto en la UI; el backend se conserva para Phase 4 |
| Primer arranque y cuenta del dueño | Operativo: Argon2id, mínimo 12 caracteres, código de recuperación de un solo uso |
| Login local y sesión en memoria | Operativo: bloqueo tras 5 fallos y expiración por 15 min de inactividad |
| Recuperación del dueño | Operativo: código de un solo uso, sin correo ni autenticador |
| RBAC por rol en comandos Tauri | Operativo para auth, personal, catálogo, ventas, caja e inventario |
| Personal | Operativo: crear accesos, cambiar rol y desactivar/reactivar con auditoría |
| Auditoría inmutable | Operativo: append-only, hash chain y verificación de integridad |
| Catálogo, precios e impuestos | Operativo: productos, variantes, precios versionados y archivo lógico |
| Ventas y cobro | Operativo: carrito, impuestos, efectivo/transferencia/Point, devoluciones parciales y ticket no fiscal |
| Caja y turnos | Operativo: fondo inicial, ingresos/gastos con evidencia, cierre con diferencia auditada e historial |
| Inventario y valoración | Operativo: entradas, ajustes, umbrales y valoración por costo promedio |
| Panel operativo | Operativo: existencias, valor, alertas de mínimo y rotación básica |
| Suscripciones V1 | Diferidas; se unifican con membresías de socios en Phase 4 |
| CRM básico, socios y membresías | Misma versión desktop, Fase 4 |
| Validación Windows/macOS | Diferida a Fase 5 por falta de hardware macOS |
| Smoke E2E de interfaz | 28 comprobaciones en verde con IPC simulado |

La app abre la base local, aplica migraciones, configura el gimnasio y su dueño,
entrega un código de recuperación, exige login, administra accesos del equipo con
permisos revalidados en Rust,
registra auditoría append-only con hash chain, administra el catálogo con precios
versionados, abre y cierra turnos de caja, registra ingresos y gastos con evidencia, cobra ventas con snapshot de precio,
registra devoluciones parciales contra el ledger inmutable, controla inventario por
movimientos append-only, muestra un panel con alertas de mínimo y rotación básica,
y crea snapshots cifrados. Funciona sin Internet. Solo se admite un gimnasio por
instalación y opera con una sola ubicación lógica de inventario; no existen
conceptos de multi-organización, multi-sede ni transferencias.

## Requisitos

- Node.js >= 22 y pnpm >= 11
- Rust stable (1.85+) con `rustfmt` y `clippy`. Si rustup se instaló sin
  modificar el PATH, `cargo` no estará disponible hasta ejecutar:

  ```bash
  source "$HOME/.cargo/env"
  ```

  Para que persista, añade esa línea a `~/.bashrc` o `~/.zshrc`.
- Dependencias de Tauri y SQLite en el sistema

Fedora:

```bash
sudo dnf install pkgconf-pkg-config dbus-devel webkit2gtk4.1-devel gtk3-devel \
  libappindicator-gtk3-devel librsvg2-devel openssl-devel sqlcipher-devel
```

Ubuntu/Debian:

```bash
sudo apt install build-essential pkg-config libssl-dev libdbus-1-dev \
  libgtk-3-dev libwebkit2gtk-4.1-dev librsvg2-dev \
  libayatana-appindicator3-dev patchelf
```

> SQLCipher se compila desde el crate `rusqlite` con la feature
> `bundled-sqlcipher`; `sqlcipher-devel` es opcional si prefieres enlazar el
> paquete del sistema.

## Puesta en marcha

```bash
source "$HOME/.cargo/env"   # solo si `cargo --version` falla
pnpm install
pnpm tauri dev        # app desktop con backend Rust
pnpm dev              # solo frontend en el navegador (sin comandos Tauri)
```

Comprueba antes que Tauri encuentra Rust:

```bash
cargo --version
rustc --version
```

## Verificación

```bash
pnpm lint
pnpm typecheck
pnpm test
pnpm build

cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings
cargo test --workspace
```

El workflow [`.github/workflows/ci.yml`](.github/workflows/ci.yml) reproduce
estos pasos en Ubuntu.

### Smoke E2E de interfaz

El frontend incluye un modo de desarrollo con IPC simulado para probar la UI en
un navegador. Solo se activa con `import.meta.env.DEV` y `?e2e=1`; no modifica el
binario de producción ni sustituye las pruebas Rust del backend.

```bash
pnpm --filter gym-desktop dev
npx -y @playwright/cli@latest open "http://localhost:1420/?e2e=1#/" --headed
npx -y @playwright/cli@latest run-code --filename=apps/gym-desktop/e2e/smoke.js
```

El script cubre primer arranque con dueño, health, validación de frase, creación
de backup, rechazo de frase incorrecta, restauración y cierre de sesión. También crea un producto con precio
inicial, una nueva versión de precio y una segunda variante, además de un cajero.
Comprueba que el cajero no ve módulos administrativos, registra un gasto de caja,
cobra una venta, cierra el turno con una diferencia justificada, verifica la
cadena de auditoría, confirma que una cuenta desactivada ya no puede iniciar
sesión y que el código de recuperación permite fijar una contraseña nueva.

## Estructura

```text
apps/gym-desktop/          Tauri v2 + React/Vite/TS
  src-tauri/               comandos y estado del core
crates/
  gym-core/                reglas puras: ids, RBAC, eventos idempotentes
  gym-contracts/           DTOs versionados (camelCase)
  gym-data/                SQLCipher, migraciones y key providers
  gym-backup/              snapshots cifrados y restauración
packages/
  contracts/               espejo TypeScript + JSON Schema
  config/                  tsconfig compartido
docs/
  architecture.md          límites, flujo y criterios de la fase
  runbooks/migrations.md   rollback y operación de migraciones
```

## Decisiones y seguridad

- [ADRs](docs/adr/README.md): arquitectura local-first, identidad, cifrado y backups.
- [Threat model](docs/threat-model.md): STRIDE, activos, fronteras y riesgos residuales.
- [Matriz de permisos](docs/security/permission-matrix.md): roles y acciones.
- [Contrato de personal](docs/contracts/staff.md): comandos y reglas.
- [Contrato de ventas](docs/contracts/sales.md): cobro, impuestos y ticket.
- [Contrato de caja](docs/contracts/cash.md): turnos, arqueo y diferencias.
- [Contrato de inventario](docs/contracts/inventory.md): stock, valoración y movimientos.
- [Contrato de auditoría](docs/contracts/audit.md): append-only, hash chain y consulta.
- [Runbook de migraciones](docs/runbooks/migrations.md).

## Seguridad implementada

- Contraseñas con Argon2id y salt único; el dueño exige 12 caracteres.
- Sesión solo en memoria del proceso Tauri, con expiración por inactividad.
- Cada comando operativo revalida rol y permiso en Rust; la UI solo oculta acciones.
- SQLCipher con clave de 256 bits fuera de la base.
- Clave en el keychain del SO (`keyring`: Secret Service, Keychain o Credential
  Manager). Las instalaciones anteriores migran `keys/db.key` al keychain y
  eliminan el archivo. `GYM_DB_KEY` o el archivo `0600` quedan solo para
  desarrollo/CI; un build release falla cerrado si el keychain no está disponible.
- Snapshots: SQLCipher + Argon2id + XChaCha20-Poly1305, checksum SHA-256 y
  verificación de integridad antes de reemplazar la base.
- CSP estricta en producción y `core:default` como única capability; sin shell
  genérico ni filesystem amplio.
- Migraciones con checksum y `user_version`; drift detectado al abrir.
- El backend de acceso simulado conserva validación de sesión, permiso y
  deduplicación durable por `idempotency_key`, pero no se expone en la UI de V1.
- Auditoría append-only con triggers anti `UPDATE`/`DELETE`, hash chain SHA-256 y
  verificación de integridad desde la UI.
- Logs sin claves ni passphrases.

## Siguientes pasos

1. Phase 3: cobros, ticket no fiscal y reportes operativos.
2. Phase 4: CRM básico, socios, membresías, accesos y analítica operativa.
3. Phase 5: hardening, piloto y validación/empaquetado en Windows/macOS.
4. Fuera de la V1: mobile, growth (leads, campañas, referidos, churn), suscripciones
   V1, escáner de productos y multi-sede.

Contratos: [acceso simulado](docs/contracts/access-simulated.md),
[auth local](docs/contracts/auth-local.md),
[personal](docs/contracts/staff.md),
[catálogo](docs/contracts/catalog.md),
[ventas](docs/contracts/sales.md),
[caja](docs/contracts/cash.md),
[inventario](docs/contracts/inventory.md),
[reportes](docs/contracts/reports.md) y
[auditoría](docs/contracts/audit.md).
