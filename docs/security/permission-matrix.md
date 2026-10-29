# Matriz de permisos V1

Fuente de verdad en código: `crates/gym-core/src/rbac.rs`.
Permisos y roles se serializan en camelCase; la UI solo los usa para ocultar
acciones. **El backend siempre revalida.**

## Roles

| Rol | Uso previsto |
| --- | --- |
| `owner` | Dueño del gimnasio; control total, permisos y backups |
| `manager` | Gerente; supervisa operación, personal, configuración y reportes |
| `receptionist` | Recepción; socios, accesos, cobros, caja, reservas y prospectos |
| `advisor` | Asesor; seguimiento comercial, socios, reservas y marketing |
| `trainer` | Entrenador; socios asignados, clases y reportes de mantenimiento |

`accountant` y `cashier` ya no existen. La migración `v20` convierte
`cashier` en `receptionist` y desactiva las cuentas `accounting` para que el
Dueño las reasigne.

## Matriz

| Permiso | Owner | Manager | Receptionist | Advisor | Trainer |
| --- | :---: | :---: | :---: | :---: | :---: |
| `viewMembers` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `manageMembers` | ✅ | ✅ | ✅ | ✅ | — |
| `viewAssignedMembers` | ✅ | — | — | — | ✅ |
| `viewAccess` | ✅ | ✅ | ✅ | — | — |
| `operateAccess` | ✅ | ✅ | ✅ | — | — |
| `manageAccessSnapshots` | ✅ | ✅ | — | — | — |
| `viewSales` | ✅ | ✅ | ✅ | ✅ | — |
| `operatePos` | ✅ | ✅ | ✅ | — | — |
| `superviseCash` | ✅ | ✅ | — | — | — |
| `viewClasses` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `manageClasses` | ✅ | ✅ | — | — | ✅ |
| `manageReservations` | ✅ | ✅ | ✅ | ✅ | — |
| `viewMarketing` | ✅ | ✅ | ✅ | ✅ | — |
| `manageProspects` | ✅ | ✅ | ✅ | ✅ | — |
| `manageMarketing` | ✅ | ✅ | — | ✅ | — |
| `viewInventory` | ✅ | ✅ | ✅ | — | — |
| `manageInventory` | ✅ | ✅ | ✅ | — | — |
| `viewCatalog` | ✅ | ✅ | ✅ | ✅ | — |
| `manageCatalog` | ✅ | ✅ | — | — | — |
| `viewFinance` | ✅ | ✅ | — | — | — |
| `manageFinance` | ✅ | — | — | — | — |
| `exportFinance` | ✅ | ✅ | — | — | — |
| `viewReports` | ✅ | ✅ | ✅ | ✅ | — |
| `viewCommercialReports` | ✅ | ✅ | — | ✅ | — |
| `viewCashReports` | ✅ | ✅ | ✅ | — | — |
| `exportReports` | ✅ | ✅ | — | — | — |
| `viewStaff` | ✅ | ✅ | — | — | — |
| `manageStaff` | ✅ | ✅ | — | — | — |
| `managePermissions` | ✅ | — | — | — | — |
| `viewSettings` | ✅ | ✅ | — | — | — |
| `manageOperationalSettings` | ✅ | ✅ | — | — | — |
| `manageSettings` | ✅ | ✅ | — | — | — |
| `viewAudit` | ✅ | ✅ | — | — | — |
| `manageBackups` | ✅ | — | — | — | — |
| `viewIntegrations` | ✅ | ✅ | — | — | — |
| `manageIntegrations` | ✅ | ✅ | — | — | — |
| `viewMaintenance` | ✅ | ✅ | ✅ | ✅ | ✅ |
| `manageMaintenance` | ✅ | ✅ | — | — | — |
| `reportMaintenanceIssue` | ✅ | ✅ | ✅ | ✅ | ✅ |

