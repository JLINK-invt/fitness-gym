# Contrato local · configuración, complementos, clases, marketing, finanzas y mantenimiento

- **Versión de contrato:** 3.2.0
- **Estado:** implementado
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red
- **Esquema:** v22 (`0022_critical_settings_approval.sql`)

## Comandos

| Comando | Entrada | Salida | Permiso |
| --- | --- | --- | --- |
| `get_gym_settings` | — | `GymSettings` | `ViewSettings` |
| `request_settings_change` | `UpdateGymSettingsInput` | `SettingsChangeRequest` | `ManageSettings` |
| `list_settings_change_requests` | — | `SettingsChangeRequest[]` | `ViewSettings` |
| `review_settings_change` | `ReviewSettingsChangeInput` | `SettingsChangeRequest` | `ManageSettings` |
| `list_integration_links` | — | `IntegrationLink[]` | `ViewIntegrations` |
| `upsert_integration_link` | `UpsertIntegrationLinkInput` | `IntegrationLink` | `ManageIntegrations` |
| `run_sync_now` | — | `SyncRunReport` | `ManageIntegrations` |
| `integration_journal_summary` | — | `IntegrationJournalSummary` | `ViewIntegrations` |
| `list_classes` | — | `ClassSession[]` | `ViewClasses` |
| `create_class` | `CreateClassInput` | `ClassSession` | `ManageClasses` |
| `update_class` | `UpdateClassInput` | `ClassSession` | `ManageClasses` |
| `list_class_bookings` | `classId` | `ClassBooking[]` | `ViewClasses` |
| `book_class` | `BookClassInput` | `ClassBooking` | `ManageReservations` |
| `update_class_booking` | `UpdateClassBookingInput` | `ClassBooking` | `ManageReservations` |
| `list_campaigns` | — | `Campaign[]` | `ViewMarketing` |
| `create_campaign` | `CreateCampaignInput` | `Campaign` | `ManageMarketing` |
| `update_campaign` | `UpdateCampaignInput` | `Campaign` | `ManageMarketing` |
| `marketing_funnel` | `inactivityDays?`, `expiryDays?` | `MarketingFunnel` | `ViewMarketing` |
| `list_equipment` | — | `Equipment[]` | `ViewMaintenance` |
| `create_equipment` | `CreateEquipmentInput` | `Equipment` | `ManageMaintenance` |
| `update_equipment` | `UpdateEquipmentInput` | `Equipment` | `ManageMaintenance` |
| `list_maintenance_orders` | `status?` | `MaintenanceOrder[]` | `ViewMaintenance` |
| `create_maintenance_order` | `CreateMaintenanceOrderInput` | `MaintenanceOrder` | `ManageMaintenance` |
| `update_maintenance_order` | `UpdateMaintenanceOrderInput` | `MaintenanceOrder` | `ManageMaintenance` |

## Reglas

- Configuración: nombre comercial obligatorio, moneda ISO de 3 letras y límites
  de días de inactividad/vencimiento; razón social y RFC son opcionales y solo
  acompañan la exportación. Los cambios críticos requieren **aprobación de dos
  personas**: el solicitante envía una solicitud (`request_settings_change`) y
  otra persona con `ManageSettings` la aprueba (`review_settings_change`). Todo
  cambio queda auditado y sincronizado. Pendientes máximos: 1 por gimnasio.
- Complementos: `module` único por gimnasio; capacidades separadas por coma,
  `pending` hasta el primer evento de sincronización. Botón **Sincronizar ahora**
  (permiso `ManageIntegrations`) ejecuta `run_sync_now` y muestra reporte de
  push/pull.
- Clases: cupo mínimo de 1; no se puede reducir por debajo de las reservas
  activas; solo clases activas aceptan reservas y un socio no puede duplicarlas.
- Reservas: estados `booked`, `attended`, `cancelled`, `no_show`; la reactivación
  de una reserva cancelada vuelve a contar para el cupo.
- Marketing: embudo sobre prospectos y socios; el resumen usa la configuración
  del gimnasio para inactividad y vencimiento, con valores por defecto de 30 días.
- Finanzas: reutiliza `financial_report` y exporta CSV por el flujo de reportes
  existente, con razón social y RFC como metadata; no crea asientos ni calcula
  impuestos. Exportaciones separadas: `export_financial_report_csv` (permiso
  `ExportFinance`) y `export_report_csv` (permiso `ExportReports`).
- Mantenimiento: órdenes `open`, `in_progress`, `completed`, `cancelled`;
  equipos `operational`, `maintenance`, `out_of_service`.

## Persistencia

- `gym_settings`, `settings_change_requests`, `integration_links`, `classes`,
  `class_bookings`, `campaigns`, `equipment` y `maintenance_orders` por
  `organization_id`.
- Escrituras transaccionales con `sync_outbox` (`settings.updated`,
  `integration.linked`, `class.*`, `class.booking.*`, `campaign.*`,
  `equipment.*`, `maintenance.order.*`, `settings.change_approved`,
  `settings.change_rejected`, `sync.completed`, `sync.settings_saved`).
