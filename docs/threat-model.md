# Threat model · Fitness Gym desktop

- **Versión:** 2.0 (Phase 5)
- **Fecha:** 2026-10-20
- **Última actualización:** 2026-10-29
- **Alcance:** app Tauri, core Rust, SQLite cifrada, backups locales, servidor
  de sincronización mTLS, instalador firmado con minisign y cadena de build.
  La sincronización multi-dispositivo está habilitada (servidor mTLS con CN=org),
  el módulo externo de acceso real y los pagos con terminal se modelan como
  fronteras futuras. Phase 5 incluye build firmado, updater anti-downgrade,
  retención de backups y aprobación dual de configuración crítica.
- **Metodología:** STRIDE sobre fronteras de confianza + casos de abuso.

## 1. Supuestos

- Un gimnasio por instalación, sin sedes múltiples.
- El sistema operativo está actualizado y la cuenta del equipo tiene contraseña.
- El disco usa cifrado del SO (recomendado, no obligatorio).
- No hay administrador local hostil; el dueño controla el equipo.
- El webview (WebKit/WebView2) está actualizado y no está comprometido.
- La app no recibe contenido remoto ni ejecuta plugins de terceros.
- La sincronización entre dispositivos está habilitada mediante un servidor
  mTLS que identifica al gimnasio por el CN del certificado cliente. El journal
  local (esquema v16) se usa con transporte real; existe deduplicación por
  `idempotency_key` en ambos lados.

## 2. Activos

| Activo | Sensibilidad | Dónde vive |
| --- | --- | --- |
| PII de socios y contactos | Alta | SQLite cifrada |
| Pagos, adeudos y finanzas | Alta | SQLite cifrada |
| Historial de acceso y aforo | Alta | SQLite cifrada |
| Inventario y ventas | Media | SQLite cifrada |
| Auditoría de acciones | Alta | SQLite cifrada |
| Clave de la base (256 bits) | Crítica | Keychain del SO |
| Passphrase de backups | Crítica | Solo en la cabeza/gestor del usuario |
| Backups `.gymbak` | Alta | Disco local, cifrados |
| Binarios e instaladores | Alta | Canal de distribución (firmados con minisign, Phase 5) |
| Disponibilidad del negocio | Alta | Equipo local |

### 2.1 Clasificación de datos y consentimiento

| Nivel | Datos | Tratamiento |
| --- | --- | --- |
| Crítico | Clave de base, passphrase de backups, secretos de firma | Keychain, memoria zeroizada, nunca en logs ni sidecars |
| Alto | Nombre, contacto, identificación, pagos, adeudos, accesos, salud futura | Cifrado, mínimo privilegio, auditoría, minimización |
| Medio | Inventario, ventas agregadas, configuración del gimnasio | Cifrado, roles operativos, sin exposición pública |
| Bajo | Catálogo, textos de UI, preferencias no sensibles | Puede vivir en la base; no se usa para perfilado |

Principios de consentimiento:

- Aviso de privacidad y finalidad al alta del socio; sin telemetría ni
  phone-home obligatorio.
- Datos de salud, biometría o menores **no** se recolectan en Phase 1. Requieren
  consentimiento explícito, base legal y evaluación separada.
- Retención mientras dure la relación y los plazos legales; el borrado o la
  anonimización se auditan.
- Exportaciones y consultas masivas pasan por permisos y quedan auditadas.
- La revisión legal (LFPDPPP y contratos) está pendiente y no se sustituye por
  este documento.

## 3. Actores

| Actor | Confianza | Objetivo |
| --- | --- | --- |
| Owner | Alta | Operar y administrar el gimnasio, permisos y backups |
| Manager | Media-alta | Supervisar operación, personal y configuración |
| Receptionist | Media | Cobrar, atender socios, accesos y reservas |
| Advisor | Media | Seguimiento comercial y marketing |
| Trainer | Media | Socios asignados y clases |
| Ladrón de equipo | Ninguna | Extraer PII o extorsionar |
| Malware local | Ninguna | Robar credenciales, claves o datos |
| Contenido malicioso en el webview | Ninguna | Escalar a comandos Rust |
| Insider descontento | Media | Alterar historial o finanzas |
| Dependencia comprometida | Baja | Ejecutar código en build o runtime |
| Atacante de red | Ninguna | Interceptar sync/updates (futuro) |

