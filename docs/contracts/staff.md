# Contrato local · colaboradores (Fase 4.1)

- **Versión de contrato:** 3.0.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v21 (`0021_gym_operations.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_staff` | — | `StaffSummary[]` | `viewStaff` |
| `create_staff` | `name`, `role`, `password` | `StaffSummary` | `manageStaff` |
| `update_staff` | `userId`, `role`, `status` | `StaffSummary` | `manageStaff` |
| `update_staff_permissions` | `userId`, `overrides[]` | `StaffSummary` | `managePermissions` |

Una instalación representa un solo gimnasio. No hay comandos ni DTOs de sedes.

## DTOs

```ts
type StaffRole = "owner" | "manager" | "receptionist" | "advisor" | "trainer";
type StaffStatus = "active" | "revoked";
type PermissionEffect = "allow" | "deny";

interface PermissionOverride {
  permission: Permission;
  effect: PermissionEffect;
}

interface UpdateStaffPermissionsInput {
  userId: string;
  overrides: PermissionOverride[];
}

interface StaffSummary {
  userId: string;
  name: string;
  role: StaffRole;
  status: StaffStatus;
  lastLoginAt: string | null;
  createdAt: string;
  permissionOverrides: PermissionOverride[];
}
```

`Permission` se define en `packages/contracts` y coincide con los 39 permisos de
`crates/gym-core/src/rbac.rs`.

## Reglas de colaboradores

- Roles creables: `manager`, `receptionist`, `advisor` y `trainer`. `owner` solo
  nace en la configuración inicial.
- El Dueño puede crear cualquier rol operativo. Un Gerente solo puede crear y
  administrar `receptionist`, `advisor` y `trainer`.
- Contraseña temporal: mínimo 8 caracteres; se guarda solo como Argon2id.
- Nombre único por gimnasio sin distinguir mayúsculas; 2–80 caracteres.
- Una sola membresía por usuario.
- El dueño no se modifica desde esta pantalla y nadie puede editar su propia
  membresía ni sus propios permisos.
- Desactivar (`revoked`) impide el siguiente login.
- Cambiar rol aplica en el siguiente inicio de sesión; la sesión abierta conserva
  los permisos con los que fue creada hasta expirar o cerrar sesión.
- Los overrides `allow`/`deny` se guardan por usuario y se aplican al iniciar
  sesión. La UI muestra tres estados por permiso: heredar, permitir o denegar.
- `managePermissions` y `manageBackups` son solo del Dueño; el backend rechaza
  cualquier intento de concederlos mediante override.
- La UI de **Colaboradores** requiere `viewStaff`; las acciones de escritura
  requieren `manageStaff` y el editor de permisos requiere `managePermissions`.

## Migración `v20`

- `cashier` se convierte en `receptionist`.
- Las cuentas con rol `accounting` quedan con `users.status = 'disabled'`; el
  Dueño debe reasignarlas.
- Se recrea `staff_memberships` porque el `CHECK` anterior no admite los roles
  nuevos.
- Se crea `staff_permission_overrides` con clave primaria
  `(organization_id, user_id, permission)` y `effect IN ('allow', 'deny')`.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Nombre inválido, duplicado, rol no permitido, permiso desconocido, permiso solo del Dueño o intento de modificar al dueño |
| `unauthorized` | Sesión ausente, permiso insuficiente o rol fuera del alcance del Gerente |
| `notFound` | El colaborador no existe en el gimnasio |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

Se registran en `audit_events` con el actor de la sesión:

- `staff.created`
- `staff.updated`
- `staff.permissions_updated`

No se guardan contraseñas, hashes ni datos innecesarios en la auditoría.

## Fuera de alcance

Borrado físico de personas, reset de contraseñas de colaboradores, múltiples
dueños y edición del perfil propio. La recuperación del dueño se documenta en
[`auth-local.md`](auth-local.md); no se ejecuta desde el módulo de colaboradores.
