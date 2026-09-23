# ADR 0004 — Sessions relay en mémoire uniquement

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 3

## Contexte

Le document produit exige un relay qui ne stocke jamais de données métier, avec expiration automatique des sessions et association via code à 6 chiffres.

## Décision

1. Registre `SessionRegistry` en RAM (`tokio::sync::RwLock<HashMap>`)
2. TTL configurable (`TELEPORTAL_SESSION_TTL_SECS`, défaut 600s)
3. Sweep périodique (30s) + notification `SessionExpired`
4. Capacité 1 Host + 1 Guest
5. Pas de base de données, pas de fichiers de session

## Conséquences

### Positives

- Aligné avec « pas de données métier »
- Redémarrage = table rase (acceptable MVP)
- Implémentation simple et testable

### Négatives

- Perte des sessions au crash / redeploy
- Pas de multi-instance sans sticky sessions / store partagé (hors MVP)

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| Redis / DB | Contredit le MVP « no business storage » et complexifie P3 |
| Sessions infinies | Fuite mémoire et codes zombie |
