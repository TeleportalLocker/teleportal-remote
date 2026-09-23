# Architecture — Phase 11 : Overlay collaboratif

## Objectif

Afficher le curseur distant du Guest **sur le bureau Host** via une fenêtre transparente always-on-top click-through, sans remplacer le curseur OS local. Réutilise le sync `CursorMove` / `remote-cursor` de la Phase 10.

## Flux

```text
Guest pointer → CursorMove → Relay → Host CursorSync
  → emit remote-cursor
  → fenêtre cursor-overlay (marqueur HTML)
```

## Fenêtre `cursor-overlay`

| Propriété | Valeur |
|-----------|--------|
| Transparent | oui (`macOSPrivateApi`) |
| Décorations | non |
| Always on top | oui |
| Click-through | `set_ignore_cursor_events(true)` |
| Bounds | moniteur primaire |
| Contenu | `index.html#overlay` |

Cycle de vie : ouverte quand Host `InSession` + pair ; fermée sinon.

## Client

- Guest : marqueur HTML sur canvas (inchangé P10)
- Host : plus de marqueur in-app ; overlay bureau uniquement

## Hors scope

Annotations, labels, multi-écrans, overlay système Guest, Linux spécifique.
