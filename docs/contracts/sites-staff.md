# Contrato local · sedes y personal (Phase 2)

- **Versión de contrato:** 1.3.0 (aditiva sobre 1.2.0)
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v3 (`0003_sites_staff.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `list_sites` | — | `SiteSummary[]` | `ManageStaff` |
| `create_site` | `name` | `SiteSummary` | `ManageSettings` |
| `update_site` | `siteId`, `name`, `status` | `SiteSummary` | `ManageSettings` |
| `list_staff` | — | `StaffSummary[]` | `ManageStaff` |
| `create_staff` | `name`, `role`, `siteId`, `password` | `StaffSummary` | `ManageStaff` |
| `update_staff` | `userId`, `role`, `siteId`, `status` | `StaffSummary` | `ManageStaff` |

`list_sites` usa `ManageStaff` porque la pantalla **Equipo** necesita las sedes
para asignar personal. Un encargado solo recibe su propia sede.

## DTOs

```ts
type SiteStatus = "active" | "closed";
type StaffStatus = "active" | "revoked";

interface SiteSummary {
  id: string;
  name: string;
  status: SiteStatus;
  createdAt: string;
}

interface StaffSummary {
  userId: string;
  name: string;
  role: "owner" | "manager" | "cashier" | "accounting";
  siteId: string | null;
  siteName: string | null;
  status: StaffStatus;
  lastLoginAt: string | null;
  createdAt: string;
}
```

## Reglas de sedes

- Nombre: 2–80 caracteres visibles tras `trim`, sin caracteres de control.
- Único por gimnasio sin distinguir mayúsculas.
- Una sede se cierra; no se borra. Cerrar exige cero personal activo asignado.
- Una sede cerrada no acepta personal nuevo ni reactivaciones.
- Renombrar o reabrir una sede conserva su `id` y su historial.
- El dueño ve todas; un encargado solo la suya.

## Reglas de personal

- Roles creables: `manager`, `cashier`, `accounting`. `owner` solo nace en la
  configuración inicial.
- Contraseña temporal: mínimo 8 caracteres; se guarda solo como Argon2id.
- Nombre único por gimnasio sin distinguir mayúsculas; 2–80 caracteres.
- Una sola membresía por usuario.
- `siteId` es obligatorio para todo el personal; el dueño usa `null`.
- El dueño no se modifica desde esta pantalla y nadie puede editar su propia
  membresía.
- Un encargado solo administra `cashier` y `accounting` de su sede; no crea otros
  encargados, no mueve personal a otra sede y no ve otras sedes.
- Desactivar (`revoked`) impide el siguiente login. Reactivar exige una sede
  activa.
- Cambiar rol o sede aplica en el siguiente comando; la sesión abierta conserva
  los permisos con los que fue creada hasta expirar o cerrar sesión.

## Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Nombre inválido, duplicado, sede cerrada, cierre con personal activo, rol no permitido o intento de modificar al dueño |
| `unauthorized` | Sesión ausente, permiso insuficiente o alcance de sede inválido |
| `notFound` | La sede o el empleado no existe en el gimnasio |
| `storage` | Fallo de SQLCipher o de persistencia |
| `internal` | Error inesperado del core |

## Auditoría

Se registran en `audit_events` con el actor de la sesión:

- `site.created` y `site.updated`.
- `staff.created` y `staff.updated`.

No se guardan contraseñas, hashes ni datos innecesarios en la auditoría.

## Fuera de alcance

Borrado físico de sedes o personas, transferencia de membresías entre
instalaciones, permisos personalizados, múltiples dueños y edición del perfil
propio. La recuperación del dueño requiere una decisión posterior.
