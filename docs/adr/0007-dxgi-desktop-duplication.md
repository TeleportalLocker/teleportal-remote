# ADR 0007 — DXGI Desktop Duplication + frames BGRA8

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 5

## Contexte

Le MVP Windows doit capturer l’écran avec faible latence. L’encodage H264 arrive en Phase 7 ; il faut un format CPU stable pour alimenter l’encodeur.

## Décision

1. API Windows : **DXGI Desktop Duplication** + D3D11 via le crate `windows`
2. Format livré : **BGRA8** en mémoire CPU (copie staging)
3. Trait `Capturer` partagé pour préparer ScreenCaptureKit (Phase 6)
4. Un écran à la fois (`display_index`), enumération multi prête
5. Ne pas baker le curseur dans la frame

## Conséquences

### Positives

- API Microsoft recommandée pour la duplication desktop
- Abstraction prête pour macOS
- Pipeline encode distinct (P7)

### Négatives

- Coût CPU de la copie staging (acceptable avant encode GPU dédié)
- Tests DXGI uniquement sur runners Windows

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| GDI BitBlt | Plus lent, moins fiable |
| Windows.Graphics.Capture | Plus récent mais permissions/UI plus complexes pour le MVP |
| NV12 GPU zero-copy dès P5 | Couplage prématuré avec l’encodeur |
