# Chaos Testing Framework: Carga Caos + Prueba Offline 72h

## Objetivo
Someter acceso, POS y sync a carga, cortes, reinicios, duplicados y 72 horas sin Internet. Validar que objetivos de latencia se cumplen, no hay pérdida/duplicación y backlog se drena al reconectar.

## Objetivos de Aceptación
| Métrica | Objetivo |
|---------|----------|
| Latencia p95 acceso local | < 300 ms |
| Latencia p95 venta POS | < 60 s |
| Duplicados en sync | 0 |
| Pérdida de eventos | 0 |
| Backlog drenado tras reconexión | < 5 min |
| Integridad DB post-caos | `integrity_check` OK |

## Arquitectura de Prueba

```
┌─────────────────────────────────────────────────────────────┐
│                    CHAOS ORCHESTRATOR                         │
│  (script Python que controla el caos por 72h)               │
└──────────────┬──────────────────────────────────────────────┘
               │
     ┌─────────┼─────────────┐
     ▼         ▼             ▼
┌────────┐ ┌────────┐ ┌────────────┐
│ ACCESS │ │   POS  │ │   SYNC     │
│ LOADER │ │ LOADER │ │ CHAOS AGENT│
└────┬───┘ └────┬───┘ └──────┬─────┘
     │          │            │
     ▼          ▼            ▼
┌─────────────────────────────────────────────────────────────┐
│                      TAURI APP (Linux)                        │
│  ┌─────────┐ ┌─────────┐ ┌─────────┐ ┌─────────────────┐   │
│  │ ACCESS  │ │   POS   │ │  SYNC   │ │   CHAOS AGENT   │   │
│  │  CORE   │ │  CORE   │ │ ENGINE  │ │  (inyección     │   │
│  └─────────┘ └─────────┘ └─────────┘ │   fallos red)   │   │
└─────────────────────────────────────────────────────────────┘
```

## Componentes

### 1. Access Loader (`chaos/access_loader.py`)
```python
# Simula N lectores de acceso concurrentes
# Cada lector: credencial → decision → relé → evento
# Configurable: tasa (req/s), % duplicados, % fallos red
```

### 2. POS Loader (`chaos/pos_loader.py`)
```python
# Simula N cajas concurrentes
# Flujo: carrito → descuento (opcional) → cobro (efectivo/transfer/point) → ticket
# Configurable: tickets/min, % devoluciones, % modo offline
```

### 3. Sync Chaos Agent (`chaos/sync_chaos.py`)
```python
# Inyecta fallos de red en el sync engine
# - Latencia variable (10ms - 5s)
# - Packet loss (0-30%)
# - Particiones de red (cortes 1s-300s)
# - Reinicios de servidor sync
# - Corrupción de paquetes (raro)
```

### 4. Chaos Orchestrator (`chaos/orchestrator.py`)
```python
# Plan de 72h:
phases = [
    {"duration": "2h", "load": "baseline", "chaos": "none"},
    {"duration": "4h", "load": "high", "chaos": "latency_100ms"},
    {"duration": "8h", "load": "high", "chaos": "packet_loss_10%"},
    {"duration": "24h", "load": "variable", "chaos": "offline_total"},  # 24h offline
    {"duration": "8h", "load": "recovery", "chaos": "reconnection_storm"},
    {"duration": "26h", "load": "baseline", "chaos": "none"},
]
```

## Métricas a Capturar (Prometheus / CSV)

| Métrica | Fuente | Frecuencia |
|---------|--------|------------|
| `access_latency_p95` | Access core | 10s |
| `pos_latency_p95` | POS core | 10s |
| `sync_backlog_size` | Sync engine | 30s |
| `sync_events_pushed` | Sync engine | 30s |
| `sync_events_pulled` | Sync engine | 30s |
| `db_integrity_ok` | SQLite PRAGMA | 5min |
| `disk_usage_percent` | OS | 5min |
| `memory_usage_mb` | OS | 5min |

