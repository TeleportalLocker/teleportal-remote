#!/usr/bin/env bash
# Assemble latest.json for Tauri updater from release assets + .sig files.
# Usage: build-latest-json.sh <version> <github_repo> <assets_dir> <out_json>
set -euo pipefail

VERSION="${1:?version required (semver without v)}"
REPO="${2:?owner/repo required}"
ASSETS="${3:?assets dir required}"
OUT="${4:?output json path required}"
TAG="v${VERSION}"
BASE="https://github.com/${REPO}/releases/download/${TAG}"
PUB_DATE="$(date -u +"%Y-%m-%dT%H:%M:%SZ")"

json_escape() {
  # Escape backslash and double-quote for JSON strings.
  printf '%s' "$1" | sed 's/\\/\\\\/g; s/"/\\"/g'
}

platform_entry() {
  local platform="$1"
  local file="$2"
  local sig
  if [[ ! -f "${ASSETS}/${file}" || ! -f "${ASSETS}/${file}.sig" ]]; then
    return 1
  fi
  sig="$(tr -d '\n\r' < "${ASSETS}/${file}.sig")"
  printf '"%s":{"signature":"%s","url":"%s/%s"}' \
    "$platform" "$(json_escape "$sig")" "$BASE" "$(json_escape "$file")"
}

# First matching basename in ASSETS that has a sibling .sig
find_asset() {
  local f base
  for f in "$@"; do
    base="$(basename "$f")"
    if [[ -f "${ASSETS}/${base}" && -f "${ASSETS}/${base}.sig" ]]; then
      echo "$base"
      return 0
    fi
  done
  return 1
}

ENTRIES=()

# darwin-aarch64
if file="$(find_asset \
  "${ASSETS}"/TeleportalRemote.app.tar.gz \
  "${ASSETS}"/*aarch64*.app.tar.gz \
  "${ASSETS}"/*aarch64*.dmg \
  "${ASSETS}/TeleportalRemote_${VERSION}_aarch64.app.tar.gz")"; then
  ENTRIES+=("$(platform_entry "darwin-aarch64" "$file")")
else
  echo "warning: no asset for platform darwin-aarch64" >&2
fi

# darwin-x86_64 (exclude aarch64-named files)
X64_FILE=""
for f in "${ASSETS}"/*x86_64*.app.tar.gz "${ASSETS}"/*x64*.app.tar.gz \
         "${ASSETS}"/*x86_64*.dmg "${ASSETS}"/*x64*.dmg; do
  [[ -e "$f" ]] || continue
  base="$(basename "$f")"
  [[ "$base" == *aarch64* ]] && continue
  if [[ -f "${ASSETS}/${base}.sig" ]]; then
    X64_FILE="$base"
    break
  fi
done
if [[ -n "$X64_FILE" ]]; then
  ENTRIES+=("$(platform_entry "darwin-x86_64" "$X64_FILE")")
else
  echo "warning: no asset for platform darwin-x86_64" >&2
fi

# windows-x86_64
if file="$(find_asset \
  "${ASSETS}"/*.exe \
  "${ASSETS}"/*.msi \
  "${ASSETS}"/*.nsis.zip \
  "${ASSETS}/TeleportalRemote_${VERSION}_x64-setup.exe")"; then
  ENTRIES+=("$(platform_entry "windows-x86_64" "$file")")
else
  echo "warning: no asset for platform windows-x86_64" >&2
fi

if [[ ${#ENTRIES[@]} -eq 0 ]]; then
  echo "error: no platform entries built from ${ASSETS}" >&2
  ls -la "${ASSETS}" >&2 || true
  exit 1
fi

{
  echo '{'
  echo "  \"version\": \"${VERSION}\","
  echo "  \"notes\": \"Teleportal Remote ${VERSION} (stable)\","
  echo "  \"pub_date\": \"${PUB_DATE}\","
  echo '  "platforms": {'
  for i in "${!ENTRIES[@]}"; do
    if [[ $i -lt $((${#ENTRIES[@]} - 1)) ]]; then
      echo "    ${ENTRIES[$i]},"
    else
      echo "    ${ENTRIES[$i]}"
    fi
  done
  echo '  }'
  echo '}'
} >"$OUT"

echo "Wrote ${OUT}"
cat "$OUT"
