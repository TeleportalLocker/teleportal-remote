# Architecture — Phase 13 : Auto-update

## Objectif

Mettre à jour le client via **Tauri Updater** (canal `stable`) : check au démarrage, téléchargement en arrière-plan, installation automatique + relaunch. Intégrité des paquets via **minisign** (distinct de la signature OS Phase 14).

## Flux

```text
GitHub Release (installers + .sig + latest.json)
  ← check() — desktop (plugin updater)
  → downloadAndInstall()
  → relaunch()
```

Endpoint :

`https://github.com/teleportal/teleportal-remote/releases/latest/download/latest.json`

## Composants

| Pièce | Rôle |
|-------|------|
| `tauri-plugin-updater` | Runtime check / download / install |
| `tauri-plugin-process` | `relaunch` |
| `teleportal-updater` | Constantes canal / URL + helpers semver |
| `createUpdaterArtifacts` + target `app` | Produit `.app.tar.gz` + `.sig` au build |
| `infra/release/build-latest-json.sh` | Assemble le manifeste |

## Client

Au démarrage (Accueil) : `check()` silencieux ; si update → statut « Mise à jour… » → install → relaunch. Erreurs non bloquantes.

## Hors scope

Authenticode / notarisation Apple (hooks P14, signature effective post-certs), canaux beta, serveur update dédié.