Regla: **denegado por defecto**. Un permiso ausente se rechaza con
`CoreError::Forbidden`.

## Overrides por usuario

- La plantilla del rol define el punto de partida.
- El Dueño puede conceder (`allow`) o denegar (`deny`) permisos concretos por
  colaborador desde **Colaboradores**.
- Los overrides viven en `staff_permission_overrides` y se aplican al iniciar
  sesión; la sesión abierta conserva los permisos con los que nació.
- `managePermissions` y `manageBackups` son **solo del Dueño**: no se pueden
  conceder a otro rol ni mediante override.
- El Dueño no puede quitarse permisos a sí mismo ni bloquear su propia cuenta.
- Un Gerente administra colaboradores, pero no permisos; esa capacidad queda
  reservada al Dueño.

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
| Ver reportes financieros | `viewFinance` | Solo lectura; exportaciones auditadas |
| Cobrar, cancelar o reembolsar | `operatePos` | Reembolso/cancelación con motivo y auditoría |
| Abrir o cerrar turno de caja | `operatePos` | Un solo turno abierto; cierre con efectivo contado y motivo obligatorio si hay diferencia |
| Supervisar caja sin operar | `superviseCash` | Consulta y respaldo operativo del Gerente |
| Registrar ingreso o gasto de caja | `operatePos` | Monto positivo, método, categoría y evidencia; movimiento append-only |
| Consultar historial de turnos | `viewFinance` | Solo lectura; incluye fondo, esperado, contado, diferencia, motivo y movimientos |
| Ajustar inventario | `manageInventory` | Motivo obligatorio; auditar diferencia y actor |
| Crear o modificar catálogo | `manageCatalog` | Precios append-only; editar un precio crea una versión nueva |
| Crear/editar personal | `manageStaff` | No se puede crear ni elevar a `owner`; el dueño solo nace en la configuración inicial |
| Editar permisos por usuario | `managePermissions` | Solo el Dueño; no puede modificar sus propios permisos |
| Crear/restaurar backup | `manageBackups` | Solo el Dueño; confirmación explícita y auditoría |
| Ver auditoría | `viewAudit` | Solo lectura; no se puede editar ni borrar desde la UI |
| Cambiar configuración | `manageSettings` | Incluye moneda, zona horaria y política de sesión |
| Generar código de recuperación del dueño | `manageSettings` | Solo el rol `owner`; invalida el código anterior y se audita |
| Administrar finanzas | `manageFinance` | Solo el Dueño; no se asigna automáticamente al Gerente |

## Auditoría mínima

Cada comando sensible debe registrar en `audit_events`:

- `actor_id`, `organization_id`.
- `action` estable (por ejemplo `backup.restore`), `resource` y resultado
  (`allowed`, `denied`, `failed`).
- `correlation_id` para agrupar una operación.
- Nunca incluir contraseñas, claves, passphrases ni PII innecesaria.

## Pruebas esperadas

- Matriz completa por rol (unitarias en `gym-core`).
- Overrides `allow`/`deny`, incluido el rechazo de permisos solo del Dueño.
- Denegación de comando sensible con membresía revocada.
- Denegación de acceso a recurso de otra organización.
- Registro de denegaciones con actor y correlación.

## Estado de implementación

- Fase 4.1: contrato `3.1.0`, esquema `v21`, cinco roles, overrides por usuario,
  menú agrupado y editor de permisos en **Colaboradores**.
- Aplicado en auth, personal, acceso, backups, catálogo, ventas, caja,
  inventario, reportes, exportaciones, configuración, complementos, clases,
  marketing, finanzas y mantenimiento.
- `viewAudit` solo permite consultar y verificar la cadena; no existe comando de
  edición o borrado.

## Pendientes de endurecimiento (Fase 5)

- Añadir aprobación de dos personas para cambios de configuración críticos.
- Exportación inmutable o copia externa de auditoría (hash chain ya implementada).
