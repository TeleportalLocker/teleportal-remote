# Checklist de validation — Phase 1

À cocher avant validation explicite et passage à la Phase 2.

## Livrables

- [x] Architecture : `docs/architecture/phase-1-monorepo.md`
- [x] ADR : `docs/adr/0001-monorepo-cargo-workspace.md`
- [x] Diagramme : `docs/diagrams/phase-1-workspace.mmd`
- [x] Code : workspace Cargo + 2 apps + 8 crates
- [x] Tests : smoke tests sur chaque crate lib
- [x] Documentation : README racine + rustdoc libs
- [x] Cette checklist

## Structure

- [x] `Cargo.toml` workspace avec tous les membres listés
- [x] `rust-toolchain.toml` (stable, rustfmt, clippy)
- [x] `.gitignore` adapté
- [x] `apps/desktop-client` et `apps/relay-server` binaires stub
- [x] Crates : `shared`, `capture`, `input`, `protocol`, `transport`, `video-stream`, `cursor-sync`, `updater`
- [x] `infra/docker`, `infra/github-actions`, `infra/release` présents
- [x] Aucune dépendance entre crates métier (uniquement vers `shared`)

## Qualité

- [x] `cargo fmt --all -- --check` OK
- [x] `cargo clippy --workspace --all-targets -- -D warnings` OK
- [x] `cargo test --workspace` OK
- [x] `cargo build --workspace` OK
- [x] Aucun warning

## CI

- [x] `.github/workflows/ci.yml` : fmt, clippy, test

## Hors scope vérifié

- [x] Pas de Tauri
- [x] Pas d’Axum / logique relay
- [x] Pas de protocoles réseau
- [x] Pas de capture / vidéo / input / curseur natifs

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 2 — Protocoles réseau**
