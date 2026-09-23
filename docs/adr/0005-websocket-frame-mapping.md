# ADR 0005 — Mapping frame WebSocket ↔ protocole

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 3

## Contexte

Le protocole Phase 2 définit des frames length-prefixed MessagePack. WebSocket fournit déjà des frontières de messages. Il faut un mapping unique pour le relay Axum et le client `WebsocketConnection`.

## Décision

**Un message binaire WebSocket = une frame protocolaire complète** (`[u32 BE length][msgpack]`), identique à `encode_message` / `decode_message`.

Les frames texte WS sont rejetées.

## Conséquences

### Positives

- Même chemin codec pour in-memory, WS client, et serveur
- Pas de double logique « avec / sans length prefix »

### Négatives

- Longueur redondante avec le framing WS (coût négligeable vs payload vidéo)

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| MessagePack seul dans le message WS | Deux modes encode selon transport |
| Stream length-prefixed continu sur un seul WS | Buffering plus complexe, pas nécessaire |
