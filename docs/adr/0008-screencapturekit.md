# ADR 0008 — ScreenCaptureKit + permissions TCC

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 6

## Contexte

Le MVP macOS doit capturer l’écran avec le même contrat que DXGI (frames BGRA8, trait `Capturer`). Les APIs historiques (`CGDisplayStream`) sont dépréciées ; ScreenCaptureKit est l’API Apple recommandée depuis macOS 12.3 / 13.

## Décision

1. API : **ScreenCaptureKit** via le crate `screencapturekit` **0.3.6** (bindings objc, sans Swift/apple-metal)
2. Format livré : **BGRA8** CPU (copie depuis `CVPixelBuffer`)
3. Curseur : `shows_cursor = false` (composition côté client en P10)
4. Permissions : `CGPreflightScreenCaptureAccess` / `CGRequestScreenCaptureAccess` → `CaptureError::PermissionDenied`
5. Modèle pull : callback SCStream → `Mutex` + `Condvar` ; `grab()` attend jusqu’à `frame_timeout_ms`

## Conséquences

### Positives

- API Apple moderne, support multi-affichage
- Même `Frame` / trait que Windows → pipeline encode unique (P7)
- Tests CI tolérants à l’absence de permission TCC

### Négatives

- Dépendance Optional Screen Recording (TCC) — échec sans grant utilisateur
- Copie CPU du pixel buffer (acceptable avant encode GPU dédié)
- Pin de version 0.3.6 : versions ≥8 du crate basculent sur Swift et cassent le build local avec toolchains Apple récentes

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| `CGDisplayStream` | Déprécié |
| `objc2-screencapturekit` | Moins mature pour SCStream prêt à l’emploi au moment du choix |
| `screencapturekit` ≥8 (Swift) | Échec de build (apple-metal / APIs macOS 26) sur l’environnement cible |
| Curseur baked-in | Conflit avec le double curseur collaboratif (P10) |
