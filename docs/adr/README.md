# Architecture Decision Records

Decisiones de la fundación desktop. Estado `Proposed` = lista para revisión;
`Accepted` = aprobada y vinculante.

| ADR | Decisión | Estado |
| --- | --- | --- |
| [0001](0001-local-first-architecture.md) | Arquitectura local-first con Tauri, Rust modular y SQLite | Accepted |
| [0002](0002-identity-and-authorization.md) | Identidad local, roles y autorización | Accepted |
| [0003](0003-encryption-and-key-custody.md) | SQLCipher y custodia de clave en keychain del SO | Accepted |
| [0004](0004-backups-and-restore.md) | Backups cifrados, verificación y restauración | Accepted |
| [0005](0005-immutable-audit.md) | Auditoría append-only con hash chain local | Accepted |
| [0006](0006-single-site-installation.md) | Una instalación, un gimnasio; sin sedes | Accepted |

Aprobados el 2026-10-20. El ADR-0002 fija valores iniciales de contraseña y
bloqueo, revisables con el primer piloto. El ADR-0004 difiere el destino cloud y
la retención a Phase 4.

## Cómo aprobar

1. Revisar las consecuencias y los riesgos residuales de cada ADR.
2. Confirmar las decisiones abiertas marcadas con **Decisión pendiente**.
3. Cambiar el estado a `Accepted` y registrar fecha y responsable.
4. Cualquier cambio posterior se hace con un ADR nuevo que supersede al anterior.
