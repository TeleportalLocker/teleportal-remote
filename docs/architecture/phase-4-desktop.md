# Architecture — Phase 4 : Application Tauri

## Objectif

Fournir le shell desktop Teleportal Remote : UI Host / Rejoindre, connexion au relay Phase 3, affichage du code 6 chiffres et placeholder vidéo. Pas de capture ni de contrôle distant.

## Stack

| Couche | Techno |
|--------|--------|
| Shell | Tauri v2 |
| Front | Vite + TypeScript vanilla |
| Session | `teleportal-protocol` + `WebsocketConnection` |
| Observabilité | `tracing` via `teleportal-shared` |

## Layout

```text
apps/desktop-client/
  src/                 # UI
  src-tauri/           # Rust (member workspace)
  dist/                # build Vite
```

## Flux utilisateur

1. Accueil : URL relay + **Héberger** / **Rejoindre**
2. Hôte : `host_start` → code affiché → attend `PeerJoined`
3. Invité : saisie code → `guest_join` → session
4. Session : placeholder vidéo + **Quitter** (`leave_session`)

## Commands Tauri

| Command | Rôle |
|---------|------|
| `app_version` | Version workspace |
| `get_config` | URL relay (`TELEPORTAL_RELAY_URL` ou défaut) |
| `get_session_state` | Dernier snapshot |
| `host_start` | Crée une session |
| `guest_join` | Rejoint via code |
| `leave_session` | Quitte et ferme le WS |

## Events

- `session-state` : payload `SessionState`
- `session-error` : message string

## Hors scope

Capture, encode/decode, input, overlay curseur, installateurs, auto-update.
