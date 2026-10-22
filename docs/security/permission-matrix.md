# Matriz de permisos V1

Fuente de verdad en código: `crates/gym-core/src/rbac.rs`.
Permisos y roles se serializan en camelCase; la UI solo los usa para ocultar
acciones. **El backend siempre revalida.**

## Roles

| Rol | Uso previsto |
| --- | --- |
| `owner` | Dueño del gimnasio; configuración, personal, finanzas y backups |
| `manager` | Encargado; operación, personal, inventario, catálogo y backups |
| `cashier` | Mostrador; cobros y ajustes operativos de inventario |
| `accounting` | Contabilidad; consulta de finanzas y auditoría, sin operar caja |

## Matriz

| Permiso | Owner | Manager | Cashier | Accounting |
| --- | :---: | :---: | :---: | :---: |
| `ManageSettings` | ✅ | — | — | — |
| `ManageStaff` | ✅ | ✅ | — | — |
| `ViewFinance` | ✅ | ✅ | — | ✅ |
| `OperatePos` | ✅ | ✅ | ✅ | — |
| `ManageInventory` | ✅ | ✅ | ✅ | — |
| `ManageCatalog` | ✅ | ✅ | — | — |
| `ManageBackups` | ✅ | ✅ | — | — |
| `ViewAudit` | ✅ | ✅ | — | ✅ |

Regla: **denegado por defecto**. Un permiso ausente se rechaza con
`CoreError::Forbidden`.

## Reglas de alcance

1. **Organización:** toda consulta y escritura filtra por `organization_id`. Un
   usuario no puede cruzar organizaciones aunque conozca un UUID.
2. **Membresía activa:** `status = 'active'`; una membresía `revoked` no puede
   iniciar una sesión nueva. Una sesión ya abierta conserva sus permisos hasta
   expirar por inactividad o cerrarse.
3. **Rol actual:** para editar personal, el comando valida tanto el rol solicitado
   como el rol actual del empleado; un `manager` no puede administrar a otro
   `manager` ni al `owner`.
4. **Recurso:** además del permiso, el comando valida que el recurso pertenezca a
   la organización del actor antes de escribir.
5. **UI no confiable:** ocultar un botón no es un control. Los comandos Tauri
   reciben el actor desde la sesión en Rust, nunca desde un `userId` del webview.

## Acciones sensibles

| Acción | Permiso | Reglas adicionales |
| --- | --- | --- |
| Ver reportes financieros | `ViewFinance` | Solo lectura; exportaciones auditadas |
| Cobrar, cancelar o reembolsar | `OperatePos` | Reembolso/cancelación con motivo y auditoría |
| Ajustar inventario | `ManageInventory` | Motivo obligatorio; auditar diferencia y actor |
| Crear o modificar catálogo | `ManageCatalog` | Precios append-only; editar un precio crea una versión nueva |
| Crear/editar personal | `ManageStaff` | No se puede crear ni elevar a `owner`; el dueño solo nace en la configuración inicial |
| Crear/restaurar backup | `ManageBackups` | Confirmación explícita; registrar archivo y resultado |
| Ver auditoría | `ViewAudit` | Solo lectura; no se puede editar ni borrar desde la UI |
| Cambiar configuración | `ManageSettings` | Incluye moneda, zona horaria y política de sesión |

## Auditoría mínima

Cada comando sensible debe registrar en `audit_events`:

- `actor_id`, `organization_id`.
- `action` estable (por ejemplo `backup.restore`), `resource` y resultado
  (`allowed`, `denied`, `failed`).
- `correlation_id` para agrupar una operación.
- Nunca incluir contraseñas, claves, passphrases ni PII innecesaria.

## Pruebas esperadas

- Matriz completa por rol (unitarias en `gym-core`).
- Denegación de comando sensible con membresía revocada.
- Denegación de acceso a recurso de otra organización.
- Registro de denegaciones con actor y correlación.

## Estado de implementación

- Aplicado en auth, personal, acceso, backups y catálogo: permisos, membresía
  activa, validación de recurso y auditoría append-only.
- `ViewAudit` solo permite consultar y verificar la cadena; no existe comando de
  edición o borrado.
- Pendiente al construir cada módulo: recepción, finanzas, inventario y
  exportaciones.

## Pendientes de endurecimiento (Phase 5)

- Definir permisos de socios, entrenadores y marketing en la misma versión desktop.
- Evaluar separar `ManageInventory` en consulta vs ajuste.
- Añadir aprobación de dos personas para cambios de configuración críticos.
- Exportación inmutable o copia externa de auditoría (hash chain ya implementada).
