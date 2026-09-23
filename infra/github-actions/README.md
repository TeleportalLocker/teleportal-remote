# GitHub Actions

Workflows réels : [`.github/workflows/`](../../.github/workflows/).

| Workflow | Rôle |
|----------|------|
| `ci.yml` | fmt, clippy, test (Ubuntu + deps GTK/WebKit pour le crate Tauri), frontend, capture Win/macOS |
| `release.yml` | Installateurs + signatures updater + `latest.json` sur tag `v*` ; hooks signature OS optionnels |
| `ecr-relay.yml` | Build `infra/docker/Dockerfile` → push `${ECR_REGISTRY}/teleportal-relay` (AWS ECR) |

## Secrets

### Updater / signature OS

| Secret | Obligatoire | Rôle |
|--------|-------------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Oui (release) | Minisign updater |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Non | Password minisign |
| `APPLE_*` | Non | Codesign + notarisation (voir [os-signing.md](../release/os-signing.md)) |
| `WINDOWS_CERTIFICATE` (+ password) | Non | Authenticode |

### AWS ECR (relay → TeleportalOperator)

| Secret | Obligatoire | Rôle |
|--------|-------------|------|
| `AWS_ACCESS_KEY_ID` | Oui (ecr-relay) | IAM avec droits ECR push |
| `AWS_SECRET_KEY` | Oui (ecr-relay) | Clé secrète (même nom que Operator) |
| `AWS_REGION` | Oui (ecr-relay) | Ex. `eu-west-1` |
| `ECR_REGISTRY` | Oui (ecr-relay) | Ex. `123456789012.dkr.ecr.eu-west-1.amazonaws.com` |

Image produite : `${ECR_REGISTRY}/teleportal-relay:latest` (+ tag SHA).  
Le repo ECR `teleportal-relay` est créé automatiquement s’il n’existe pas.

Sans secrets OS, la CI release produit des installateurs **non signés OS** (Gatekeeper / SmartScreen peuvent avertir).
