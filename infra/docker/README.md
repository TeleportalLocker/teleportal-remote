# Docker — Teleportal Remote

Images et Compose pour le **relay** derrière **Caddy** (TLS en prod, HTTP en local).

## Démarrage local

Depuis ce dossier :

```bash
cp .env.example .env
docker compose up --build
```

- Health : `http://127.0.0.1:8080/health`
- WebSocket : `ws://127.0.0.1:8080/ws`

Client desktop :

```bash
export TELEPORTAL_RELAY_URL=ws://127.0.0.1:8080/ws
```

## Production (HTTPS Let’s Encrypt)

Dans `.env` :

```bash
TELEPORTAL_DOMAIN=relay.example.com
TELEPORTAL_ACME_EMAIL=ops@example.com
```

Caddy obtient un certificat ACME automatiquement. Ports `80`/`443` doivent être joignables.

Client :

```bash
export TELEPORTAL_RELAY_URL=wss://relay.example.com/ws
```

## Build image seule

Depuis la **racine** du monorepo :

```bash
docker build -f infra/docker/Dockerfile -t teleportal-relay .
```

## Variables relay

| Variable | Défaut | Rôle |
|----------|--------|------|
| `TELEPORTAL_RELAY_BIND` | `0.0.0.0:7800` | Bind interne (exposé seulement au réseau compose) |
| `TELEPORTAL_SESSION_TTL_SECS` | `600` | TTL session |
| `TELEPORTAL_RATE_LIMIT_PER_MIN` | `30` | Max Create/Join par IP / minute |

Le TLS n’est **pas** terminé dans le binaire relay (ADR Phase 3) : Caddy termine HTTPS et reverse-proxy vers `relay:7800`.