## 4. Fronteras de confianza

```text
[Persona]
   │ UI
   ▼
[WebView renderer] ──IPC allowlist──▶ [Comandos Rust]
                                          │
                    ┌─────────────────────┼─────────────────────┐
                    ▼                     ▼                     ▼
             [SQLCipher DB+WAL]     [Backups .gymbak]     [Keychain SO]
                    ▲                     ▲
                    │                     │
             [Filesystem local] ──────────┘
                    ▲
                    │
          [Usuario del SO / malware]

[CI / build] ──artefacto firmado (Phase 5)──▶ [Instalador]
```

Fronteras:

1. Persona ↔ UI: autenticación y bloqueo de sesión.
2. WebView ↔ Rust: IPC con allowlist, validación y autorización.
3. Rust ↔ disco: cifrado, integridad y permisos de archivo.
4. Rust ↔ keychain: secreto fuera del árbol de datos.
5. App ↔ backups: sobre cifrado y verificación antes de restaurar.
6. Build ↔ equipo: cadena de suministro y firma (Phase 5).

## 5. STRIDE

### S — Spoofing

| Amenaza | Control | Estado |
| --- | --- | --- |
| Suplantar a otro empleado en el login local | Contraseña Argon2id, bloqueo tras 5 intentos, auditoría encadenada | Implementado |
| Robar o adivinar el código de recuperación | 20 símbolos aleatorios, hash Argon2id, un solo uso, rotación, bloqueo y auditoría | Implementado |
| Sesión abierta en mostrador | Bloqueo por inactividad (15 min) y cierre de sesión manual | Implementado |
| Backup restaurado para hacerse pasar por otro usuario | Restore requiere `ManageBackups`; auditoría `backup.restored`/`backup.restore_failed` con actor y archivo; aviso de reemplazo total | Implementado |
| PIN compartido | PIN no será factor único | Decidido |

### T — Tampering

| Amenaza | Control | Estado |
| --- | --- | --- |
| Editar páginas de la base a mano | SQLCipher HMAC-SHA512 + `PRAGMA integrity_check` | Implementado |
| Inyectar SQL desde la UI | Consultas parametrizadas; comandos con tipos | Implementado |
| Reemplazar la base por una copia vieja válida | Checksum de migraciones y detección de drift; anti-rollback global por updater minisign (versión mínima, sin downgrade) | Implementado |
| Alterar backups | SHA-256 del sobre en sidecar; passphrase para descifrar | Implementado |
| Manipular `-wal`/`-shm` durante restore | Cierre, checkpoint y limpieza antes del swap | Implementado |
| Cambiar código del frontend | CSP, assets locales y binario empaquetado | Implementado |
| Modificar logs de auditoría | Triggers append-only + hash chain SHA-256; verificación desde la UI | Implementado; truncado del final requiere copia externa |

### R — Repudiation

| Amenaza | Control | Estado |
| --- | --- | --- |
| Negar una venta, descuento o ajuste | `audit_events` con actor, acción, recurso y correlación | Implementado para auth, personal, acceso, backups, catálogo, ventas, descuentos y ajustes de inventario |
| Cuenta compartida | Identidad por persona y bloqueo de sesión | Decidido |
| Restore sin responsable | Registrar actor, backup y resultado (`backup.restored` / `backup.restore_failed`) | Implementado |

### I — Information disclosure

| Amenaza | Control | Estado |
| --- | --- | --- |
| Robo del equipo o del disco | SQLCipher AES-256 + clave en keychain | Implementado; keychain del SO |
| Código de recuperación visible en pantalla o archivo | Se muestra una vez; queda a cargo del dueño guardarlo fuera del equipo | Aceptado |
| Backup filtrado | XChaCha20-Poly1305 + Argon2id; passphrase no almacenada | Implementado |
| Clave en logs o errores | Sin `Debug` de claves; mensajes genéricos | Implementado |
| Clave junto a la base | Keychain; archivo `0600` solo fallback dev/CI | Implementado |
| Datos sensibles en `localStorage`/caché web | Sin persistencia sensible en el frontend; CSP | Implementado, revisar por pantalla |
| Capturas o shoulder surfing | No se controla; mitigación física | Aceptado |

