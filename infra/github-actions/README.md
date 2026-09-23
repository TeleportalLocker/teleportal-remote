# GitHub Actions

Workflows réels : [`.github/workflows/`](../../.github/workflows/).

| Workflow | Rôle |
|----------|------|
| `ci.yml` | fmt, clippy, test, frontend, capture Win/macOS |
| `release.yml` | Installateurs + signatures updater + `latest.json` sur tag `v*` ; hooks signature OS optionnels |

## Secrets

| Secret | Obligatoire | Rôle |
|--------|-------------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Oui (release) | Minisign updater |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Non | Password minisign |
| `APPLE_*` | Non | Codesign + notarisation (voir [os-signing.md](../release/os-signing.md)) |
| `WINDOWS_CERTIFICATE` (+ password) | Non | Authenticode |

Sans secrets OS, la CI produit des installateurs **non signés OS** (Gatekeeper / SmartScreen peuvent avertir).
