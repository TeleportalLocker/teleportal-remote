# Architecture — Phase 2 : Protocoles réseau

## Objectif

Définir le contrat filaire partagé entre client et relay : messages MVP, validation, codec, et abstraction transport — sans serveur réseau réel.

## Séparation des responsabilités

| Crate | Responsabilité |
|-------|----------------|
| `teleportal-protocol` | Types, validation métier protocolaire, encode/decode |
| `teleportal-transport` | Envoi/réception de frames bytes ; backends pluggables |
| Apps (Phase 3+) | Orchestration session, HTTP/WS, UI |

```text
Message (Rust)
  → validate_message
  → encode_message  → [u32 BE len][MessagePack]
  → Connection::send
  → Connection::recv
  → decode_message (+ validate)
  → Message
```

## Framing

1. Longueur payload en `u32` big-endian
2. Corps MessagePack (`rmp-serde`, champs nommés)
3. Limite : `MAX_FRAME_BYTES` = 4 MiB ; payload vidéo ≤ `MAX_VIDEO_PAYLOAD`

L’exemple JSON du document produit reste **illustratif** uniquement.

## Catalogue messages

### Signaling

`Hello`, `CreateSession`, `SessionCreated`, `JoinSession`, `JoinAccepted`, `JoinRejected`, `LeaveSession`, `PeerJoined`, `PeerLeft`, `Heartbeat`, `Error`, `SessionExpired`

### Collaboration

`CursorMove` — coords normalisées `[0.0, 1.0]`, ~20 Hz côté producteur (Phase 10)

### Contrôle

`MouseMove`, `MouseButton`, `MouseScroll`, `KeyEvent`

### Vidéo

`VideoFrame` — `VideoCodec::H264` accepté ; `H265` présent pour évolution, **rejeté** en validation MVP

## Identifiants

- `SessionId`, `PeerId`, `CursorId` : UUID v4
- `SessionCode` : exactement 6 chiffres ASCII (validé à `parse` et à la désérialisation)

## Validation MVP

- `Hello.protocol_version == PROTOCOL_VERSION` (1)
- Coords souris/curseur dans `[0.0, 1.0]` finis
- Vidéo : H264 only, dimensions > 0, taille limitée
- Champs texte requis non vides (`JoinRejected.reason`, `Error.message`, `KeyEvent.key`)

## Transport Phase 2

- Trait `Connection` : `send` / `recv` / `close`
- Helpers `send_message` / `recv_message`
- `InMemoryConnection::pair` pour tests
- WebSocket : Phase 3 ; QUIC : ultérieur
- E2E crypto : point d’extension documenté, non implémenté

## Graphe de dépendances

```text
teleportal-transport → teleportal-protocol → teleportal-shared
```

## Hors scope

Axum, bind TCP/WS, génération serveur du code 6 chiffres, Tauri, capture, injection input.
