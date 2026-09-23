# ADR 0013 — Overlay Host via WebviewWindow transparente

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 11

## Contexte

La Phase 10 synchronise déjà `CursorMove` et affiche un marqueur HTML in-app. Le produit exige que le Host voie le curseur Guest **sur son bureau**, sans remplacer son curseur OS.

## Décision

1. Seconde fenêtre Tauri `cursor-overlay` : transparente, sans décorations, always-on-top, click-through permanent
2. Bounds = moniteur primaire (aligné poll curseur P10)
3. Contenu = même frontend avec hash `#overlay`
4. macOS : `app.macOSPrivateApi` + feature Cargo `macos-private-api` (requis pour transparence ; **hors App Store**)
5. Réutiliser `emit("remote-cursor")` ; pas de nouveau protocole
6. Guest reste sur marqueur canvas ; Host retire le marqueur in-app

## Conséquences

### Positives

- Aligné ADR 0006 (HTML / Tauri)
- Click-through = curseur OS local intact
- Pas de crate OS overlay supplémentaire

### Négatives

- API privée macOS → distribution App Store impossible tant que conservée
- Un seul moniteur (primaire) en MVP
- Fenêtre webview légèrement plus lourde qu’un layer natif

## Alternatives écartées

| Alternative | Pourquoi non (P11) |
|-------------|---------------------|
| Layer OS natif (Win/macOS) | Plus de code plateforme ; hors ADR HTML |
| Hit-test dynamique (ignore partiel) | Inutile : marqueur non interactif |
| Overlay Guest système | Le canvas vidéo suffit |
| Annotations / labels | Hors MVP |
