# Release — Installateurs + auto-update Teleportal Remote

Packaging Tauri v2 (Phase 12) + updater minisign / `latest.json` (Phase 13).
Signature OS / notarisation : hooks + runbook Phase 14 — **effectifs seulement si certificats configurés**. Voir [`os-signing.md`](os-signing.md).

## Artefacts cibles

| Plateforme | Artefacts |
|------------|-----------|
| Windows x64 | NSIS `.exe`, WiX `.msi`, `.sig` updater |
| macOS Apple Silicon | `.dmg` / `.app.tar.gz` + `.sig` |
| macOS Intel | `.dmg` / `.app.tar.gz` + `.sig` |
| Commun | `latest.json` (canal `stable`) |

Endpoint client :

`https://github.com/TeleportalLocker/teleportal-remote/releases/latest/download/latest.json`

## Secrets GitHub

### Updater (requis pour `createUpdaterArtifacts`)

| Secret | Rôle |
|--------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Contenu de la clé privée minisign |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Mot de passe (vide si clé sans password) |

### Signature OS (optionnel)

| Secret | Plateforme |
|--------|------------|
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`, `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID` | macOS |
| `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD` | Windows |

Détail : [`os-signing.md`](os-signing.md).

Génération locale updater :

```bash
cd apps/desktop-client
CI=true npm run tauri -- signer generate -w ../../infra/release/.updater-private.key -f --ci -p ""
# Pubkey → tauri.conf.json plugins.updater.pubkey + infra/release/updater.pubkey
# Ne jamais committer .updater-private.key
```

## Build local signé (macOS)

```bash
export CI=true
export TAURI_SIGNING_PRIVATE_KEY="$(cat infra/release/.updater-private.key)"
./infra/release/build-local.sh
# ou : cd apps/desktop-client && npm run tauri build
# Vérifier presence de *.sig sous target/release/bundle/
```

Sans clé, `tauri build` échoue si `createUpdaterArtifacts` est actif.

## Manifeste

```bash
infra/release/build-latest-json.sh 0.1.1 TeleportalLocker/teleportal-remote ./release-assets ./latest.json
```

## CI

Workflow [`.github/workflows/release.yml`](../../.github/workflows/release.yml) :

- tag `v*` → build Win/macOS avec signature updater
- injecte les secrets OS s’ils existent (sinon bundles non signés OS)
- publie installers + `.sig` + `latest.json`
