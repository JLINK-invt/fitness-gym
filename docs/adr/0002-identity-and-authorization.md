# ADR-0002 · Identidad local y autorización

- **Estado:** Accepted (2026-10-20)
- **Fecha:** 2026-10-20
- **Decide:** Identidad, sesión y permisos
- **Relacionado:** Phase 1, Phase 2

## Contexto

Recepción necesita operar rápido y cambiar de usuario durante el turno. No hay
proveedor de identidad ni correo garantizado. El daño más plausible es interno:
un cajero que intenta ver finanzas, un manager que restaura un backup para
alterar historial o un ex empleado con acceso al equipo.

## Decisión

1. **Identidad local por organización.** Cada usuario tiene `user_id` UUID,
   nombre, rol y alcance de sede. No se requiere correo ni nube.
2. **Contraseña local** con Argon2id y salt único. Valores iniciales: mínimo 8
   caracteres (12 para el owner) y bloqueo de 5 minutos tras 5 intentos
   fallidos, con auditoría del evento.
3. **Sesión en memoria** dentro del proceso Tauri. Sin tokens en el webview,
   `localStorage` ni cookies. Bloqueo automático por inactividad a los 15
   minutos (configurable por organización).
4. **Roles V1:** `owner`, `manager`, `cashier`, `accounting`. Un usuario puede
   tener una membresía por organización/sede; `site_id = NULL` significa
   alcance de toda la organización y solo se permite a `owner`.
5. **Autorización en Rust, no en la UI.** Cada comando valida permiso, estado de
   membresía, organización y sede antes de tocar datos. La UI solo oculta
   acciones; el backend siempre vuelve a decidir.
6. **Denegado por defecto.** Un permiso no listado se rechaza. Toda denegación
   sensible se registra en `audit_events` con actor, recurso y correlación.
7. **PIN rápido:** aplazado. Si se implementa, será un desbloqueo local sobre
   una sesión ya autenticada, nunca el único factor, con rate limit y auditoría.
8. **Bootstrap:** en el primer arranque se crea el `owner` con contraseña. No
   existe usuario por defecto ni contraseña universal.
9. **MFA y recuperación del dueño:** diferidos a Phase 5 (endurecimiento). No
   bloquean la operación V1 porque el equipo es local, de un solo gimnasio, y el
   dueño custodia su contraseña. Si se implementan, MFA será adicional (nunca
   sustituto) y la recuperación exigirá presencia física más un factor de
   recuperación auditable; no habrá reset por correo.

## Consecuencias

- El equipo debe custodiar la contraseña del owner; no hay "recuperar cuenta"
  por correo ni reset automático. Perderla exige intervención de soporte con
  acceso físico y un procedimiento auditado que se definirá en Phase 5.
- Cambiar de cajero requiere login o bloqueo/desbloqueo; se medirá el costo en
  el vertical slice antes de añadir PIN.
- La matriz de permisos es un contrato verificable en pruebas unitarias.
- La auditoría actual no es a prueba de manipulación local; ver threat model.

## Alternativas descartadas

- **Solo cuenta del sistema operativo:** no distingue roles ni deja auditoría
  por persona.
- **OAuth/cloud identity:** rompe la operación offline y añade dependencia.
- **PIN como único factor:** demasiado débil para finanzas y backups.
- **Permisos en el frontend:** evitable con DevTools o un webview comprometido.

## Criterios de revisión

- Al cerrar el piloto, con métricas de tiempo de login y cambios de turno.
- Antes de añadir roles de entrenador, marketing y socio.
- Si se solicita SSO, directorio corporativo o gestión remota de usuarios.
- Tras una auditoría de seguridad o un incidente de acceso interno.

## Aprobación

Aprobado el 2026-10-20 con los valores iniciales de contraseña (8/12) y bloqueo
por inactividad (15 min). Se revisan con el primer gimnasio piloto; cualquier
cambio requiere un ADR nuevo.
