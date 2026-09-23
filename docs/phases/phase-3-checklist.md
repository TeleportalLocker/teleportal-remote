# Checklist de validation — Phase 3

À cocher avant validation explicite et passage à la Phase 4.

## Livrables

- [x] Architecture : `docs/architecture/phase-3-relay.md`
- [x] ADR : `docs/adr/0004-relay-in-memory-sessions.md`
- [x] ADR : `docs/adr/0005-websocket-frame-mapping.md`
- [x] Diagramme : `docs/diagrams/phase-3-relay.mmd`
- [x] Code : `teleportal-relay-server` + `WebsocketConnection`
- [x] Tests unitaires registry / codes
- [x] Tests intégration Create/Join/forward + health
- [x] Cette checklist

## Fonctionnel

- [x] `GET /health` → 200
- [x] `GET /ws` WebSocket
- [x] CreateSession → code 6 chiffres
- [x] JoinSession + PeerJoined
- [x] Forward CursorMove
- [x] JoinRejected code inconnu / session pleine
- [x] Cleanup TTL
- [x] Pas de persistance métier

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Hors scope vérifié

- [x] Pas de Tauri
- [x] Pas de capture / input natif
- [x] Pas de multi-invités
- [x] Pas de DB

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 4 — Application Tauri**
