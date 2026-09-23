# ADR 0017 — Curseur collaboratif sans injection + géométrie display unique

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 15

## Contexte

Le Guest envoyait à la fois `CursorMove` (marqueur) et `Mouse*` / `KeyEvent` (injection OS Host). Le Host injectait via `PlatformInjector`. Overlay, capture et normalize étaient primary-only et pas forcément le même moniteur. Sur `PeerLeft`, le Guest restait en session zombie (`peer_connected: false`).

## Décision

1. **Pas d’injection** en session : ni émission Guest des contrôles souris/clavier, ni `PlatformInjector` côté Host. Uniquement `CursorMove` → overlay / marqueur.
2. **Pas de toggle** « prise de contrôle » en Phase 15 (reporté).
3. **Un display** capturé : `SessionDisplay` (origine + taille + index) partagé entre capture, overlay et `poll_os_cursor` / `normalize_in_display`.
4. **Tout `PeerLeft`** → `Idle` + fermeture WebSocket + arrêt capture/overlay (Host et Guest).

## Conséquences

### Positives

- Double curseur sans déplacer la souris OS Host
- Alignement multi-moniteur cohérent sur le rectangle capturé
- Fin de session claire (retour Accueil) pour les deux rôles

### Négatives

- Pas de contrôle distant souris/clavier tant que le toggle n’est pas réintroduit
- Un seul écran capturé à la fois

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| Toggle soft-control en P15 | Hors scope ; complexité UX + permissions Accessibilité |
| Multi-capture simultanée | Hors MVP |
| Garder session zombie après PeerLeft | UX confuse ; slot session bloqué |
