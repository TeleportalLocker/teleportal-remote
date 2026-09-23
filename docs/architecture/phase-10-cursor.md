# Architecture — Phase 10 : Double curseur

## Objectif

Synchroniser les positions de curseur Host ↔ Guest via `Message::CursorMove` (~20 Hz), interpoler côté récepteur, et afficher le curseur distant comme marqueur HTML dans la fenêtre Tauri. Le curseur OS local n’est jamais remplacé.

## Flux

```text
Local (OS poll Host / pointer Guest)
  → CursorSync.push_local (~20 Hz)
  → WS CursorMove → Relay → peer
  → CursorSync.on_remote + remote_pose (interp)
  → emit remote-cursor → marqueur HTML
```

## API `teleportal-cursor-sync`

| Type | Rôle |
|------|------|
| `CursorSync` | Id local, throttle envoi, samples distants, `remote_pose` |
| `SyncConfig` | `send_hz` (20), `max_interp_ms` |
| `CursorPose` | Pose normalisée `[0,1]` |
| `poll_os_cursor` | Position OS Host → coords normalisées |

## Client

- Guest : `send_cursor_pos { x, y }` (Rust wrappe `CursorMove` + throttle) ; marqueur sur `.video-stage` canvas
- Host : tick poll OS dès pair connecté ; marqueur dans la zone session Host
- `CursorMove` n’injecte pas (contrôle reste `Mouse*` P9)

## Hors scope

Overlay transparent système bureau Host, labels collaboratifs riches, annotations, Linux poll OS (stub), injection via `CursorMove`.
