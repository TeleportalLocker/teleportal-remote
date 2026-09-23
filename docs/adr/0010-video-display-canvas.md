# ADR 0010 — Decode OpenH264 + canvas Tauri

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 8

## Contexte

Les frames H264 Annex-B transitent déjà via le relay (`Message::VideoFrame`). Il faut un affichage Guest dans l’UI HTML Tauri (ADR 0006), symétrique au soft-encode P7.

## Décision

1. Decode : **OpenH264** via `VideoDecoder` / `OpenH264Decoder` dans `teleportal-video-stream`
2. Affichage : `<canvas>` 2D + `ImageData` (RGBA8)
3. IPC : événement Tauri `video-frame` avec pixels en **base64** (sérialisation JSON fiable)
4. Host : boucle capture→encode→send dès pair connecté ; pas de preview locale
5. Drop de frames côté UI si paint encore en cours

## Conséquences

### Positives

- Même stack soft Win/macOS/CI que P7
- UI HTML prête pour overlay curseur (P10–P11)
- Round-trip encode/decode testé unitairement

### Négatives

- Coût CPU IPC base64 + copie canvas (lourd à 1080p30)
- Pas de zero-copy / Channel binaire (reporté)

## Alternatives écartées

| Alternative | Pourquoi non (P8) |
|-------------|-------------------|
| WebCodecs dans le webview | Couplage navigateur, moins aligné avec soft OpenH264 |
| egui / wgpu natif | Contredit ADR 0006 (HTML pour overlay) |
| Channel binaire Tauri d’emblée | Plus complexe ; optimisation future |
