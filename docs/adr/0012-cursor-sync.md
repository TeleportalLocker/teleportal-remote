# ADR 0012 — Sync curseur ~20 Hz + marqueur HTML

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 10

## Contexte

Le différenciateur produit est le double curseur collaboratif. Le protocole expose déjà `CursorMove` (coords `[0,1]`, relay forward). Il faut produire/consommer ces messages et rendre le curseur distant visible sans remplacer le curseur OS local.

## Décision

1. Crate `teleportal-cursor-sync` : `CursorSync` (throttle 20 Hz, interpolation linéaire entre samples)
2. Host : poll OS (`GetCursorPos` / `CGEvent` location) normalisé sur l’écran primaire
3. Guest : frontend envoie `{x,y}` via `send_cursor_pos` ; Rust assigne `cursor_id` local
4. Affichage P10 : élément HTML `.remote-cursor` dans la fenêtre Tauri (Guest sur canvas, Host en session)
5. Overlay transparent always-on-top sur le bureau Host → **Phase 11**
6. Séparation stricte : `CursorMove` = collaboration visuelle ; `Mouse*` = contrôle (P9)

## Conséquences

### Positives

- Réutilise le wire format Phase 2 et le relay Phase 3
- Interpolation fluide malgré un fil à 20 Hz
- Pas de conflit avec l’injection P9

### Négatives

- Marqueur Host dans l’app (pas sur le bureau réel) jusqu’à P11
- Poll OS Host = écran primaire uniquement (MVP)

## Alternatives écartées

| Alternative | Pourquoi non (P10) |
|-------------|---------------------|
| Overlay système bureau Host | Scope Phase 11 |
| Dessiner le curseur dans la frame vidéo | Capture exclut déjà le curseur hardware |
| Remplacer le curseur OS local | Contredit le produit |
| Fréquence fil = 60 Hz | Bande passante inutile ; interp suffit |
