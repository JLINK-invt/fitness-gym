# Runbook: Backups, Restauración y Runbooks

## Objetivo
Documentar y validar los procedimientos de backup, restauración y operación en entornos limpios. Validar RPO/RTO y que una persona distinta al autor pueda ejecutar los runbooks sin ayuda.

## RPO / RTO Objetivos
| Métrica | Objetivo |
|---------|----------|
| RPO (pérdida máxima de datos) | 24 horas (backup diario automático) |
| RTO (tiempo de recuperación) | < 30 minutos (restauración completa) |

## Tipos de Backup

| Tipo | Frecuencia | Retención | Destino |
|------|------------|-----------|---------|
| Automático (programado) | Diario 03:00 | 15 backups / 30 días | Local (`~/FitnessGym/backups`) |
| Manual (bajo demanda) | Bajo demanda | Según política | Local + opcional nube |
| Pre-actualización | Antes de cada release | 5 últimas | Local |

## Formato de Backup
- Archivo: `fitness-gym-YYYYMMDDTHHMMSS.gymbak`
- Cifrado: XChaCha20-Poly1305 + Argon2id (passphrase usuario)
- Integridad: SHA-256 en sidecar `.sha256`
- Contenido: SQLCipher dump + metadatos (schema_version, checksum)

## Procedimiento: Backup Automático
```bash
# Configurado en tauri.conf.json / background task
# Se ejecuta diariamente a las 03:00 si hay sesión activa
# Comando equivalente manual:
pnpm tauri backup create --passphrase "tu_passphrase"
```

## Procedimiento: Backup Manual (UI)
1. Ir a **Configuración → Backups**
2. Click **Crear backup**
3. Ingresar passphrase (mín 8 chars)
3. Esperar confirmación "Backup creado: fitness-gym-20261029T030000.gymbak"
4. Verificar archivo `.sha256` generado junto al `.gymbak`

## Procedimiento: Restauración (UI)
1. Ir a **Configuración → Backups**
2. Click **Restaurar** en el backup deseado
2. Ingresar passphrase del backup
3. Confirmar: "Esto reemplazará TODOS los datos actuales. ¿Continuar?"
3. App se cierra, restaura, y reabre en pantalla de login
4. Verificar: datos restaurados, backup anterior listado

## Procedimiento: Restauración CLI (emergencia)
```bash
# 1. Detener app
pkill -f "fitness-gym"

# 2. Restaurar con herramienta CLI
pnpm tauri backup restore \
  --backup-file ~/FitnessGym/backups/fitness-gym-20261029T030000.gymbak \
  --passphrase "tu_passphrase" \
  --db-path ~/FitnessGym/fitness-gym.db

# 3. Verificar integridad
pnpm tauri backup verify --db-path ~/FitnessGym/fitness-gym.db
```

## Runbook: Restauración en Entorno Limpio (RTO Test)

### Prerrequisitos
- Máquina limpia (VM/VirtualBox) con Ubuntu 22.04 / Windows 11 / macOS 14
- App **no instalada**
- Archivo backup `.gymbak` + `.sha256` + passphrase

### Pasos
1. Instalar app (AppImage / MSI / DMG) **sin abrir**
2. Copiar `.gymbak` + `.sha256` a `~/FitnessGym/backups/`
3. Ejecutar restauración CLI (ver arriba)
4. Abrir app → login con credenciales restauradas
5. Verificar:
   - [ ] Gimnasio configurado (nombre, moneda, zona horaria)
   - [ ] Socios y membresías (conteo coincide)
   - [ ] Inventario y catálogo
   - [ ] Reportes generan correctamente
   - [ ] Auditoría muestra eventos históricos

### Cronometrar
- Inicio: `date +%s` antes del paso 1
- Fin: `date +%s` después de verificación completa
- **RTO = fin - inicio** (objetivo < 1800s / 30 min)

## Runbook: Verificación de Integridad Backup
```bash
# Verificar checksum
sha256sum -c fitness-gym-20261029T030000.gymbak.sha256

# Verificar estructura interna (sin passphrase)
pnpm tauri backup info --backup-file backup.gymbak
# Salida esperada:
# schema_version: 22
# encrypted: true
# cipher: xchacha20poly1305
# kdf: argon2id
# size_bytes: 12345678
```

## Políticas de Retención (implementadas en código)
```rust
// apps/gym-desktop/src-tauri/src/lib.rs - prune_old_backups()
const RETAINED: usize = 15;      // Mantener 15 más recientes
const GRACE_DAYS: i64 = 30;      // Solo podar > 30 días
// Lógica: ordenar por created_at desc, saltar primeros 15,
// luego eliminar los que tengan created_at > 30 días
```

## Checklist de Validación (ejecutar mensualmente)
- [ ] Backup automático se ejecutó anoche (ver `~/FitnessGym/backups/`)
- [ ] `.sha256` coincide para cada backup
- [ ] Restauración en VM limpia < 30 min (cronometrar)
- [ ] Datos post-restauración concuerdan con origen
- [ ] Passphrase documentada y accesible para dueño
- [ ] Backup en nube opcional sincronizado (si configurado)

## Incidentes Conocidos
| Incidente | Causa | Solución |
|-----------|-------|----------|
| Restore falla "database locked" | App no cerró completamente | `pkill -9` antes de restore |
| Passphrase incorrecta | Teclado en layout distinto | Verificar layout antes de ingresar |
| Backup corrupto | Disco lleno durante escritura | Verificar espacio libre > 2x tamaño DB |

## Escalamiento
| Nivel | Contacto | SLA |
|-------|----------|-----|
| 1 - Backup falla | DevOps / Dev | 1 hora |
| 2 - Restore falla en producción | Dev + Dueño | 30 min |
| 3 - Pérdida de datos | Dueño + Legal | Inmediato |