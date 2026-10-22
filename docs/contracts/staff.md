# Contrato local · personal (Phase 2)

- **Versión de contrato:** 2.0.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v6 (`0006_single_site.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_staff` | — | `StaffSummary[]` | `ManageStaff` |
| `create_staff` | `name`, `role`, `password` | `StaffSummary` | `ManageStaff` |
| `update_staff` | `userId`, `role`, `status` | `StaffSummary` | `ManageStaff` |

Una instalación representa un solo gimnasio. No hay comandos ni DTOs de sedes.

## DTOs

```ts
type StaffStatus = "active" | "revoked";

interface StaffSummary {
  userId: string;
  name: string;
  role: "owner" | "manager" | "cashier" | "accounting";
  status: StaffStatus;
  lastLoginAt: string | null;
  createdAt: string;
}
```

## Reglas de personal

- Roles creables: `manager`, `cashier`, `accounting`. `owner` solo nace en la
  configuración inicial.
- Contraseña temporal: mínimo 8 caracteres; se guarda solo como Argon2id.
- Nombre único por gimnasio sin distinguir mayúsculas; 2–80 caracteres.
- Una sola membresía por usuario.
- El dueño no se modifica desde esta pantalla y nadie puede editar su propia
  membresía.
- Un encargado solo lista y administra `cashier` y `accounting`; no crea otros
  encargados, no edita encargados ni al dueño.
- Desactivar (`revoked`) impide el siguiente login.
- Cambiar rol aplica en el siguiente inicio de sesión; la sesión abierta conserva
  los permisos con los que fue creada hasta expirar o cerrar sesión.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Nombre inválido, duplicado, rol no permitido o intento de modificar al dueño |
| `unauthorized` | Sesión ausente, permiso insuficiente o rol fuera del alcance del encargado |
| `notFound` | El empleado no existe en el gimnasio |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

Se registran en `audit_events` con el actor de la sesión:

- `staff.created` y `staff.updated`.

No se guardan contraseñas, hashes ni datos innecesarios en la auditoría.

## Fuera de alcance

Borrado físico de personas, permisos personalizados, múltiples dueños y edición
del perfil propio. La recuperación del dueño se difiere a Phase 5
(endurecimiento).
