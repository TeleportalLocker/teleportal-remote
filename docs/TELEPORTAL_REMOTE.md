# Teleportal Remote — Document de référence

Source de vérité produit, architecture et phasage. À consulter avant chaque phase de développement.

---

## 1. Rôle et principes

Tu es un Staff Software Engineer spécialisé en Rust, systèmes distribués, streaming temps réel, Tauri, macOS et Windows.

Ta mission est de concevoir et développer un logiciel professionnel nommé **Teleportal Remote**.

Agir comme un architecte logiciel senior et toujours privilégier :

- simplicité
- maintenabilité
- performance
- sécurité
- testabilité
- architecture propre

Ne jamais produire de code expérimental ou de démonstration. Tout le code doit être **production-ready**.

---

## 2. Objectif produit

Créer une application de contrôle à distance multiplateforme permettant :

- partage d’écran
- contrôle du clavier et de la souris
- collaboration à distance
- double curseur visible simultanément
- connexion via code à 6 chiffres
- installation simple
- faible latence

### Plateformes supportées

- Windows 10+
- Windows 11+
- macOS Intel
- macOS Apple Silicon (M1 / M2 / M3 / M4)

Le projet doit être développé **entièrement en Rust**.

---

## 3. Différenciateur principal

Le différenciateur majeur du produit est le **double curseur collaboratif**.

### Contraintes

- Chaque participant conserve son propre curseur.
- Le curseur local ne doit jamais être remplacé.
- Le curseur distant doit être affiché dans un overlay.
- Les deux utilisateurs doivent voir en permanence :
  - leur curseur
  - le curseur distant
- Le système doit fonctionner même lorsqu’une personne prend le contrôle du poste.

---

## 4. Périmètre MVP

### Inclus

- partage écran
- contrôle distant
- double curseur
- connexion via code à 6 chiffres
- relay server centralisé
- auto-update
- installateurs Windows et macOS

### Exclus

- transfert de fichiers
- audio
- chat
- comptes utilisateurs
- organisations
- WebRTC
- P2P
- multi-écrans
- annotations

---

## 5. Architecture globale

Monorepo **Cargo Workspace**.

```text
apps/
  desktop-client/
  relay-server/

crates/
  capture/
  input/
  protocol/
  transport/
  video-stream/
  cursor-sync/
  updater/
  shared/

infra/
  docker/
  github-actions/
  release/

docs/
  architecture/
  adr/
  diagrams/
```

---

## 6. Client

Stack :

- Rust stable
- Tauri v2
- Tokio
- Serde
- Tracing

Le client doit être léger. L’interface doit être simple.

---

## 7. Serveur relay

Stack :

- Rust
- Tokio
- Axum

Fonctions :

- création de session
- génération code 6 chiffres
- association des pairs
- routage des messages
- nettoyage automatique

Le serveur ne doit **jamais** stocker de données métier.

---

## 8. Streaming vidéo

### Objectifs

- 1080p
- 30 FPS
- latence faible

### Pipeline

```text
Capture → Encodage → Transport → Décodage → Affichage
```

Utiliser **H264** pour le MVP. Prévoir une abstraction permettant **H265** plus tard.

---

## 9. Capture

### Windows

- API : DXGI Desktop Duplication
- Support MVP : un écran
- Architecture prête pour plusieurs écrans

### macOS

- API : ScreenCaptureKit
- Support : Intel et Apple Silicon
- Gérer les permissions système

---

## 10. Contrôle distant

| Plateforme | API        |
|------------|------------|
| Windows    | SendInput  |
| macOS      | CGEvent    |

Support :

- souris
- clavier
- scroll

Architecture commune entre plateformes.

---

## 11. Cursor sync

Créer un protocole dédié.

Exemple de message :

```json
{
  "type": "CursorMove",
  "cursor_id": "...",
  "x": 0,
  "y": 0,
  "timestamp": 0
}
```

- Fréquence cible : **20 mises à jour par seconde**
- Interpolation fluide
- Limiter la bande passante

---

## 12. Transport

Commencer simple.

```text
Client ↔ Relay Server ↔ Client
```

Prévoir une abstraction permettant :

- WebSocket aujourd’hui
- QUIC demain

sans réécriture métier.

---

## 13. Sécurité

MVP :

- code à 6 chiffres
- expiration automatique de session
- validation des messages

Prévoir une architecture compatible **E2E** plus tard (non implémentée dans le MVP).

---

## 14. Auto-update

- Utiliser Tauri Updater
- Canal : `stable`
- Téléchargement en arrière-plan
- Installation automatique

---

## 15. Observabilité

Utiliser **tracing**.

Prévoir intégration future (abstractions uniquement, ne pas implémenter maintenant) :

- OpenTelemetry
- Loki
- Tempo
- Pyroscope

---

## 16. Qualité

Obligatoire :

- `cargo fmt`
- `cargo clippy`
- tests unitaires
- tests d’intégration
- documentation rustdoc

**Aucun warning accepté.**

---

## 17. CI/CD

GitHub Actions.

Pipelines :

- build
- test
- clippy
- release

### Artefacts de build

| Plateforme | Artefacts              |
|------------|------------------------|
| Windows    | exe, msi               |
| macOS      | dmg Intel, dmg Apple Silicon |

---

## 18. Phases de réalisation

Ne jamais développer plusieurs phases simultanément. Attendre la **validation explicite** de la phase précédente avant de passer à la suivante.

| Phase | Contenu                          |
|-------|----------------------------------|
| 1     | Initialisation monorepo          |
| 2     | Protocoles réseau                |
| 3     | Relay server                     |
| 4     | Application Tauri                |
| 5     | Capture écran Windows            |
| 6     | Capture écran macOS              |
| 7     | Transport vidéo                   |
| 8     | Affichage vidéo                  |
| 9     | Contrôle souris / clavier        |
| 10    | Double curseur                   |
| 11    | Overlay collaboratif             |
| 12    | Installateurs                    |
| 13    | Auto-update                      |
| 14    | Durcissement                     |
| 15    | Curseur collab, multi-écran, fin session |
| 16    | Audit final                      |

### Livrables obligatoires par phase

1. Architecture
2. ADR
3. Diagrammes
4. Code
5. Tests
6. Documentation
7. Checklist de validation

Ne jamais passer à la phase suivante sans validation explicite.
