# Checklist de validation — Phase 14

À cocher avant validation explicite et passage à la Phase 15 (Audit final).

## Livrables

- [x] Architecture : `docs/architecture/phase-14-hardening.md`
- [x] ADR : `docs/adr/0016-runtime-hardening-unsigned-os.md`
- [x] Diagramme : `docs/diagrams/phase-14-hardening.mmd`
- [x] Rate-limit Create/Join (`TELEPORTAL_RATE_LIMIT_PER_MIN`)
- [x] Docker : Dockerfile + compose + Caddyfile + `.env.example`
- [x] Runbook `infra/release/os-signing.md`
- [x] Hooks CI `APPLE_*` / `WINDOWS_*` dans `release.yml`
- [x] Validation Phase 13 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Rate-limit prouvé par test d’intégration
- [x] Compose documenté (`ws://127.0.0.1:8080/ws` / `wss://…`)
- [x] Signature OS : hooks + docs ; **pas** de binaires réellement signés (certs absents)

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)
- [x] `docker compose config` (smoke)
- [x] `docker build -f infra/docker/Dockerfile` (image `teleportal-relay:local`)

## Reporté (certs)

- [ ] Binaires réellement signés Authenticode / notarisation Apple — **après obtention des certificats** (voir `infra/release/os-signing.md`)

## Validation

- [ ] Revue humaine / validation explicite pour ouvrir la **Phase 15 — Audit final**
