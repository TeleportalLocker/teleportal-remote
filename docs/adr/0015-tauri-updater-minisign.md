# ADR 0015 — Tauri Updater + minisign + latest.json

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 13

## Contexte

Le produit exige un auto-update (canal `stable`, download arrière-plan, install auto). Les installateurs Phase 12 sont non signés OS ; l’updater Tauri vérifie l’intégrité des paquets via minisign indépendamment de Gatekeeper / SmartScreen.

## Décision

1. Plugin officiel `tauri-plugin-updater` + `@tauri-apps/plugin-updater`
2. `bundle.createUpdaterArtifacts: true`
3. Manifeste statique GitHub Releases : `latest.json`
4. Endpoint unique stable dans `plugins.updater.endpoints`
5. Clé publique minisign commitée ; privée = secret CI `TAURI_SIGNING_PRIVATE_KEY`
6. UX : check au démarrage + download/install auto + `relaunch`
7. Signature OS (Authenticode / notarisation) : hooks + runbook en **Phase 14** ; effective seulement avec certificats (ADR 0016)

## Conséquences

### Positives

- Aligné stack Tauri et pipeline release existant
- Vérification crypto des updates sans certificat Apple/Microsoft
- Canal `stable` simple

### Négatives

- Gatekeeper / SmartScreen peuvent encore avertir après install (certs OS absents ; hooks P14)
- Secrets CI obligatoires pour produire des `.sig`
- Repo GitHub figé dans l’URL endpoint

## Alternatives écartées

| Alternative | Pourquoi non (P13) |
|-------------|---------------------|
| Serveur update custom | Complexité hors MVP |
| Dynamic GitHub API endpoint | Moins portable / rate-limit |
| Signer OS dès P13 | Scope Phase 14 |
| Update manuelle uniquement | Contredit « installation automatique » |
