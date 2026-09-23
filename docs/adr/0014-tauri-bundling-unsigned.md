# ADR 0014 — Bundling Tauri non signé (NSIS / MSI / DMG)

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 12

## Contexte

Le produit exige des installateurs Windows (`exe`, `msi`) et macOS (`dmg` Intel + Apple Silicon). Le client est déjà Tauri v2 ; `bundle.active` était désactivé depuis la Phase 4.

## Décision

1. Activer le bundling Tauri avec cibles explicites : **`nsis`**, **`msi`**, **`dmg`**
2. `productName` = `TeleportalRemote` ; identifier = `remote.teleportal.desktop` (évite `.app` / espaces)
3. Windows : NSIS = installeur `.exe` (mode `currentUser`) ; MSI via WiX
4. macOS : deux builds `--target aarch64-apple-darwin` et `x86_64-apple-darwin` sur `macos-14` ; `CI=true` pour create-dmg
5. Artefacts **non signés OS** en MVP ; Phase 14 livre hooks CI + runbook — signature effective **après** obtention des certificats (voir ADR 0016)
6. Distribution hors App Store (contrainte `macOSPrivateApi` Phase 11)
7. Pipeline `release.yml` sur tags `v*` ; `workflow_dispatch` pour builds sans publication de Release

## Conséquences

### Positives

- Aligné stack Tauri existante
- Quatre artefacts produit couverts
- CI reproductible sans secrets de signature

### Négatives

- Gatekeeper / SmartScreen avertissent sur binaires non signés (hooks P14 prêts ; certs requis pour lever les warnings)
- MSI nécessite WiX sur les runners Windows
- Cross-compile Intel depuis ARM peut être plus lent

## Alternatives écartées

| Alternative | Pourquoi non (P12) |
|-------------|---------------------|
| Signature + notarisation dès P12 | Secrets / comptes Apple-Dev hors MVP |
| Universal macOS binary unique | Deux DMG explicites demandés par le produit |
| Electron Builder / cargo-bundle | Contredit le choix Tauri |
| `createUpdaterArtifacts` | Réservé Phase 13 |
