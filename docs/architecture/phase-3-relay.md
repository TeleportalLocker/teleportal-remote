# Architecture — Phase 3 : Relay server

## Objectif

Servir de point central Client ↔ Relay ↔ Client : création de session, code 6 chiffres, association host/guest, routage des messages protocolaires, expiration automatique. Aucune donnée métier persistée.

## Endpoints

| Méthode | Chemin | Rôle |
|---------|--------|------|
| `GET` | `/health` | Santé (`ok`) |
| `GET` | `/ws` | Upgrade WebSocket binaire |

Bind : `TELEPORTAL_RELAY_BIND` (défaut `0.0.0.0:7800`).  
TTL session : `TELEPORTAL_SESSION_TTL_SECS` (défaut `600`), sweep toutes les 30s.

## Flux signaling

1. Client WS → `Hello { protocol_version, role }`
2. Host → `CreateSession` → `SessionCreated { session_id, code }`
3. Guest → `JoinSession { code }` → `JoinAccepted` (+ `PeerJoined` vers host)
4. Ensuite : forward des messages média/contrôle/curseur/heartbeat
5. `LeaveSession` / drop TCP → `PeerLeft` à l’autre pair ; session détruite si vide
6. TTL dépassé → `SessionExpired` + destruction

## Capacité MVP

- 1 Host + 1 Guest par session
- Second guest → `JoinRejected { reason: "session full" }`
- Code inconnu → `JoinRejected { reason: "session not found" }`

## Messages forwardés

`CursorMove`, `MouseMove`, `MouseButton`, `MouseScroll`, `KeyEvent`, `VideoFrame`, `Heartbeat`

Les messages signaling (`Hello`, `CreateSession`, `JoinSession`, …) ne sont **pas** forwardés.

## Framing WebSocket

1 message binaire WS = 1 frame `encode_message` (u32 BE + MessagePack), voir ADR 0005.

## Composants

```text
apps/relay-server/
  config.rs     # env
  code.rs       # génération code
  session.rs    # Session / PeerSlot
  state.rs      # SessionRegistry (RAM)
  handler.rs    # machine à états connexion
  ws.rs         # Axum /health + /ws
  cleanup.rs    # sweep TTL
```

`teleportal-transport::WebsocketConnection` : client WS pour tests et Phase 4.

## Hors scope

Tauri, capture, persistance, multi-invités, TLS terminé dans l’app (peut être devant un reverse-proxy), E2E crypto.
