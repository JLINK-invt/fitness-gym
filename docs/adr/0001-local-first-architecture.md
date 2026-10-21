# ADR-0001 · Arquitectura local-first

- **Estado:** Accepted (2026-10-20)
- **Fecha:** 2026-10-20
- **Decide:** Arquitectura de la aplicación
- **Relacionado:** Phase 1

## Contexto

El producto se vende a gimnasios independientes con conectividad irregular. La
operación de recepción, cobros, accesos y reportes no puede depender de la nube.
El equipo es pequeño y el costo operativo debe ser cercano a cero. A la vez, los
datos incluyen PII de socios, pagos e historial de acceso, por lo que el disco
robado o el backup filtrado son riesgos reales.

## Decisión

1. **Un solo binario de escritorio** con Tauri v2, React/TypeScript y backend
   Rust. La UI solo habla con comandos Tauri tipados.
2. **Una organización por instalación** y soporte de múltiples sedes. La base
   SQLite cifrada es la fuente de verdad; no hay servidor obligatorio.
3. **Backend modular** con dependencias en una sola dirección:
   `gym-core` (dominio puro) ← `gym-data` / `gym-backup` ← app Tauri.
4. **SQLite en modo WAL** con un único escritor serializado por proceso. Las
   escrituras sensibles son transaccionales.
5. **Contratos versionados** (`gym-contracts` ↔ `packages/contracts`) con
   `CONTRACT_VERSION`; un cambio incompatible sube la versión.
6. **Sin sincronización multi-dispositivo en Phase 1.** El diseño futuro usará
   un log de eventos idempotentes y un relay que nunca vea datos en claro; se
   decidirá en un ADR propio antes de implementarse.
7. **Operación offline total**: la app arranca, migra, opera y respalda sin
   red. Las dependencias de red solo existirán para actualizaciones firmadas.

## Consecuencias

- La disponibilidad depende del equipo local y de sus backups, no de un SLA
  cloud. El runbook de restauración es parte del producto.
- No hay colaboración remota ni varias cajas sobre la misma base en Phase 1.
- El empaquetado y las pruebas deben cubrir Linux, Windows y macOS.
- La UI no puede acceder al filesystem ni a shell: todo pasa por comandos
  revisados, lo que reduce la superficie de ataque.
- El costo marginal por gimnasio es casi cero; el soporte se concentra en
  instalación, backups y recuperación.

## Alternativas descartadas

- **SaaS multi-tenant:** mayor costo operativo y dependencia de red; se puede
  ofrecer después como sincronización opcional.
- **Electron + SQLite:** mayor consumo y superficie; Tauri ya está integrado.
- **SQLite en claro:** inaceptable para disco robado.
- **Base por sede con sincronización desde el día uno:** agrega conflictos,
  identidad remota y claves por dispositivo antes de validar el producto.

## Criterios de revisión

- Antes de habilitar sincronización o una segunda caja sobre la misma base.
- Si el piloto exige collaboration remota o respaldo centralizado.
- Al cambiar de framework de escritorio o de motor de base.
- Si el costo de soporte de instalaciones locales supera el de un relay.

## Decisión pendiente

Ninguna. La estrategia de sync queda explícitamente fuera de Phase 1.