### D — Denial of service

| Amenaza | Control | Estado |
| --- | --- | --- |
| Disco lleno durante venta o backup | Backup previo, escritura atómica, monitoreo de espacio | Parcial: venta en transacción; monitoreo pendiente |
| Base corrupta | `integrity_check`, backups verificados, runbook de restore | Implementado |
| Bloqueo por archivo abierto | Cierre y checkpoint antes de restore | Implementado |
| Borrado accidental de backups | Confirmación y retención configurable (15 más recientes, >30 días) | Implementado |

### E — Elevation of privilege

| Amenaza | Control | Estado |
| --- | --- | --- |
| Cajero invoca comandos de finanzas/backups | Autorización obligatoria en Rust, denegado por defecto | Implementado en los comandos actuales |
| XSS en el webview invoca IPC arbitrario | CSP estricta, sin contenido remoto, allowlist de comandos | Parcial |
| Acceso a shell/filesystem desde la UI | Sin plugins shell/fs; solo comandos propios | Implementado |
| Cruce de organización | Filtro por `organization_id` en cada consulta | Implementado en auth, personal, catálogo, ventas y caja |
| Encargado edita a otro encargado o al dueño | Roles permitidos y rol actual validados en Rust | Implementado |
| Path traversal en restore | Se exige nombre de archivo simple dentro de `backups/` | Implementado |

### Supply chain y actualizaciones

| Amenaza | Control | Estado |
| --- | --- | --- |
| Dependencia npm/cargo maliciosa | Lockfiles, versiones fijas, revisión, `cargo deny`/`pnpm audit` | Parcial |
| Toolchain alterado | `rust-toolchain.toml` fijo y CI reproducible | Implementado |
| Instalador modificado | Firma minisign y updater verificados | Implementado |
| Downgrade a versión vulnerable | Updater sin downgrade, versión mínima y `createUpdaterArtifacts` | Implementado |

## 6. Casos de abuso

1. **Cajero curioso:** intenta abrir finanzas o backups desde DevTools. El
   comando Rust debe denegar por permiso y registrar el intento.
2. **Manager restaura un backup antiguo:** puede hacerlo con su permiso, pero el
   sistema registra actor, archivo y hora; el restore es total y visible.
3. **Ladrón con el equipo apagado:** obtiene el archivo SQLCipher y el keychain
   protegido por la cuenta del SO; sin la sesión desbloqueada no obtiene la clave.
4. **Malware en la sesión del dueño:** puede actuar como el dueño. El cifrado no
   protege una sesión comprometida; se mitiga con EDR, updates y backups.
5. **Backup en una USB perdida:** sin la passphrase no se descifra; la fortaleza
   depende de la passphrase elegida.
6. **Webview comprometido:** solo puede llamar comandos allowlisted; la
   autorización se revalida en Rust y no hay shell.
7. **Insider borra ventas:** `sales` y `sale_items` rechazan `UPDATE` y
   `DELETE` por trigger; una corrección futura exigirá un documento
   compensatorio auditado.
8. **Cajero oculta un faltante:** el cierre compara el efectivo contado con el
   esperado, exige un motivo cuando hay diferencia y guarda actor, hora y motivo.
   `cash_sessions` no admite borrado ni reapertura; los ingresos y gastos son
   append-only. Un faltante requiere revisión y queda visible en el historial.

## 6.1 Integraciones futuras modeladas

Estas fronteras no existen en Phase 1, pero condicionan el diseño y deben tener
su propio ADR antes de implementarse.

### Módulo externo de acceso (torniquete, lector, biometría)

| Amenaza | Control requerido | Fase |
| --- | --- | --- |
| Dispositivo suplantado | Pairing con clave única y autenticación mutua | 2+ |
| Repetición de eventos de acceso | Nonce, timestamp y deduplicación por `idempotency_key` | 2+ |
| Alteración del veredicto de acceso | Firma de mensajes y verificación en ambos lados | 2+ |
| Caída del módulo | Política fail-open/fail-closed explícita y auditoría | 2+ |
| Biometría expuesta | La plantilla vive en el dispositivo; la app solo guarda el evento | 2+ |
| Red local hostil | Segmentación, TLS o canal firmado, sin exponer la base | 2+ |

