# Contrato local · autenticación y sesión (Phase 2)

- **Versión de contrato:** 3.0.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v21 (`0021_gym_operations.sql`)

## Flujo

1. La UI llama `get_setup_state` para saber si falta el gimnasio o el dueño.
2. En el primer arranque, `bootstrap_owner` crea gimnasio y dueño en una sola
   operación. Si ya existe un dueño, falla con `invalidRequest`.
3. `login` verifica Argon2id, crea una sesión en memoria y devuelve
   `SessionSummary`.
4. `bootstrap_owner` devuelve también un código de recuperación de un solo uso.
5. Si el dueño olvida su contraseña, `recover_owner` verifica ese código, fija
   una contraseña nueva, rota el código y abre sesión.
6. Cada comando operativo revalida rol y permiso contra la sesión Rust.
7. `logout` y un restore exitoso eliminan la sesión.

No existe usuario ni contraseña por defecto. El webview nunca recibe hashes,
salts ni la clave de la base.

## Comandos

| Comando | Entrada | Salida | Protección |
| --- | --- | --- | --- |
| `get_setup_state` | — | `SetupState` | Pública |
| `bootstrap_owner` | `gymName?`, `name`, `password` | `RecoveryKit` | Solo si no hay dueño |
| `generate_owner_recovery` | — | `RecoveryCode` | Dueño autenticado (`ManageSettings`) |
| `recover_owner` | `recoveryCode`, `newPassword` | `RecoveryKit` | Pública con bloqueo |
| `login` | `name`, `password` | `SessionSummary` | Pública con bloqueo |
| `logout` | — | `null` | Sesión actual |
| `current_session` | — | `SessionSummary \| null` | Expira por inactividad |

`bootstrap_owner` solo usa `gymName` si el equipo aún no tiene gimnasio.

```ts
interface RecoveryKit {
  session: SessionSummary;
  recoveryCode: string;
}

interface RecoveryCode {
  recoveryCode: string;
}
```

`RecoveryKit` es la única respuesta que contiene el código en claro. Se muestra
una sola vez; la base guarda únicamente su hash Argon2id.

## Reglas

- Nombre de gimnasio: 1–120 caracteres visibles tras `trim`.
- Nombre de persona: 2–80 caracteres; único por gimnasio sin distinguir
  mayúsculas.
- Contraseña del dueño: mínimo 12 caracteres. El resto del personal: mínimo 8.
- El código de recuperación usa 20 símbolos en cuatro grupos de cinco
  (`XXXXX-XXXXX-XXXXX-XXXXX`), sin caracteres ambiguos. No distingue mayúsculas
  y acepta espacios o guiones.
- Cada código sirve una sola vez. Generar o usar uno nuevo invalida el anterior;
  el cambio es condicional al hash verificado, así que dos usos simultáneos no
  pueden consumir el mismo código.
- Cinco fallos por nombre o por recuperación bloquean nuevos intentos durante
  cinco minutos.
- La sesión expira tras 15 minutos sin actividad.
- Roles: `owner`, `manager`, `receptionist`, `advisor`, `trainer`.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Datos inválidos, contraseña débil, equipo ya configurado o dueño existente |
| `unauthorized` | Credenciales o código inválidos, cuenta bloqueada o sesión ausente/expirada |
| `notFound` | No existe una cuenta de dueño que recuperar |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

Los mensajes de credenciales inválidas no distinguen si el nombre existe.

## Fuera de alcance

MFA, OIDC, recuperación por correo, reset de contraseñas de empleados,
múltiples dueños, sesiones persistentes entre
reinicios. La recuperación del dueño no depende de la red: exige el código
entregado al crear la cuenta o uno generado después desde **Colaboradores**. Si se
pierden contraseña y código, no hay puerta trasera; solo queda restaurar un
respaldo anterior o reiniciar una instalación de desarrollo.
