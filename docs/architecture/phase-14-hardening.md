# Architecture — Phase 14 : Durcissement

## Objectif

Durcir le **runtime** (relay) et **préparer** la signature OS des installateurs, sans bloquer la livraison sur l’absence de certificats Apple Developer / Authenticode.

## Découpage

| Volet | Livrable |
|-------|----------|
| Rate-limit | Create/Join plafonnés par IP (`TELEPORTAL_RATE_LIMIT_PER_MIN`, défaut 30/min) |
| Docker | `infra/docker` : image relay + Compose + Caddy |
| TLS | Terminé par Caddy (Let’s Encrypt si `TELEPORTAL_DOMAIN`) ; relay en HTTP interne |
| Signature OS | Runbook + env CI optionnels ; bins non signés OS tant que secrets absents |
| Secrets | Matrice documentée ; `.p12` / `.pem` gitignorés |

## Flux déploiement relay

```text
Client  --ws/wss-->  Caddy  --http-->  teleportal-relay-server:7800
```

- Local : `ws://127.0.0.1:8080/ws` (override `TELEPORTAL_RELAY_URL`)
- Prod (défaut client) : `wss://relay.teleportal.fr/ws`

IP client pour le rate-limit : `X-Forwarded-For` / `X-Real-IP` si présents (Caddy).

## CSP / capabilities

Le WebSocket relay est géré côté Rust (`WebsocketConnection`), pas par le webview : la CSP `connect-src` (GitHub updater) n’a pas besoin d’autoriser `wss://`. Aucun élargissement CSP en P14.

## Hors scope

- Signature OS **effective** (attente certificats)
- Chiffrement E2E applicatif (hors MVP produit)
- OpenTelemetry / Loki / Tempo / Pyroscope (placeholders inchangés)
- App Store / Linux
