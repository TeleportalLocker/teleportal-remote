#!/usr/bin/env bash
# Build installateur local (cible hôte macOS / Darwin).
set -euo pipefail

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
CLIENT="$ROOT/apps/desktop-client"

# create-dmg / Finder AppleScript échoue souvent hors CI interactif.
export CI="${CI:-true}"

if [[ -f "$ROOT/infra/release/.updater-private.key" && -z "${TAURI_SIGNING_PRIVATE_KEY:-}" ]]; then
  export TAURI_SIGNING_PRIVATE_KEY
  TAURI_SIGNING_PRIVATE_KEY="$(cat "$ROOT/infra/release/.updater-private.key")"
fi

echo "==> Teleportal Remote — build local (tauri)"
cd "$CLIENT"
npm ci
npm run tauri build

echo "==> Bundles :"
find "$ROOT/target/release/bundle" -type f \( -name '*.dmg' -o -name '*.exe' -o -name '*.msi' -o -name '*.sig' -o -name '*.app' -o -name '*.tar.gz' \) 2>/dev/null \
  || echo "(aucun artefact trouvé — vérifier les logs tauri build / TAURI_SIGNING_PRIVATE_KEY)"
