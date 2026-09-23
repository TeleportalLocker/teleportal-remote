# Checklist de validation — Phase 9

À cocher avant validation explicite et passage à la Phase 10.

## Livrables

- [x] Architecture : `docs/architecture/phase-9-input.md`
- [x] ADR : `docs/adr/0011-input-injection.md`
- [x] Diagramme : `docs/diagrams/phase-9-input.mmd`
- [x] `teleportal-input` (trait + Win/macOS)
- [x] Host injecte Mouse*/KeyEvent
- [x] Guest `send_control` + événements canvas
- [x] Validation Phase 8 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Souris move / boutons / scroll
- [x] Clavier via `KeyboardEvent.code`
- [x] Permission Accessibility (macOS)
- [x] Pas de `CursorMove` / overlay

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 10 — Double curseur**
