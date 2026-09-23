# Checklist de validation — Phase 5

À cocher avant validation explicite et passage à la Phase 6.

## Livrables

- [x] Architecture : `docs/architecture/phase-5-capture-windows.md`
- [x] ADR : `docs/adr/0007-dxgi-desktop-duplication.md`
- [x] Diagramme : `docs/diagrams/phase-5-dxgi.mmd`
- [x] Trait `Capturer` + types `Frame` / `CaptureConfig`
- [x] Impl DXGI `DxgiCapturer`
- [x] Stub non-Windows
- [x] CI job `capture-windows`
- [x] Cette checklist

## Fonctionnel

- [x] `displays()` (Windows)
- [x] Capture un écran (`display_index`)
- [x] Frames BGRA8
- [x] Gestion timeout / `ACCESS_LOST`
- [x] Pas d’encode / pas d’UI Tauri / pas de macOS

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace` (stub macOS/Linux)
- [x] `cargo check -p teleportal-capture --target x86_64-pc-windows-msvc`

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 6 — Capture écran macOS**