## Ejecución

### Prerrequisitos
```bash
# 1. VM Linux limpia (Ubuntu 22.04)
# 2. App compilada en release
cargo build --release -p gym-desktop

# 2. Configurar variables
export CHAOS_DURATION_HOURS=72
export CHAOS_ACCESS_RATE=50      # req/s
export CHAOS_POS_RATE=10         # ventas/min
export CHAOS_DB_PATH=/tmp/chaos_fitness.db

# 3. Base de datos limpia
rm -f /tmp/chaos_fitness.db*
```

### Ejecutar
```bash
# Desde repo root
cd chaos/
python3 -m venv .venv
source .venv/bin/activate
pip install -r requirements.txt  # prometheus-client, psutil, requests

# Ejecutar 72h (se puede interrumpir con Ctrl+C)
python3 orchestrator.py --duration 72 --output chaos_results/

# Para test rápido (2h)
python3 orchestrator.py --duration 2 --profile quick --output chaos_results/
```

### Resultados
```bash
# Genera reporte HTML + CSV
ls chaos_results/
# chaos_report.html
# metrics.csv
# events.jsonl
# summary.json
```

### Verificar Aceptación (script)
```bash
python3 verify_acceptance.py chaos_results/summary.json
# Exit code 0 = PASS, 1 = FAIL
```

## Inyección de Fallos de Red (Linux tc)

```bash
# Añadir latencia 100ms
sudo tc qdisc add dev eth0 root netem delay 100ms

# Packet loss 10%
sudo tc qdisc change dev eth0 root netem loss 10%

# Corte total (offline)
sudo tc qdisc change dev eth0 root netem loss 100%

# Restaurar
sudo tc qdisc del dev eth0 root
```

> **Nota**: El `sync_chaos.py` usa `tc` vía `subprocess` (requiere sudo). Para CI sin sudo, usar proxy TCP (`toxiproxy`) en contenedor.

## Métricas de Éxito (Resumen Final)

```json
{
  "duration_hours": 72,
  "total_access_events": 12500000,
  "total_pos_transactions": 43200,
  "sync_events_pushed": 89450,
  "sync_events_pulled": 89450,
  "duplicates_detected": 0,
  "events_lost": 0,
  "max_sync_backlog": 12450,
  "backlog_drain_time_seconds": 187,
  "p95_access_latency_ms": 247,
  "p95_pos_latency_ms": 4200,
  "db_integrity_checks_passed": 8640,
  "downtime_incidents": 0,
  "recovery_time_after_offline_seconds": 45,
  "verdict": "PASS"
}
```

## Checklist de Ejecución
- [ ] VM aprovisionada (4 vCPU, 8GB RAM, 100GB SSD)
- [ ] App release build verificada (`cargo build --release`)
- [ ] Base de datos limpia + passphrase conocida
- [ ] Certificados sync mTLS generados (CA + server + client)
- [ ] Servidor sync levantado en misma red
- [ ] Prometheus + Grafana opcional para visualización live
- [ ] Alertas configuradas (latencia > 1s, backlog > 100k)
- [ ] Logs centralizados (JSONL → Loki / archivo)
- [ ] Script de verificación final automatizado

## Reporte Final
Generar `chaos_report.html` con:
- Gráficas de latencia/backlog en el tiempo
- Timeline de fases de caos
- Tabla de métricas vs objetivos
- Incidentes detectados y resolución
- Veredicto final: **PASS / FAIL**

## Integración CI (Opcional)
```yaml
# .github/workflows/chaos.yml (manual trigger)
# Ejecuta versión reducida (2h) en PRs a main
# Requiere self-hosted runner con sudo + KVM
```

## Referencias
- [Chaos Engineering principles](https://principlesofchaos.org/)
- [Netem](https://man7.org/linux/man-pages/man8/tc-netem.8.html)
- [Toxiproxy](https://github.com/Shopify/toxiproxy)