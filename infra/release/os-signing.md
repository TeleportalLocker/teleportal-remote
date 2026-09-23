# Signature OS & notarisation (Phase 14)

Hooks CI et runbook **préparés**. La signature effective nécessite des certificats payants (Apple Developer, Authenticode) **non disponibles** dans le MVP actuel : les builds restent non signés OS tant que les secrets ci-dessous sont absents. La CI reste verte.

Distinct de la signature **minisign updater** (`TAURI_SIGNING_PRIVATE_KEY`).

## Matrice secrets GitHub

### Déjà requis (updater)

| Secret | Rôle |
|--------|------|
| `TAURI_SIGNING_PRIVATE_KEY` | Clé privée minisign |
| `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` | Mot de passe (souvent vide) |

### macOS (codesign + notarisation)

| Secret | Rôle |
|--------|------|
| `APPLE_CERTIFICATE` | Contenu du `.p12` encodé en base64 |
| `APPLE_CERTIFICATE_PASSWORD` | Mot de passe du `.p12` |
| `APPLE_SIGNING_IDENTITY` | Ex. `Developer ID Application: …` |
| `APPLE_ID` | Apple ID pour notarisation |
| `APPLE_PASSWORD` | Mot de passe **app-specific** |
| `APPLE_TEAM_ID` | Team ID (10 caractères) |

Alternative API key notarisation (optionnel, à la place de Apple ID/password) :

| Secret | Rôle |
|--------|------|
| `APPLE_API_KEY` | Contenu de la clé `.p8` |
| `APPLE_API_ISSUER` | Issuer ID |
| `APPLE_API_KEY_PATH` | Chemin local si fichier monté (CI : souvent écrit depuis le secret) |

### Windows (Authenticode)

| Secret | Rôle |
|--------|------|
| `WINDOWS_CERTIFICATE` | Certificat `.pfx` / `.p12` en base64 |
| `WINDOWS_CERTIFICATE_PASSWORD` | Mot de passe |

## Comportement CI

Dans [`release.yml`](../../.github/workflows/release.yml) :

- Les variables `APPLE_*` / `WINDOWS_*` ne sont **exportées** vers `tauri build` que si le certificat est non vide (sinon `unset` — évite l’échec `security import` avec un secret vide / placeholder).
- **Sans secrets** : Tauri produit des bundles non signés OS ; le job réussit.
- **Avec secrets valides** : Tauri codesign / notarize / Authenticode selon la plateforme.

## Préparation locale des secrets

### Encoder un `.p12` / `.pfx`

```bash
base64 -i DeveloperID.p12 | pbcopy   # macOS
# Coller dans le secret GitHub APPLE_CERTIFICATE ou WINDOWS_CERTIFICATE
```

### Vérifier l’identité macOS

```bash
security find-identity -v -p codesigning
```

## Quand les certificats seront disponibles

1. Créer les secrets dans Settings → Secrets and variables → Actions
2. Relancer un tag `v*` (ou `workflow_dispatch` + tag)
3. Vérifier sur un Mac/Windows frais : plus d’avertissement Gatekeeper / SmartScreen (réputation SmartScreen peut encore demander du volume de téléchargements)
4. Cocher la case « binaires réellement signés » dans [`docs/phases/phase-14-checklist.md`](../../docs/phases/phase-14-checklist.md)

## Hors scope

- Distribution App Store (contrainte `macOSPrivateApi`)
- Azure Trusted Signing (alternative Windows possible plus tard)
