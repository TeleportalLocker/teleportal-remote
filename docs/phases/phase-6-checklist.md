# Checklist de validation — Phase 6

À cocher avant validation explicite et passage à la Phase 7.

## Livrables

- [x] Architecture : `docs/architecture/phase-6-capture-macos.md`
- [x] ADR : `docs/adr/0008-screencapturekit.md`
- [x] Diagramme : `docs/diagrams/phase-6-sck.mmd`
- [x] `SckCapturer` + modules `sck/`
- [x] `CaptureError::PermissionDenied`
- [x] Alias `PlatformCapturer` macOS
- [x] CI job `capture-macos`
- [x] Validation Phase 5 cochée
- [x] Cette checklist

## Fonctionnel

- [x] `displays()` (macOS, ou `PermissionDenied`)
- [x] Capture un écran (`display_index`)
- [x] Frames BGRA8
- [x] `shows_cursor = false`
- [x] Permissions Screen Recording
- [x] Pas d’encode / pas d’UI Tauri / pas de Linux

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` (backend SCK sur macOS)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 7 — Encode H264**
