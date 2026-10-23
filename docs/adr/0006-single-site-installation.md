# ADR-0006 · Una instalación, un gimnasio

- **Estado:** Accepted
- **Fecha:** 2026-10-22
- **Supersede:** soporte multi-sede de ADR-0001 y alcance por sede de ADR-0002
- **Relacionado:** contrato 2.0.0, esquema v6

## Contexto

El producto se diseñó con organizaciones y sedes, pero ningún flujo operativo
usaba más de una sede. Mantener `sites` y `site_id` agregaba complejidad en
esquema, contratos, autorización, UI, backups y pruebas sin aportar valor al
gimnasio objetivo. La pantalla de sedes y el alcance por sede también dificultaban
razonar sobre accesos y auditoría.

## Decisión

1. Cada instalación representa **un solo gimnasio**. No existen `sites` ni
   `site_id` en el esquema.
2. `staff_memberships` guarda solo el rol. El alcance de un `manager` se define
   por roles: administra `cashier` y `accounting`, no a otros `manager` ni al
   `owner`.
3. Los contratos suben a 2.0.0 y eliminan `SiteStatus`, `SiteSummary` y los
   campos de sede de sesión, personal y auditoría. También desaparecen los
   comandos `list_sites`, `create_site` y `update_site`.
4. La auditoría deja de registrar `site_id`. Todos los roles con `ViewAudit` ven
   los eventos de la instalación.
5. La migración 6 elimina la tabla `sites`, reconstruye `staff_memberships`,
   elimina `audit_events.site_id`, limpia los hashes y reinicia la cadena con el
   formato `fitness-gym-audit-v2`.
6. La UI elimina la pantalla **Sedes**; **Equipo** administra personas y roles.

## Consecuencias

- Menos tablas, comandos, DTOs y reglas de autorización que mantener.
- La migración 6 es irreversible: no puede conservar la cadena anterior porque el
  contenido canónico cambió. Revertir exige restaurar un snapshot previo.
- Una instalación no puede separar operación, personal ni auditoría por sucursal.
  Si el producto necesita varias sucursales, se diseñará como instalaciones
  independientes o con un ADR nuevo.
- Los backups anteriores a v6 siguen siendo restaurables; al abrirse se aplica la
  migración 6.

## Alternativas descartadas

- **Ocultar la UI y conservar el esquema:** mantiene deuda y permite que llamadas
  IPC directas operen con sedes.
- **Dejar `site_id` opcional:** conserva ambigüedad en consultas, permisos y
  auditoría sin habilitar un flujo real.
- **Una base por sede:** agrega coordinación, backups y soporte antes de validar
  el producto.

## Criterios de revisión

- Si un cliente exige operar varias sucursales desde un mismo equipo.
- Antes de introducir sincronización multi-dispositivo o una consola central.
- Si cambia el modelo comercial de una instalación por gimnasio.
