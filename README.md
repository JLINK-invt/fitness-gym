# Fitness Gym · Operación local

Core de escritorio para gimnasios independientes: Tauri v2 + React + TypeScript,
backend Rust modular y SQLite cifrada con SQLCipher.

La **Fase 1 — Fundación desktop Rust/SQLite** está cerrada. La **Fase 2** inicia
con identidad local, sesiones y autorización real en Rust antes de construir
Recepción, miembros y cobros.

## Estado actual

| Capacidad | Estado |
| --- | --- |
| Persistencia SQLCipher, migraciones y backups | Operativo; esquema v2 |
| Acceso simulado con evento idempotente | Operativo como herramienta interna de prueba |
| Primer arranque y cuenta del dueño | Operativo: Argon2id, mínimo 12 caracteres, sin usuario por defecto |
| Login local y sesión en memoria | Operativo: bloqueo tras 5 fallos y expiración por 15 min de inactividad |
| RBAC por rol en comandos Tauri | Operativo para dueño; personal y sedes llegan en la siguiente tarea |
| Recepción, miembros, planes y cobros | Siguiente vertical slice de Fase 2 |
| Validación Windows/macOS | Diferida a Fase 6 por falta de hardware macOS |
| Smoke E2E de interfaz | 9 comprobaciones en verde con IPC simulado |

La app abre la base local, aplica migraciones, configura el gimnasio y su dueño,
exige login, simula accesos con permisos revalidados en Rust y crea snapshots
cifrados. Funciona sin Internet. Solo se admite un gimnasio por instalación; la
pantalla no expone conceptos de multi-organización.

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

El script cubre primer arranque con dueño, health, acceso permitido/denegado con
idempotencia, validación de frase, creación de backup, rechazo de frase
incorrecta, restauración y cierre de sesión.

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
- [Matriz de permisos](docs/security/permission-matrix.md): roles, sedes y acciones.
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
- Acceso simulado protegido por sesión y permiso de operación, sin guardar la
  credencial y con deduplicación durable por `idempotency_key`.
- Logs sin claves ni passphrases.

## Siguientes pasos de Phase 2

1. CRUD de sedes y personal con roles y alcance por sede.
2. Recepción real: búsqueda de miembros, membresías, cobros y check-in.
3. Auditoría completa de acciones sensibles y política de recuperación del dueño.
4. Validar el binario Tauri en Windows y macOS durante Phase 6; Playwright solo
   cubre la UI con IPC simulado.

Contratos: [acceso simulado](docs/contracts/access-simulated.md) y
[auth local](docs/contracts/auth-local.md).
