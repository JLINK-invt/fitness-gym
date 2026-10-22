# Contrato local · acceso simulado (Phase 1)

- **Versión:** 1.2.0 (payload de acceso sin cambios desde 1.1.0)
- **Estado:** implementado para el vertical slice de Phase 1
- **Ámbito:** app Tauri, core Rust y SQLite local; no cruza la red

## Objetivo

Demostrar el flujo de acceso de punta a punta sin integrar el lector real:

1. La app de acceso envía una lectura simulada al core.
2. `gym-core` decide localmente y sin I/O.
3. `gym-data` registra el evento una sola vez mediante `idempotency_key`.
4. Tauri devuelve la decisión y la UI muestra el evento persistido.

El evento sobrevive reinicios y restauraciones porque vive en `module_events`.
El comando exige una sesión local con permiso `operate_pos`; no hay firmas,
biometría, membresías ni torniquete.

## Componentes

```text
UI React (Acceso)
  │ invoke("simulate_access", { request })
  ▼
Comando Tauri ──► gym-core::access (decisión pura)
  │
  └────────────► gym-data (organization + module_events)
```

La “app de acceso” de esta fase es la propia pantalla de simulación. El
contrato queda separado para que un módulo externo lo consuma después.

## Flujo mínimo

1. La UI exige que el gimnasio esté configurado.
2. Genera un `idempotencyKey` nuevo por lectura (`crypto.randomUUID()`).
3. Envía credencial pública, lector y clave de idempotencia.
4. El core valida campos y decide:
   - `CRED-ALLOW` → permitido.
   - `CRED-DENY` → denegado.
   - cualquier otra → denegada por desconocida.
5. El core calcula `SHA-256(credential_id + "\n" + device_id)`. La credencial
   nunca se guarda en claro.
6. SQLite inserta en `module_events` con
   `UNIQUE (organization_id, idempotency_key)` y `ON CONFLICT DO NOTHING`.
7. La respuesta incluye el evento durable y `duplicate`.
8. La UI consulta `list_access_events` y muestra el evento una sola vez.

## Contrato

### `AccessRequest`

```json
{
  "credentialId": "CRED-ALLOW",
  "deviceId": "recepcion-1",
  "idempotencyKey": "8f14e45f-ea0d-4c3b-9f0e-6b1d7a2c1234"
}
```

- `credentialId`: identificador público simulado, 1–64 bytes.
- `deviceId`: identificador del lector, 1–64 bytes.
- `idempotencyKey`: obligatorio, 1–128 bytes, único por lectura.

### `AccessDecision`

```json
{
  "eventId": "3f0e6b1d-7a2c-4c3b-9f0e-8f14e45fea0d",
  "allowed": true,
  "reason": "credencial simulada activa",
  "duplicate": false,
  "idempotencyKey": "8f14e45f-ea0d-4c3b-9f0e-6b1d7a2c1234",
  "latencyMs": 4,
  "decidedAt": "2026-10-21T09:30:00.123456+00:00"
}
```

`duplicate: true` significa que la clave ya existía y se devolvió el resultado
original; no se creó otro evento.

### `AccessEventSummary`

```json
{
  "eventId": "3f0e6b1d-7a2c-4c3b-9f0e-8f14e45fea0d",
  "allowed": true,
  "idempotencyKey": "8f14e45f-ea0d-4c3b-9f0e-6b1d7a2c1234",
  "occurredAt": "2026-10-21T09:30:00.123456+00:00"
}
```

### Eventos persistidos

| `event_type` | Significado |
| --- | --- |
| `access.allowed` | Decisión permitida |
| `access.denied` | Decisión denegada |

Todos usan `aggregate = access_decision`, `direction = outbound` y
`state = applied`.

### Errores

| Código | Caso |
| --- | --- |
| `invalidRequest` | Campos vacíos o demasiado largos |
| `invalidRequest` | Gimnasio sin configurar |
| `invalidRequest` | Misma clave con otra credencial/lector |
| `storage` | SQLite cerrada, bloqueada o error de escritura |

## Idempotencia

- La misma clave con la misma solicitud devuelve el mismo `eventId` y
  `duplicate: true`.
- La misma clave con otra solicitud se rechaza; no se sobrescribe el evento.
- La restricción `UNIQUE (organization_id, idempotency_key)` es la última
  defensa, incluso si dos llamadas llegan a la vez.
- Un reinicio o restore no borra la clave: la tabla forma parte del snapshot.

## Rendimiento

Objetivo del vertical slice: **p95 < 300 ms** desde el clic hasta la respuesta
y el evento visible, en el equipo local. `latencyMs` mide core + SQLite.

Medición del 2026-10-21 (100 muestras, decisión + inserción + consulta de la
lista en SQLite cifrada): **p95 = 0.35 ms**. El benchmark es local e ignorado por
defecto; se ejecuta con:

```bash
cargo test -p gym-data access_event_latency_p95_is_under_300ms -- --ignored --nocapture
```

No incluye el render de React ni el IPC de Tauri; el margen frente a 300 ms es
amplio, pero la medición de la UI completa sigue siendo manual.

## Privacidad y límites

- No se guarda la credencial en claro, solo su hash junto con el lector.
- No se guarda biometría, nombre ni membresía.
- La simulación no concede acceso físico; no controla hardware.
- El módulo real requerirá pairing, autenticación mutua, firma, TLS local y
  política fail-open/fail-closed. Eso pertenece a Phase 2 y no está implementado.

## Compatibilidad

La versión 1.1.0 solo agrega tipos y comandos; los payloads 1.0.0 siguen siendo
válidos. Un cambio incompatible debe subir `CONTRACT_VERSION` en Rust y
TypeScript al mismo tiempo.
