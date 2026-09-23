# Checklist de validation — Phase 13

À cocher avant validation explicite et passage à la Phase 14.

## Livrables

- [x] Architecture : `docs/architecture/phase-13-updater.md`
- [x] ADR : `docs/adr/0015-tauri-updater-minisign.md`
- [x] Diagramme : `docs/diagrams/phase-13-updater.mmd`
- [x] `tauri-plugin-updater` + `process` branchés
- [x] `createUpdaterArtifacts` + pubkey
- [x] UI check / download / install au démarrage
- [x] `teleportal-updater` helpers
- [x] `release.yml` + `latest.json` + `.sig`
- [x] Validation Phase 12 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Canal `stable` / endpoint GitHub `latest.json`
- [x] Minisign (pubkey conf, privée secret CI)
- [x] Pas de signature OS (P14)

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)
- [x] Build local signé avec `.sig` (`TeleportalRemote.app.tar.gz.sig`)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 14 — Durcissement**
