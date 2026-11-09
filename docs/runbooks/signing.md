# Runbook: Firma de instaladores, Updater y Licencia local

## Objetivo
Generar instaladores firmados para Linux (AppImage/.deb), Windows (NSIS/MSI) y macOS (DMG/.app) con actualizador automático verificado por minisign y licencia offline firmada.

## Prerrequisitos

### Certificados requeridos
| Plataforma | Certificado | Almacén | Variable CI |
|------------|-------------|---------|-------------|
| Windows | EV Code Signing (EV) | Azure Key Vault / HSM | `TAURI_WINDOWS_CERT` |
| Windows | Timestamp RFC3161 | DigiCert / Sectigo | `TAURI_WINDOWS_TS_URL` |
| macOS | Developer ID Application | Apple Developer / Keychain | `TAURI_MACOS_CERT_NAME` |
| macOS | Notarización | Apple ID + App-specific password | `TAURI_MACOS_AC_USER`, `TAURI_MACOS_AC_PASSWORD` |
| Linux | minisign (updater) | Archivo local / Vault | `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_UPDATER_PUBKEY` |

### Generar claves minisign (una vez)
```bash
# Genera par de claves para el updater
pnpm tauri signer generate
# Salida:
# Private key:  save to ~/.minisign/minisign.key (proteger con passphrase)
# Public key:   copiar a TAURI_UPDATER_PUBKEY (variable GitHub)
```

## Proceso de firma en CI

### Workflow `.github/workflows/release.yml` (ya configurado)
```yaml
# El workflow usa tauri-action@v0 con:
args: --config src-tauri/tauri.release.conf.json
env:
  TAURI_SIGNING_PRIVATE_KEY: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY }}
  TAURI_SIGNING_PRIVATE_KEY_PASSWORD: ${{ secrets.TAURI_SIGNING_PRIVATE_KEY_PASSWORD }}
  TAURI_UPDATER_PUBKEY: ${{ vars.TAURI_UPDATER_PUBKEY }}
  TAURI_WINDOWS_CERT: ${{ secrets.TAURI_WINDOWS_CERT }}
  TAURI_WINDOWS_TS_URL: ${{ vars.TAURI_WINDOWS_TS_URL }}
  TAURI_MACOS_CERT_NAME: ${{ vars.TAURI_MACOS_CERT_NAME }}
  TAURI_MACOS_AC_USER: ${{ vars.TAURI_MACOS_AC_USER }}
  TAURI_MACOS_AC_PASSWORD: ${{ secrets.TAURI_MACOS_AC_PASSWORD }}
```

### Configuración `tauri.release.conf.json`
```json
{
  "bundle": {
    "createUpdaterArtifacts": true,
    "targets": ["deb", "appimage", "nsis", "msi", "dmg", "app"]
  },
  "plugins": {
    "updater": {
      "active": true,
      "pubkey": "REEMPLAZAR_TAURI_UPDATER_PUBKEY",
      "endpoints": ["https://github.com/JLINK-invt/fitness-gym/releases/latest/download/latest.json"]
    }
  }
}
```

## Licencia offline firmada

### Estructura de licencia
```json
{
  "product": "fitness-gym",
  "version": "0.2.0",
  "licensee": "Gimnasio Ejemplo",
  "issued_at": "2026-10-29T00:00:00Z",
  "expires_at": "2027-10-29T00:00:00Z",
  "features": ["unlimited_members", "sync", "reports"],
  "signature": "minisign_signature_base64"
}
```

### Verificación en la app (Rust)
```rust
use minisign_verify::PublicKey;

pub fn verify_license(license_bytes: &[u8], pubkey: &str) -> Result<License, Error> {
    let pk = PublicKey::from_base64(pubkey)?;
    let (license, sig) = split_license_and_signature(license_bytes)?;
    pk.verify(&license, &sig)?;
    let license: License = serde_json::from_slice(&license)?;
    if license.expires_at < chrono::Utc::now() {
        return Err(Error::LicenseExpired);
    }
    Ok(license)
}
```

## Verificación local (desarrollo)
```bash
# Configurar claves locales
export TAURI_SIGNING_PRIVATE_KEY="$(cat ~/.minisign/minisign.key)"
export TAURI_SIGNING_PRIVATE_KEY_PASSWORD="tu_passphrase"
export TAURI_UPDATER_PUBKEY="RWQ...clave_publica_base64..."

# Build firmado local
pnpm tauri build --config apps/gym-desktop/src-tauri/tauri.release.conf.json
```

## Checklist de verificación
- [ ] Certificados Windows EV + timestamp configurados en CI
- [ ] Certificado macOS Developer ID + notarización funcionando
- [ ] Clave minisign generada y pública en `TAURI_UPDATER_PUBKEY`
- [ ] `tauri.release.conf.json` con `createUpdaterArtifacts: true`
- [ ] `tauri-action` genera `.minisig` + `latest.json` en release
- [ ] App verifica firma minisign antes de instalar actualización
- [ ] Licencia offline se verifica al arranque (sin phone-home)
- [ ] Rollback funciona: versión anterior sigue verificando

## Referencias
- [tauri-plugin-updater](https://github.com/tauri-apps/tauri-plugin-updater)
- [minisign](https://jedisct1.github.io/minisign/)
- [tauri-action](https://github.com/tauri-apps/tauri-action)
- [Code signing Tauri](https://tauri.app/distribute/sign/)