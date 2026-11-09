# Runbook: Desplegar Piloto, Medir y Decidir Lanzamiento

## Objetivo
Instalar Fitness Gym en un gimnasio real, acompañar operación 2-4 semanas, medir KPIs, registrar incidencias y evaluar criterios de lanzamiento (Go/No-Go).

## Criterios de Entrada (Pre-Piloto)
| Check | Verificación |
|-------|--------------|
| Release candidate | Tag `v0.2.0-rc.X` firmado y publicado en GitHub Releases |
| Installers | `.AppImage`, `.msi`, `.dmg` verificados y firmados |
| Updater | `latest.json` servido desde GitHub Releases, firma minisign válida |
| Sync server | `gym-sync-server` desplegado en staging con certs mTLS válidos |
| Runbooks | Backup/restore, signing, chaos, pentest documentados y probados |
| Soporte | Canal Slack/Email para gimnasio piloto + on-call dev |

## Selección de Gimnasio Piloto
| Criterio | Valor objetivo |
|----------|----------------|
| Socios activos | 200-500 |
| Personal | 3-8 (dueño + 2-7 staff) |
| Clases/día | 10-30 |
| Ventas/día | 50-200 |
| Hardware | PC Windows 10/11 + lector QR/credencial (opcional) |
| Conectividad | Fibra 50Mbps+ (sync) + fallback 4G opcional |
| Disponibilidad dueño | 30 min/semana para sync + feedback |

## Checklist Pre-Instalación (Día 0)

### En gimnasio
- [ ] PC cumple requisitos (Win 10/11, 8GB RAM, 50GB libre, TPM 2.0)
- [ ] Red: IP fija / DHCP reservada, puertos 443/8443 abiertos a sync server
- [ ] Antivirus: excepción carpeta `~/FitnessGym/` (SQLCipher + backups)
- [ ] Backup previo de datos legacy (Excel/papel) exportado a CSV
- [ ] Passphrase backup generada y guardada en gestor (Bitwarden/1Password)

### En repositorio
- [ ] Tag `v0.2.0-pilot.X` creado y firmado
- [ ] Release draft en GitHub con instaladores + `latest.json`
- [ ] Sync server staging con certs mTLS (CN=org-piloto)
- [ ] Secrets CI configurados (Windows cert, macOS cert, minisign key)

## Instalación (Día 1 - ~45 min)

### Windows (MSI)
```powershell
# Ejecutar como Administrador
msiexec /i fitness-gym_0.2.0_x64_en-US.msi /qn /l*v install.log
# Verificar: C:\Program Files\Fitness Gym\fitness-gym.exe
```

### Linux (AppImage)
```bash
chmod +x fitness-gym-0.2.0-x86_64.AppImage
./fitness-gym-0.2.0-x86_64.AppImage --appimage-extract-and-run
# Crear .desktop entry opcional
```

### macOS (DMG)
```bash
hdiutil attach fitness-gym-0.2.0.dmg
cp -R /Volumes/Fitness\ Gym/Fitness\ Gym.app /Applications/
hdiutil detach /Volumes/Fitness\ Gym
# spctl --assess --type execute /Applications/Fitness\ Gym.app
```

### Configuración Inicial (15 min)
1. Abrir app → **Configura tu gimnasio**
2. Nombre gimnasio, dueño, passphrase segura (mín 12 chars)
3. **Guardar código de recuperación** en papel + gestor
3. Configurar: moneda MXN, zona horaria `America/Mexico_City`
4. Crear 1-2 miembros prueba + membresía
3. **Crear backup inicial** → Configuración → Backups → Crear backup

## Métricas de Piloto (KPIs)

| KPI | Objetivo | Fuente |
|-----|----------|--------|
| **Disponibilidad app** | > 99.5% (excluye mantenimiento) | Uptime monitor / logs |
| **Latencia acceso p95** | < 300 ms | Health endpoint / logs |
| **Latencia venta p95** | < 60 s | POS timestamps |
| **Sync success rate** | > 99.9% | Sync engine metrics |
| **Backup success rate** | 100% (diario) | Backup logs |
| **Restauración test** | < 30 min (1/semana) | Runbook test |
| **Incidencias críticas** | 0 bloqueantes | Issue tracker |
| **Incidencias altas** | < 2/semana | Issue tracker |
| **Satisfacción dueño** | ≥ 4/5 | Encuesta semanal |
| **Tiempo onboarding socio** | < 3 min | POS timestamps |
| **Cierre caja diario** | < 5 min | Cash session logs |
| **Exportación contable** | 1 click, < 10s | Reportes UI |

## Plan de Acompañamiento (Semanas 1-4)

### Semana 1: Onboarding intensivo
| Día | Actividad | Responsable |
|-----|-----------|-------------|
| Día 1 | Instalación + configuración + socios prueba | Dev + Dueño |
| Día 2 | Primera venta real + cierre caja | Dueño + Dev (remoto) |
| Día 3 | Primera clase + reserva + check-in | Dueño |
| Día 4 | Primer backup automático + verificación | Dev (remoto) |
| Día 5 | Primera exportación contable + revisión contador | Dueño + Contador |
| Día 6-7 | Uso libre + captura incidencias | Dueño + Dev (slack) |

