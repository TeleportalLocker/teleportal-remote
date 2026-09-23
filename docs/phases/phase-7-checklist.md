# Checklist de validation — Phase 7

À cocher avant validation explicite et passage à la Phase 8.

## Livrables

- [x] Architecture : `docs/architecture/phase-7-encode-h264.md`
- [x] ADR : `docs/adr/0009-openh264-encoder.md`
- [x] Diagramme : `docs/diagrams/phase-7-encode.mmd`
- [x] Trait `VideoEncoder` + `EncodeConfig` / `EncodedFrame`
- [x] Backend `OpenH264Encoder`
- [x] `to_video_message` → `Message::VideoFrame`
- [x] Validation Phase 6 cochée
- [x] Cette checklist

## Fonctionnel

- [x] Encode BGRA8 → H264 Annex-B
- [x] Keyframe détectée (`is_keyframe`)
- [x] Message protocole valide (`validate_message`)
- [x] Abstraction prête pour HW
- [x] Pas de decode / pas d’UI Tauri

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`

## Validation

- [x] Revue humaine / validation explicite pour ouvrir la **Phase 8 — Affichage vidéo**
