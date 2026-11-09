#!/usr/bin/env bash
set -euo pipefail

failures=0

run_check() {
  local label="$1"
  shift
  echo "=== ${label} ==="
  if "$@"; then
    echo "OK: ${label}"
  else
    echo "FALLO: ${label}"
    failures=$((failures + 1))
  fi
  echo
}

have() {
  command -v "$1" >/dev/null 2>&1
}

if have cargo-audit || have cargo; then
  if have cargo-audit || cargo audit --help >/dev/null 2>&1; then
    run_check "CARGO AUDIT" cargo audit --deny warnings
  else
    echo "OMITIDO: cargo-audit no instalado (cargo install cargo-audit)"
    echo
  fi
else
  echo "OMITIDO: cargo no disponible"
  echo
fi

if have cargo-deny; then
  run_check "CARGO DENY" cargo deny check advisories bans licenses sources
else
  echo "OMITIDO: cargo-deny no instalado (cargo install cargo-deny)"
  echo
fi

if have cargo-geiger; then
  run_check "CARGO GEIGER" cargo geiger --forbid-only
else
  echo "OMITIDO: cargo-geiger no instalado (cargo install cargo-geiger)"
  echo
fi

run_check "PNPM AUDIT" pnpm audit --prod --audit-level=high

if have trivy; then
  run_check "TRIVY FS" trivy fs --severity HIGH,CRITICAL --ignore-unfixed .
else
  echo "OMITIDO: trivy no instalado"
  echo
fi

run_check "FORMATO RUST" cargo fmt --all -- --check
run_check "CLIPPY" cargo clippy --workspace --all-targets -- -D warnings
run_check "PRUEBAS RUST" cargo test --workspace
run_check "LINT FRONTEND" pnpm lint
run_check "TIPOS FRONTEND" pnpm typecheck
run_check "PRUEBAS FRONTEND" pnpm test

if [ -f apps/gym-desktop/src-tauri/tauri.release.conf.json ]; then
  if grep -q "createUpdaterArtifacts" apps/gym-desktop/src-tauri/tauri.release.conf.json; then
    echo "OK: artefactos de updater habilitados"
  else
    echo "FALLO: createUpdaterArtifacts ausente en tauri.release.conf.json"
    failures=$((failures + 1))
  fi
fi

echo
if [ "$failures" -gt 0 ]; then
  echo "AUDITORIA FALLIDA: ${failures} comprobaciones con error"
  exit 1
fi

echo "AUDITORIA COMPLETA: todas las comprobaciones pasaron"
