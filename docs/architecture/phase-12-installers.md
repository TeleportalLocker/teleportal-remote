# Architecture — Phase 12 : Installateurs

## Objectif

Produire les installateurs MVP : Windows (NSIS `.exe` + MSI) et macOS (DMG Intel + Apple Silicon), via bundling Tauri v2 et un pipeline GitHub Actions `release`.

## Flux

```text
tag v* → GHA release.yml
  windows-latest → nsis.exe + msi
  macos-14 → dmg aarch64 + dmg x86_64
  → GitHub Release (artefacts non signés)
```

## Config

- [`tauri.conf.json`](../../apps/desktop-client/src-tauri/tauri.conf.json) : `createUpdaterArtifacts`, targets `nsis` / `msi` / `dmg` / `app`
- Identifier : `remote.teleportal.desktop` ; `productName` : `TeleportalRemote`
- Icônes : `.ico` / `.icns` + PNG
- Local : [`infra/release/build-local.sh`](../../infra/release/build-local.sh) (`CI=true` pour create-dmg)

## Hors scope

Auto-update (P13), signature Authenticode / notarisation Apple effective (post-certs ; hooks P14), App Store, Linux.
