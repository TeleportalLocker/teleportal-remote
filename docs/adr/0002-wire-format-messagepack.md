# ADR 0002 — Format filaire MessagePack length-prefixed

- **Statut** : Accepté
- **Date** : 2026-09-23
- **Phase** : 2

## Contexte

Le document produit illustre `CursorMove` en JSON. Le protocole doit aussi transporter des `VideoFrame` binaires (1080p30 H264) avec faible latence, tout en restant simple à debugger et versionner.

## Décision

Adopter :

1. **En-tête** : `u32` big-endian = longueur du payload
2. **Payload** : MessagePack via `rmp-serde` (`to_vec_named` / `from_slice`)
3. **Schéma Rust** : enum `Message` serde-tagged (`type`)

JSON reste un format de documentation / exemples humains, pas le format filaire.

## Conséquences

### Positives

- Payload vidéo efficace (bytes natifs via `serde_bytes`)
- Framing explicite pour streaming et buffers
- Un seul codec pour signaling et média de contrôle

### Négatives

- Moins lisible que JSON dans un proxy HTTP brut (acceptable : WS binaire en Phase 3)
- Évolution de schéma à gérer via `PROTOCOL_VERSION`

## Alternatives écartées

| Alternative | Pourquoi non |
|-------------|--------------|
| JSON pur | Coûteux et peu adapté aux frames vidéo |
| Protobuf | Plus lourd en outillage pour le MVP |
| bincode | Moins portable / versionnable entre langages futurs |
