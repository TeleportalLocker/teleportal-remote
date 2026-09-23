# Architecture — Phase 5 : Capture écran Windows

## Objectif

Fournir une API de capture d’écran multiplateforme et un backend Windows production-ready via **DXGI Desktop Duplication**, produisant des frames **BGRA8** CPU. Pas d’encodage ni d’UI.

## API commune

| Type | Rôle |
|------|------|
| `Capturer` | Trait `displays` / `start` / `grab` / `stop` |
| `CaptureConfig` | `display_index` (défaut 0), `max_fps` (défaut 30) |
| `Frame` | BGRA8 + stride + timestamp + `DisplayId` |
| `PlatformCapturer` | Alias `DxgiCapturer` (Windows) / `SckCapturer` (macOS, Phase 6) / `UnsupportedCapturer` |

## DXGI

```text
EnumOutputs → DuplicateOutput → AcquireNextFrame
  → CopyResource (staging) → Map → buffer BGRA → ReleaseFrame
```

- Timeout d’acquisition dérivé de `max_fps`
- `Ok(None)` si pas de mise à jour / wait timeout
- `ACCESS_LOST` : une recréation de duplication puis nouvel essai
- Curseur hardware non composé dans la frame (favorable au double curseur P10)

## Multi-écrans

`displays()` énumère toutes les sorties. Le MVP capture **un** écran via `display_index`.

## Hors scope

ScreenCaptureKit (P6), H264 (P7), affichage Tauri (P8), branchement Host UI.
