# Checklist de validation — Phase 15

À cocher avant validation explicite et passage à la Phase 16 (Audit final).

## Livrables

- [x] Architecture : `docs/architecture/phase-15-collab-multiscreen.md`
- [x] ADR : `docs/adr/0017-collab-cursor-session-display.md`
- [x] Diagramme : `docs/diagrams/phase-15-collab-multiscreen.mmd`
- [x] Guest : `CursorMove` uniquement (pas de `Mouse*` / `KeyEvent`)
- [x] Host : pas d’injection ; overlay collaboratif
- [x] Géométrie partagée capture / overlay / normalize
- [x] `PeerLeft` → `Idle` + fermeture WS
- [x] Roadmap P15 / P16 + README
- [x] Validation Phase 14 cochée (hors report certs)
- [x] Cette checklist

## Fonctionnel

- [x] Curseur Guest visible en overlay Host sans déplacer la souris OS
- [x] Curseur Host visible côté Guest (poll relatif au display capturé)
- [x] Hors display capturé → pas d’envoi Host
- [x] Host quitte → Guest revient Accueil
- [x] Guest quitte → Host revient Accueil

## Qualité

- [x] `cargo fmt --all -- --check`
- [x] `cargo clippy --workspace --all-targets -- -D warnings`
- [x] `cargo test --workspace`
- [x] `npm run build` (desktop-client)

## Reporté

- [ ] Toggle prise de contrôle distant
- [ ] Multi-capture simultanée
- [ ] Audit final — **Phase 16**

## Validation

- [ ] Revue humaine / validation explicite pour ouvrir la **Phase 16 — Audit final**
