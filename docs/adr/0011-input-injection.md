# ADR 0011 — Injection SendInput / CGEvent

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 9

## Contexte

Le protocole expose déjà `MouseMove`, `MouseButton`, `MouseScroll`, `KeyEvent`. Il faut une injection OS fiable pour le Host, sans empiéter sur le double curseur (P10).

## Décision

1. Crate `teleportal-input` avec trait `InputInjector` + backends Windows (`SendInput`) et macOS (`CGEvent`)
2. Coords normalisées `[0,1]` → écran primaire Host
3. Identifiants clavier = `KeyboardEvent.code` (stable cross-layout pour l’injection physique)
4. macOS : exiger **Accessibility** ; message d’erreur explicite si refus
5. Guest seul émetteur ; Host seul injecteur
6. `CursorMove` / overlay hors scope (P10–P11)

## Conséquences

### Positives

- Réutilise le wire format Phase 2 et le forward relay Phase 3
- Abstraction prête pour un stub Linux / backends futurs
- Séparation claire contrôle vs collaboration visuelle

### Négatives

- Mapping code→VK/CGKeyCode incomplet pour touches rares
- Absolute mouse sur multi-écrans MVP = écran primaire uniquement
- Prompt Accessibility potentiellement bloquant au premier pair

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| Injecter aussi depuis le Host | Hors modèle de rôles MVP |
| Overlay curseur dans P9 | Réservé P10 |
| Envoyer `event.key` (caractère) | Moins fiable pour l’injection physique |
