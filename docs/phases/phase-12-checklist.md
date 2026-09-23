# Checklist de validation — Phase 12

À cocher avant validation explicite et passage à la Phase 13.

## Livrables

- [x] Architecture : `docs/architecture/phase-12-installers.md`
- [x] ADR : `docs/adr/0014-tauri-bundling-unsigned.md`
- [x] Diagramme : `docs/diagrams/phase-12-release.mmd`
- [x] `bundle.active` + targets nsis / msi / dmg
- [x] Icônes `.ico` / `.icns`
- [x] `infra/release` + `build-local.sh`
- [x] Workflow `.github/workflows/release.yml`
- [x] Validation Phase 11 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Windows : exe (NSIS) + msi configurés
- [x] macOS : dmg arm64 + x86_64 (CI dual target)
- [x] Artefacts non signés documentés
- [x] Pas d’auto-update (P13)

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)
- [x] Tentative `tauri build` local (DMG aarch64 OK avec `CI=true`)

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 13 — Auto-update**