### Pagos y terminal Point

| Amenaza | Control requerido | Fase |
| --- | --- | --- |
| Card data en la app | Nunca almacenar PAN/CVV; solo referencias opacas del proveedor | 3 |
| Cobro duplicado | Idempotency key y conciliación con el proveedor | 3 |
| Reembolso no autorizado | Permiso, motivo y auditoría; límite por rol | 3 |
| Terminal falsa | Pairing y verificación de identidad del dispositivo | 3 |
| Descuadre de caja | Conciliación diaria y reporte de diferencias | 3 |

### Sincronización multi-dispositivo

El journal local ya implementa outbox transaccional, idempotencia por gimnasio,
inbox deduplicado por `event_id` y cursor por secuencia. El transporte real
sigue pendiente.

| Amenaza | Control requerido | Estado |
| --- | --- | --- |
| Relay lee datos | mTLS mutuo (CN=org); el servidor solo ve sobres firmados | Implementado (servidor mTLS con CN=org) |
| Replay o duplicados | Log idempotente por organización y dispositivo | Implementado en ambos lados (`idempotency_key`) |
| Conflicto de edición | Política por agregado y resolución explícita | Pendiente; arquitectura preparada |
| Dispositivo robado sigue sincronizando | Revocación de certificado de dispositivo y rotación | Pendiente; arquitectura preparada (CA por org) |
| Downgrade de reglas | Versión mínima de contrato y migración firmada | Implementado (updater minisign con versión mínima) |

## 7. Riesgos residuales

| Riesgo | Nivel | Mitigación / decisión |
| --- | --- | --- |
| Sesión desbloqueada comprometida | Alto | Aceptado; EDR, bloqueo por inactividad, backups |
| Rollback a una base válida anterior | Bajo | Updater minisign impide downgrade; anti-rollback global aplicado |
| Truncado del final de la cadena de auditoría | Bajo | Exportación inmutable con checksum SHA-256 y archivo .sha256; anclaje monotónico planificado |
| Pérdida de passphrase de backups | Alto | Aceptado; educación y prueba trimestral de restore |
| Pérdida de contraseña y código de recuperación del dueño | Alto | Sin puerta trasera; restaurar un backup o reinstalar en desarrollo |
| Dependencia comprometida | Medio | Lockfiles, auditorías y builds firmados |
| Keychain no disponible | Bajo | Fallback dev con advertencia; no se crea base en claro |
| Sin anti-rollback de binarios hasta Phase 5 | Bajo | Updater minisign con versión mínima; `createUpdaterArtifacts` habilitado |

## 8. Fuera de alcance

- Coerción física o violencia.
- Administrador del SO o malware con privilegios de kernel.
- Ataques de canal lateral sobre el hardware.
- Servicio cloud multi-tenant y transporte de sincronización (se evaluará con su
  propio threat model antes de habilitarlo). El journal local no cruza la red.
- Cumplimiento legal específico (LFPDPPP, PCI); se abordará con asesoría.

## 9. Requisitos verificables de Phase 1

- [x] La base no es legible sin la clave (prueba de cabecera y texto plano).
- [x] Clave incorrecta rechazada.
- [x] Migraciones reproducibles y drift detectado.
- [x] Backup sin texto plano; passphrase incorrecta rechazada.
- [x] Restore verifica checksum e integridad y funciona con la base cerrada.
- [x] CSP y capability mínima configuradas; pruebas estáticas validan la allowlist y la ausencia de web storage.
- [x] Clave migrada al keychain del SO (2026-10-21).
- [x] Login local, sesión y RBAC conectados a auth y personal.
- [x] Auditoría append-only con hash chain y verificación.
- [x] RBAC y auditoría en catálogo y ventas.
- [ ] Extender RBAC y auditoría a recepción, finanzas e inventario.
- [x] Vertical slice de acceso simulado con deduplicación por `idempotency_key`.
- [x] Backup/restore desde la UI con IPC simulado; el binario nativo se valida
      manualmente en Linux.
- [x] Revisión de dependencias y build firmado (firma minisign, Phase 5).

## 10. Revisión

Revisar este documento al cerrar Phase 1, antes de la sincronización y ante
cualquier incidente de seguridad. Cada cambio de control debe reflejarse en un
ADR o en este registro.
