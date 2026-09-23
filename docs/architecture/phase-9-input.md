# Architecture — Phase 9 : Contrôle souris / clavier

## Objectif

Permettre au Guest de contrôler souris, clavier et scroll sur la machine Host via les messages protocole existants. Injection native SendInput (Windows) / CGEvent (macOS). Pas de double curseur ni overlay.

## Flux

```text
Guest canvas/keyboard → send_control → WS → Relay → Host → PlatformInjector → OS
```

## API `teleportal-input`

| Type | Rôle |
|------|------|
| `InputInjector` | Trait `start` / `inject` / `stop` |
| `InjectConfig` | Taille écran pour coords `[0,1]` → pixels |
| `PlatformInjector` | Win / Mac / Stub |

- Clés : `KeyboardEvent.code` (ex. `KeyA`) mappé vers VK / CGKeyCode
- Permissions macOS : Accessibility (`AXIsProcessTrusted`) → `PermissionDenied`

## Client

- Guest : événements canvas (`pointer*` / `wheel`) + `keydown`/`keyup` (focus canvas), throttle move ~30 Hz
- Host : injecte dès pair connecté ; ignore `CursorMove` (P10)

## Hors scope

`CursorMove`, overlay, `teleportal-cursor-sync`, Linux, contrôle Host→Guest.
