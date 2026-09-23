# ADR 0009 — OpenH264 logiciel + trait VideoEncoder

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 7

## Contexte

Le MVP exige 1080p30 H264 avec faible latence. Les frames capture sont BGRA8 CPU. Un encodeur hardware (MF / VideoToolbox) est souhaitable long terme, mais alourdit fortement la Phase 7.

## Décision

1. Backend MVP : **Cisco OpenH264** via le crate `openh264` 0.8 (`source`)
2. Trait `VideoEncoder` + alias `PlatformEncoder` pour brancher plus tard un encodeur HW
3. Sortie **Annex-B** (start codes) dans `EncodedFrame` / `Message::VideoFrame`
4. Usage type screen (`ScreenContentRealTime`), bitrate configurable (défaut 4 Mbit/s)
5. H265 reste dans le protocole mais non produit

## Conséquences

### Positives

- Même code Win / macOS / Linux CI
- Compilation out-of-the-box (pas de lib système)
- Contrat stable pour P8 (decode) et Host UI

### Négatives

- Coût CPU (BGRA→YUV + encode soft)
- Warnings OpenH264 screen-content (AQ / background detection désactivés)
- Pas encore de zero-copy GPU

## Alternatives écartées

| Alternative | Pourquoi non (P7) |
|-------------|-------------------|
| Media Foundation + VideoToolbox d’emblée | Double backend, CI complexe |
| FFmpeg | Dépendance lourde / licensing |
| openh264 ≥0.9 (edition 2024) | Inutile pour le MVP ; 0.8 suffit |
