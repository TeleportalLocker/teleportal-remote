# ADR 0003 — Abstraction transport async

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 2

## Contexte

L’architecture exige Client ↔ Relay ↔ Client en WebSocket pour le MVP, avec migration possible vers QUIC sans réécrire la logique métier (sessions, curseur, vidéo, input).

## Décision

1. Crate `teleportal-transport` exposant le trait async `Connection` (`send` / `recv` / `close` sur `Bytes`)
2. Helpers `send_message` / `recv_message` branchés sur `teleportal-protocol`
3. Phase 2 : uniquement `InMemoryConnection` (tests)
4. Phase 3 : `WebSocketConnection` (côté Axum / client)
5. Plus tard : backend QUIC derrière le même trait
6. Chiffrement E2E : couche optionnelle autour des frames, hors trait de base (non MVP)

## Conséquences

### Positives

- Métier dépend de frames/messages, pas de tungstenite/quinn
- Tests d’intégration sans ports réseau
- Remplacement de backend localisé

### Négatives

- Une indirection async (`async-trait`) supplémentaire
- Spécificités WS (ping, close codes) à mapper dans l’impl Phase 3

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| Apps appellent tungstenite directement | Couplage fort, réécriture QUIC coûteuse |
| Channel tokio uniquement comme API publique | Pas assez proche du modèle connexion réseau |
