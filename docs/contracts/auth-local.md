# Contrato local · autenticación y sesión (Phase 2)

- **Versión:** 1.2.0 (aditiva sobre 1.1.0)
- **Estado:** implementado para el primer slice de Phase 2
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red

## Flujo

1. La UI llama `get_setup_state` para saber si falta el gimnasio o el dueño.
2. En el primer arranque, `bootstrap_owner` crea gimnasio y dueño en una sola
   operación. Si ya existe un dueño, falla con `invalidRequest`.
3. `login` verifica Argon2id, crea una sesión en memoria y devuelve
   `SessionSummary`.
4. Cada comando operativo revalida rol y permiso contra la sesión Rust.
5. `logout` y un restore exitoso eliminan la sesión.

No existe usuario ni contraseña por defecto. El webview nunca recibe hashes,
salts ni la clave de la base.

## Comandos

| Comando | Entrada | Salida | Protección |
| --- | --- | --- | --- |
| `get_setup_state` | — | `SetupState` | Pública |
| `bootstrap_owner` | `gymName?`, `name`, `password` | `SessionSummary` | Solo si no hay dueño |
| `login` | `name`, `password` | `SessionSummary` | Pública con bloqueo |
| `logout` | — | `null` | Sesión actual |
| `current_session` | — | `SessionSummary \| null` | Expira por inactividad |

`bootstrap_owner` solo usa `gymName` si el equipo aún no tiene gimnasio.

## Reglas

- Nombre de gimnasio: 1–120 caracteres visibles tras `trim`.
- Nombre de persona: 2–80 caracteres; único por gimnasio sin distinguir
  mayúsculas.
- Contraseña del dueño: mínimo 12 caracteres. El resto del personal: mínimo 8.
- Cinco fallos por nombre bloquean nuevos intentos durante cinco minutos.
- La sesión expira tras 15 minutos sin actividad.
- Roles: `owner`, `manager`, `cashier`, `accounting`.
- `siteId` solo puede ser `null` para el dueño; el personal operativo requiere
  sede.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Datos inválidos, equipo ya configurado o dueño existente |
| `unauthorized` | Credenciales inválidas, cuenta bloqueada o sesión ausente/expirada |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

Los mensajes de credenciales inválidas no distinguen si el nombre existe.

## Fuera de alcance

MFA, OIDC, recuperación por correo, múltiples dueños, permisos personalizados y
sesiones persistentes entre reinicios. La recuperación del dueño requiere una
decisión posterior.
