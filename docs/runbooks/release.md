# Runbook: Release de Fitness Gym

## Objetivo
Generar instaladores firmados (minisign) con actualizador automático para Linux (AppImage/.deb), Windows (NSIS/.msi) y macOS (DMG/.app) y publicar una release draft en GitHub.

## Prerrequisitos
- Claves minisign (`minisign -G`) → `TAURI_SIGNING_PRIVATE_KEY` y `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` como secrets de GitHub.
- Clave pública del updater (`TAURI_UPDATER_PUBKEY`) como variable de GitHub.
- Repo `JLINK-invt/fitness-gym` con branch `main` protegido y tags `vX.Y.Z`.
- Permisos de `contents: write` en el workflow.

## Flujo estándar
1. Merge a `main` → CI verde (web + rust matrix).
2. Tag semver: `git tag v1.2.3 && git push origin v1.2.3`.
3. Workflow **Release** se dispara:
   - Sustituye `REEMPLAZAR_TAURI_UPDATER_PUBKEY` por el secreto en `tauri.release.conf.json`.
   - `tauri-action` construye en 4 runners (linux-x86_64, windows-x86_64, macos-aarch64, macos-x86_64).
   - Firma con `TAURI_SIGNING_PRIVATE_KEY` → artefactos `.minisig` + `latest.json`.
   - Crea **release draft** en GitHub con los instaladores y `latest.json`.
4. Revisar la release draft: changelog, versiones, assets.
5. Publicar la release → el updater de los clientes la detecta.

## Variables / Secrets requeridos
| Nombre | Tipo | Descripción |
| --- | --- | --- |
| `TAURI_SIGNING_PRIVATE_KEY` | secret | Clave privada minisign (PEM, base64). |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | secret | Passphrase de la clave privada. |
| `TAURI_UPDATER_PUBKEY` | variable (no secreto) | Clave pública minisign (una línea, `minisign -G` salida). |

## Generar claves minisign (una sola vez)
```bash
pnpm tauri signer generate
# Guarda la privada en ~/.minisign/minisign.key (proteger)
# Copia la pública a la variable TAURI_UPDATER_PUBKEY
```

## Verificación local (opcional)
```bash
pnpm tauri build --config apps/gym-desktop/src-tauri/tauri.release.conf.json \
  --runner cargo -- --sign --signing-private-key ~/.minisign/minisign.key
```
Los instaladores saldrán en `apps/gym-desktop/src-tauri/target/release/bundle/`.

## Checklist pre-release
- [ ] CI `main` verde (web + rust matrix).
- [ ] `CHANGELOG.md` actualizado.
- [ ] Versión en `Cargo.toml` / `package.json` coincide con el tag.
- [ ] Secrets `TAURI_SIGNING_PRIVATE_KEY` / `PASSWORD` / `TAURI_UPDATER_PUBKEY` configurados en GitHub.
- [ ] Test manual del instalador en VM limpia (Linux/Windows/macOS).
- [ ] Updater detecta la release draft y `check()` devuelve la nueva versión.

## Rollback
Si la release tiene un bug crítico:
1. `git tag -d vX.Y.Z && git push origin :refs/tags/vX.Y.Z`
2. Borrar la release draft en GitHub.
3. Re-tag con parche y volver a lanzar.

## Anti-downgrade
El updater compara semver estricto: solo acepta versiones **mayores** a la instalada. El JSON `latest.json` incluye `version`, `pub_date`, `platforms` y `signature`. El binario verifica la firma minisign antes de descargar.

## Notas de seguridad
- La clave privada **nunca** toca el repo ni los artefactos públicos.
- Los instaladores se distribuyen vía GitHub Releases (HTTPS, TLS).
- El updater valida la firma **antes** de escribir el nuevo binario.
- No hay canal de downgrade: versiones antiguas no aparecen en `latest.json`.
- `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` evita uso accidental de la clave.