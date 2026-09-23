# Checklist de validation — Phase 4

À cocher avant validation explicite et passage à la Phase 5.

## Livrables

- [x] Architecture : `docs/architecture/phase-4-desktop.md`
- [x] ADR : `docs/adr/0006-tauri-v2-vanilla-ui.md`
- [x] Diagramme : `docs/diagrams/phase-4-session-ui.mmd`
- [x] App Tauri v2 + UI Host/Rejoindre/Session
- [x] Commands + events session
- [x] Tests `apply_signal`
- [x] CI job `desktop-ui`
- [x] Cette checklist

## Fonctionnel

- [x] Accueil avec URL relay
- [x] Héberger → code 6 chiffres
- [x] Rejoindre via code
- [x] Quitter session
- [x] Placeholder vidéo
- [x] Pas de capture / input / overlay

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (apps/desktop-client)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 5 — Capture écran Windows**
