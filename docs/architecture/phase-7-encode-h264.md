# Architecture — Phase 7 : Encode H264

## Objectif

Encoder les frames **BGRA8** de `teleportal-capture` en bitstream **H264 Annex-B**, exposé via le trait `VideoEncoder` et convertible en `Message::VideoFrame`. Pas de décode / UI Tauri (Phase 8).

## API

| Type | Rôle |
|------|------|
| `VideoEncoder` | Trait `start` / `encode` / `force_keyframe` / `stop` |
| `EncodeConfig` | `max_fps`, `bitrate_bps`, `keyframe_interval` |
| `EncodedFrame` | payload Annex-B + métadonnées + `is_keyframe` |
| `to_video_message` | `EncodedFrame` → `Message::VideoFrame` (H264) |
| `PlatformEncoder` | Alias MVP = `OpenH264Encoder` |

## Pipeline

```text
Frame_BGRA8 → pack (stride) → YUV420 → OpenH264 → Annex-B → VideoFrame
```

- Dimensions forcées en **pair** (crop 1 px si impair)
- Curseur déjà exclu côté capture (P5/P6)
- Abstraction prête pour Media Foundation / VideoToolbox sans changer les appelants

## Hors scope

Decode, affichage Tauri, boucle Host session, encode hardware, H265 actif.
