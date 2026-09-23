# Architecture — Phase 6 : Capture écran macOS

## Objectif

Backend macOS de capture d’écran via **ScreenCaptureKit**, aligné sur le même trait `Capturer` et le même format **BGRA8** CPU que DXGI (Phase 5). Gestion des permissions Screen Recording. Pas d’encode ni d’UI.

## API commune

| Type | Rôle |
|------|------|
| `Capturer` | Trait `displays` / `start` / `grab` / `stop` |
| `CaptureConfig` | `display_index`, `max_fps` → `minimumFrameInterval` |
| `Frame` | BGRA8 + stride + timestamp + `DisplayId` |
| `PlatformCapturer` | `DxgiCapturer` (Windows) / `SckCapturer` (macOS) / stub sinon |
| `CaptureError::PermissionDenied` | Accès Screen Recording refusé |

## ScreenCaptureKit

```text
CGPreflightScreenCaptureAccess → SCShareableContent
  → SCContentFilter (display) → SCStream + configuration BGRA
  → callback → Mutex<Option<Frame>> → grab() (timeout)
```

- Curseur exclu (`shows_cursor = false`) pour le double curseur P10
- Push SCStream converti en pull via buffer + `Condvar`
- macOS 13+ (Intel + Apple Silicon)
- Crate `screencapturekit` 0.3.6 (bindings objc stables)

## Permissions

Sans Screen Recording (Réglages système → Confidentialité et sécurité), `start` / `displays` renvoient `PermissionDenied`. Les tests CI acceptent ce cas (runners headless sans TCC).

## Hors scope

Encode H264 (P7), affichage Tauri (P8), input, overlay curseur, Linux.
