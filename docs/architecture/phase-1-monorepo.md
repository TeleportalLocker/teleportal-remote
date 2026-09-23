# Architecture — Phase 1 : Monorepo

## Objectif

Poser le socle Cargo Workspace du produit Teleportal Remote : membres, frontières de crates, dépendances partagées, et conventions de qualité. Aucune fonctionnalité métier n’est implémentée dans cette phase.

## Principes

- **Simplicité** : stubs documentés, pas de code mort ni de dépendances inutilisées dans les libs métier.
- **Pas de cycles** : graphe de dépendances acyclique.
- **`shared` en fondation** : seul crate autorisé comme dépendance commune en Phase 1.
- **Apps minces** : logique dans les crates ; les binaires orchestrent.

## Membres du workspace

### Applications (`apps/`)

| Membre | Crate | Rôle Phase 1 | Phase métier |
|--------|-------|--------------|--------------|
| `desktop-client` | `teleportal-desktop-client` | Stub : tracing + log version | Phase 4 (Tauri v2) |
| `relay-server` | `teleportal-relay-server` | Stub : tracing + log version | Phase 3 (Axum) |

### Bibliothèques (`crates/`)

| Crate | Rôle futur | Phase |
|-------|------------|-------|
| `teleportal-shared` | Version, observabilité, utilitaires | 1 (partiel) |
| `teleportal-protocol` | Messages / codecs réseau | 2 |
| `teleportal-transport` | WebSocket MVP, abstraction QUIC | 2 / 7 |
| `teleportal-capture` | DXGI / ScreenCaptureKit | 5–6 |
| `teleportal-video-stream` | Pipeline H264 (+ abstraction H265) | 7–8 |
| `teleportal-input` | SendInput / CGEvent | 9 |
| `teleportal-cursor-sync` | Double curseur ~20 Hz | 10–11 |
| `teleportal-updater` | Tauri Updater | 13 |

## Règles de dépendances (Phase 1)

Autorisé :

```text
apps/*          → teleportal-shared
crates/* (sauf shared) → teleportal-shared
```

Interdit en Phase 1 :

- dépendances entre crates métier (ex. `video-stream` → `capture`)
- cycles
- dépendances d’un crate lib vers une app

Les arêtes métier seront ajoutées explicitement dans la phase concernée et documentées dans un ADR si le graphe change de façon structurante.

## `teleportal-shared` (contenu Phase 1)

- `VERSION` : version workspace
- `observability::init_tracing()` : subscriber console + `RUST_LOG`
- `observability::init_opentelemetry_placeholder()` : extension future documentée, non branchée

## Infra

| Chemin | Rôle |
|--------|------|
| `infra/docker/` | Réservé (relay / obs) |
| `infra/github-actions/` | Documentation CI ; workflows dans `.github/workflows/` |
| `infra/release/` | Packaging Phases 12–13 |

## Qualité

- `cargo fmt`
- `cargo clippy --workspace --all-targets -- -D warnings`
- tests unitaires smoke sur chaque crate
- rustdoc (`#![deny(missing_docs)]` sur les libs)

## Hors scope Phase 1

Protocoles, relay Axum, Tauri, capture, vidéo, input, curseur, installateurs, auto-update.
