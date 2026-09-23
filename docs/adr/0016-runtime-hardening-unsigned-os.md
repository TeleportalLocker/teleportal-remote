# ADR 0016 — Durcissement runtime + stubs signature OS

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 14

## Contexte

Les ADR 0014 / 0015 reportaient Authenticode et la notarisation Apple à la Phase 14. Les certificats ne sont pas disponibles. Docker et TLS pour le relay étaient des stubs depuis la Phase 1 / 3. Le code session à 6 chiffres est sensible au brute-force sans plafond.

## Décision

1. **Rate-limit** CreateSession / JoinSession par IP (fenêtre 60 s, défaut 30/min, `0` = off)
2. **Docker Compose** : binaire `teleportal-relay-server` + **Caddy** reverse-proxy
3. **TLS hors process** : Caddy (HTTP local `:8080` ou HTTPS ACME si domaine) — aligné ADR Phase 3
4. **Signature OS** : runbook [`infra/release/os-signing.md`](../../infra/release/os-signing.md) + injection conditionnelle des secrets `APPLE_*` / `WINDOWS_*` dans `release.yml` ; **pas** de signature réelle sans secrets
5. Hygiène : gitignore certificats ; matrice secrets documentée
6. Pas d’E2E crypto ni stack OTel dans cette phase

## Conséquences

### Positives

- Relay déployable en prod sans TLS in-app
- Anti-bruteforce basique sur les codes
- CI release reste verte sans Apple/Microsoft
- Chemin clair pour activer la signature OS plus tard

### Négatives

- Gatekeeper / SmartScreen persistent tant que les certs manquent
- Rate-limit en mémoire (reset au redémarrage ; mono-instance)
- Derrière un proxy mal configuré, l’IP peut être partagée

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| Bloquer P14 jusqu’aux certificats | Empêche le durcissement runtime |
| TLS rustls dans le relay | Contredit Phase 3 ; reverse-proxy suffit |
| E2E crypto | Hors MVP produit |
| Signer avec self-signed / ad-hoc | N’élimine pas les warnings OS |
