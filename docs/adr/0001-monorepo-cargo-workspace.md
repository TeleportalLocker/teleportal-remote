# ADR 0001 — Monorepo Cargo Workspace

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 1

## Contexte

Teleportal Remote nécessite un client desktop, un serveur relay et plusieurs bibliothèques natives (capture, input, vidéo, protocole, transport). Le produit doit rester simple à builder, tester et releaser sur Windows et macOS, avec une qualité production (fmt, clippy, tests).

## Décision

Adopter un **monorepo Cargo Workspace** unique avec :

1. **Séparation `apps/` / `crates/` / `infra/` / `docs/`**
2. **Nommage `teleportal-*`** pour tous les packages Cargo
3. **Edition Rust 2021**, `resolver = "2"`
4. **Dépendances partagées** via `[workspace.dependencies]`
5. **Crate `teleportal-shared`** comme fondation commune
6. **Toolchain pinée** via `rust-toolchain.toml` (`stable` + rustfmt + clippy)

## Conséquences

### Positives

- Un seul `cargo test --workspace` / `clippy` pour toute la qualité
- Versions et deps alignées
- Frontières de crates claires pour les phases suivantes
- Documentation (ADR, architecture, diagrammes) colocalisée

### Négatives / coûts

- Le graphe de membres grandit avec le produit (discipline de dépendances requise)
- Les packages purement librairie et les binaires partagent le même release cadence initialement

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| Multi-repos | Friction de versioning et de CI trop élevée pour un MVP |
| Un seul crate monolithique | Mauvaise isolation capture / input / protocole |
| Edition 2024 | Moins répandue en CI/images ; 2021 suffit pour le MVP |

## Suivi

Les changements structurants du graphe de dépendances feront l’objet d’ADR dédiés (ex. introduction de `protocol` → `transport`).
