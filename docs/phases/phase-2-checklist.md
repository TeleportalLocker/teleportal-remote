# Checklist de validation — Phase 2

À cocher avant validation explicite et passage à la Phase 3.

## Livrables

- [x] Architecture : `docs/architecture/phase-2-protocol.md`
- [x] ADR : `docs/adr/0002-wire-format-messagepack.md`
- [x] ADR : `docs/adr/0003-transport-abstraction.md`
- [x] Diagramme : `docs/diagrams/phase-2-protocol.mmd`
- [x] Diagramme : `docs/diagrams/phase-2-framing.mmd`
- [x] Code : `teleportal-protocol` + `teleportal-transport`
- [x] Tests unitaires / intégration mémoire
- [x] Cette checklist

## Protocole

- [x] `Message` couvre session, curseur, input, vidéo
- [x] `SessionCode` 6 chiffres (parse + deserialize)
- [x] `PROTOCOL_VERSION = 1`
- [x] Codec length-prefixed MessagePack
- [x] Validation MVP (version, coords, H264, tailles)
- [x] `transport` → `protocol` → `shared`

## Transport

- [x] Trait `Connection`
- [x] `send_message` / `recv_message`
- [x] `InMemoryConnection::pair`
- [x] Pas de WebSocket / Axum dans cette phase

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Hors scope vérifié

- [x] Pas de bind réseau
- [x] Pas de logique relay
- [x] Pas de Tauri / capture / input natif

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 3 — Relay server**
