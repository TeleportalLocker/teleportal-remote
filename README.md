# Teleportal Remote

Application de contrôle à distance multiplateforme (Windows / macOS) avec partage d’écran, contrôle clavier/souris et double curseur collaboratif.

Développé entièrement en Rust (Cargo Workspace). Voir [`docs/TELEPORTAL_REMOTE.md`](docs/TELEPORTAL_REMOTE.md) pour le document de référence produit et architecture.

## Prérequis

- [Rustup](https://rustup.rs) + toolchain stable (`rust-toolchain.toml`, composants `rustfmt`, `clippy`, `rust-analyzer`)
- Node.js 20+ (frontend Vite du client)
- macOS : Xcode Command Line Tools
- macOS (capture) : permission **Screen Recording** (Réglages système → Confidentialité et sécurité)
- macOS (contrôle distant) : permission **Accessibilité** pour l’injection CGEvent (Host)

## Build & qualité

```bash
cargo build --workspace
cargo test --workspace
cargo fmt --all -- --check
cargo clippy --workspace --all-targets -- -D warnings

cd apps/desktop-client && npm ci && npm run build
```

## Structure

| Chemin | Rôle |
|--------|------|
| `apps/desktop-client` | Client Tauri v2 (UI + `src-tauri`) |
| `apps/relay-server` | Serveur de relay (Axum) |
| `crates/*` | Bibliothèques métier partagées |
| `docs/` | Architecture, ADR, diagrammes, checklists |
| `infra/` | Docker, notes CI, release |

## Phase en cours

**Phase 14 — Durcissement** (Docker + TLS reverse-proxy, rate-limits relay, préparation signature OS). Signature Authenticode / notarisation Apple reportée jusqu’à obtention des certificats.

### Lancer le relay

```bash
cargo run -p teleportal-relay-server
```

### Lancer le client

```bash
cd apps/desktop-client
npm install
npm run tauri dev
```
