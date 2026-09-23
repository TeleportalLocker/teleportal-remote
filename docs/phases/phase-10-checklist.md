# Checklist de validation — Phase 10

À cocher avant validation explicite et passage à la Phase 11.

## Livrables

- [x] Architecture : `docs/architecture/phase-10-cursor.md`
- [x] ADR : `docs/adr/0012-cursor-sync.md`
- [x] Diagramme : `docs/diagrams/phase-10-cursor.mmd`
- [x] `teleportal-cursor-sync` (tracker + throttle + interp + poll OS)
- [x] Host poll OS → `CursorMove` ; Guest `send_cursor_pos`
- [x] Consommation `CursorMove` + emit `remote-cursor`
- [x] Marqueur HTML Guest + Host
- [x] Validation Phase 9 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Envoi ~20 Hz (throttle)
- [x] Interpolation pose distante
- [x] Curseur OS local non remplacé
- [x] Pas d’injection via `CursorMove`
- [x] Pas d’overlay système bureau (P11)

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 11 — Overlay collaboratif**
