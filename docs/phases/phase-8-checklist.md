# Checklist de validation — Phase 8

À cocher avant validation explicite et passage à la Phase 9.

## Livrables

- [x] Architecture : `docs/architecture/phase-8-display.md`
- [x] ADR : `docs/adr/0010-video-display-canvas.md`
- [x] Diagramme : `docs/diagrams/phase-8-display.mmd`
- [x] Trait `VideoDecoder` + `OpenH264Decoder`
- [x] Boucle Host capture→encode→envoi
- [x] Guest decode + event `video-frame`
- [x] Canvas frontend
- [x] Validation Phase 7 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Decode Annex-B → RGBA8
- [x] Round-trip encode/decode (tests)
- [x] Affichage Guest canvas
- [x] Host diffuse sans preview locale
- [x] Pas d’input / pas d’overlay

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 9 — Contrôle souris / clavier**
