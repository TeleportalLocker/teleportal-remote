# ADR 0006 — Tauri v2 + UI vanilla

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 4

## Contexte

Le client doit être léger, multiplateforme (Windows / macOS), entièrement orchestré en Rust, avec une UI simple pour Host/Rejoindre.

## Décision

1. **Tauri v2** comme shell natif
2. **Vite + TypeScript vanilla** (pas de React/Vue) pour l’UI
3. Package Cargo dans `apps/desktop-client/src-tauri`
4. Logique session testable dans `session.rs` (`apply_signal` pur + orchestration WS)

## Conséquences

### Positives

- Surface front minimale, build rapide
- Même protocole / transport que les tests relay
- Séparation nette UI / session Rust

### Négatives

- Prérequis Node pour le front + rustup pour MSRV Tauri
- Pas de composants UI riches out-of-the-box

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| egui / iced pur Rust | Moins adapté à une UI HTML future (overlay vidéo) |
| React | Plus lourd que nécessaire pour le MVP session |
| Electron | Contredit « entièrement Rust » / empreinte |
