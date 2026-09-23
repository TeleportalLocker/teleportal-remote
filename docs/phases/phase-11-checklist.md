# Checklist de validation — Phase 11

À cocher avant validation explicite et passage à la Phase 12.

## Livrables

- [x] Architecture : `docs/architecture/phase-11-overlay.md`
- [x] ADR : `docs/adr/0013-host-cursor-overlay-window.md`
- [x] Diagramme : `docs/diagrams/phase-11-overlay.mmd`
- [x] Fenêtre `cursor-overlay` (transparent / always-on-top / click-through)
- [x] Cycle de vie Host+peer
- [x] Frontend `#overlay` + Guest marqueur canvas
- [x] Validation Phase 10 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Host voit curseur Guest sur le bureau (overlay)
- [x] Curseur OS local non remplacé / click-through
- [x] Guest marqueur canvas inchangé
- [x] Pas d’annotations / multi-écrans

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 12 — Installateurs**
