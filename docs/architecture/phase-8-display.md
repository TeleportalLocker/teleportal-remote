# Architecture — Phase 8 : Affichage vidéo

## Objectif

Décoder les `Message::VideoFrame` H264 Annex-B et les afficher dans le client Tauri (canvas Guest). Brancher la boucle Host capture→encode→envoi. Pas d’input ni d’overlay.

## Pipeline

```text
Host: PlatformCapturer → OpenH264Encoder → WS → Relay
Guest: WS → OpenH264Decoder → RGBA8 → event video-frame → canvas
```

## API decode (`teleportal-video-stream`)

| Type | Rôle |
|------|------|
| `VideoDecoder` | Trait `start` / `decode` / `stop` |
| `DecodeConfig` / `DecodedFrame` | Config + RGBA8 dense |
| `PlatformDecoder` | Alias = `OpenH264Decoder` |

## Client desktop

- Host : thread capture dès `InSession` + peer ; arrêt sur leave / peer left / erreur
- Guest : decode + `emit("video-frame", VideoFrameEvent)`
- IPC : `rgbaBase64` (JSON Tauri) ; canvas `putImageData`
- Host UI : libellé « Diffusion active » (pas de preview locale)

## Hors scope

Input (P9), double curseur / overlay (P10–P11), Channel binaire, scale / shared buffer.