### Semanas 2-3: Operación normal
- Daily standup 15 min (async Slack): incidencias, dudas, métricas
- Backup automático diario verificado
- Sync diario verificado (dashboard Integrations)
- 1 restauración test/semana (runbook)

### Semana 4: Cierre y decisión
- Entrevista final dueño (30 min)
- Revisión KPIs vs objetivos
- Revisión incidencias abiertas/cerradas
- Encuesta satisfacción (Google Form / Typeform)
- Decisión: **Go / No-Go / Extender 2 semanas**

## Registro de Incidencias (Template)
```markdown
### INC-YYYYMMDD-XXX
**Severidad:** Crítica / Alta / Media / Baja
**Módulo:** Acceso / POS / Sync / Backup / UI / Otro
**Descripción:** Qué pasó, pasos para reproducir
**Impacto:** Qué usuarios/datos afectados
**Workaround:** Si existe
**Root Cause:** (tras análisis)
**Fix:** Commit/PR / Config / Runbook
**Verificación:** Cómo se valida el fix
**Estado:** Abierto / En progreso / Verificado / Cerrado
```

## Dashboard de Seguimiento (Grafana / Google Sheets)
| Métrica | Semana 1 | Semana 2 | Semana 3 | Semana 4 | Objetivo |
|---------|----------|----------|----------|----------|----------|
| Uptime % | | | | | >99.5% |
| Access p95 (ms) | | | | | <300 |
| POS p95 (s) | | | | | <60 |
| Sync success % | | | | | >99.9% |
| Backups OK | | | | | 7/7 |
| Incidentes Crít | | | | | 0 |
| Incidentes Altos | | | | | <2/sem |
| Satisfacción (1-5) | | | | | ≥4 |

## Criterios de Decisión (Go/No-Go)

### ✅ GO (Lanzar v0.2.0)
- [ ] 0 incidencias críticas abiertas
- [ ] ≤ 2 incidencias altas abiertas (todas con fix planificado)
- [ ] KPIs ≥ 90% de objetivos
- [ ] Satisfacción dueño ≥ 4/5
- [ ] Contador valida exportaciones
- [ ] Backup/restore test pasa en entorno limpio
- [ ] Sync 72h offline test pasa (chaos testing)
- [ ] Pentest ASVS 0 críticos/altos abiertos

### ⚠️ EXTENDER 2 SEMANAS
- [ ] 1-2 incidencias altas sin fix definitivo
- [ ] 1 KPI por debajo de objetivo (ej. sync 99.5%)
- [ ] Satisfacción 3.5-4/5
- [ ] Feedback dueño: "casi listo, necesita pulir X"

### ❌ NO-GO
- [ ] ≥ 1 incidencia crítica sin fix
- [ ] ≥ 3 incidencias altas sin fix
- [ ] ≥ 2 KPIs por debajo de objetivo
- [ ] Satisfacción < 3.5/5
- [ ] Pérdida de datos o corrupción detectada
- [ ] Pentest con crítico/alto sin remediación

## Entregables Post-Piloto
| Entregable | Formato | Destinatario |
|------------|---------|--------------|
| Informe Piloto | PDF (5-10 págs) | Dueño, Dev Lead, PO |
| KPIs Dashboard | Google Sheets / Grafana snapshot | Equipo |
| Incidencias | GitHub Issues (export CSV) | Dev Team |
| Runbooks actualizados | Markdown en `docs/runbooks/` | Repo |
| Decisión Go/No-Go | Documento firmado | PO + Dev Lead |

## Checklist Post-Piloto
- [ ] Exportar BD piloto → anonimizar → guardar como referencia test
- [ ] Actualizar runbooks con lecciones aprendidas
- [ ] Actualizar threat model con hallazgos piloto
- [ ] Tag `v0.2.0` (o `v0.2.1` si hotfix) y release workflow
- [ ] Comunicar decisión a stakeholders (email + Slack)
- [ ] Planear siguiente release (backlog → sprint planning)

## Contactos Piloto
| Rol | Nombre | Canal | Disponibilidad |
|-----|--------|-------|----------------|
| Dueño gimnasio | | Slack/WA | 9:00-21:00 |
| Dev Lead | | Slack/GitHub | 9:00-18:00 UTC |
| Dev On-call | | PagerDuty/Slack | 24/7 (semanas 1-2) |
| Soporte contador | | Email | Lunes-Viernes 9-18 |

## Referencias
- Runbook: `docs/runbooks/backup-restore.md`
- Runbook: `docs/runbooks/signing.md`
- Runbook: `docs/runbooks/chaos-testing.md`
- Runbook: `docs/runbooks/pentest-asvs.md`
- ADR-003: Sync mTLS architecture
- ADR-002: Identity and Authorization