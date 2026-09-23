# Architecture — Phase 15 : Curseur collaboratif, multi-écran & fin de session

## Objectif

Passer le MVP en **curseur collaboratif pur** (sans injection souris/clavier Host), aligner capture / overlay / coords sur **un** display, et terminer proprement la session quand le pair quitte.

## Découpage

| Volet | Livrable |
|-------|----------|
| Collab cursor | Guest → `CursorMove` uniquement ; Host overlay ; pas d’injection `Mouse*` / `KeyEvent` |
| Multi-écran | Géométrie partagée `SessionDisplay` (origine + taille) pour capture, overlay et normalize |
| Fin de session | Tout `PeerLeft` → `Idle` + leave WS + stop capture/overlay (Host et Guest) |
| Docs | ADR 0017, diagramme, checklist ; roadmap Audit → Phase 16 |

## Flux curseur

```text
Guest pointer → CursorMove → Relay → Host CursorSync → overlay marker
Host OS cursor → poll_os_cursor(display rect) → CursorMove → Relay → Guest marker
```

Aucune injection OS (`PlatformInjector` retiré du client desktop en session).

## Géométrie display

- `resolve_session_display()` : primaire (origine `0,0`) sinon premier écran listé par capture.
- Capture : `CaptureConfig.display_index` = index de ce display.
- Overlay Host : bounds logiques = même origine/taille (échelle moniteur primaire Tauri).
- Normalize Host : `poll_os_cursor(origin, size)` ; hors rectangle → pas d’envoi.

## Fin de session

`apply_signal(…, PeerLeft)` → `Idle`. La boucle session leave WS, stop capture, émet `Idle` ; l’UI revient à l’accueil ; le slot `ActiveSession` est libéré.

## Hors scope

- Toggle « prise de contrôle » distant
- Multi-capture simultanée
- Notarisation / Authenticode
- Audit final (Phase 16)
