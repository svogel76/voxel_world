# voxel-world — Weltgenerator-Workspace

Dieser Cargo Workspace enthält die Sub-Generatoren und das Phase-1-Spielcrate für das
atmosphärische Voxel-Waldprojekt (siehe [Roadmap](./docs/Roadmap_Voxel_Atmosphere.md)).

## Konzept

Der Weltgenerator bestimmt pro Chunk zuerst das **Biom** (aus Höhen- und Feuchtigkeits-Noise:
Wald, Fels, Lichtung, …). Jedes Biom hat eigene Parameter (Baumdichte, welche Baumarten,
Grasdichte, Steindichte, …). Diese Parameter werden an die jeweiligen Sub-Generatoren
weitergereicht:

- **tree_generator** — prozedurale Bäume per L-System (Grammatik → Turtle-Graphics →
  Voxelisierung → Blätter → parametrische Variation)
- **grass_generator** — Farne, Gras (einfachere prozedurale Form, kein volles L-System)
- **rock_generator** — Voxel-Cluster-Steine mit Größen-/Rotationsvariation
- **world_generator** — orchestriert die obigen, kennt die Biome, entscheidet *was* und
  *wie viel*; die einzelnen Generatoren entscheiden nur noch *wie* im Detail
- **voxel_game** — Bevy-App: `bevy_voxel_world`-Terrain, Height-Adapter, ein
  `generate_chunk`-Spawn und Avian-Capsule (Phase 1)

## Architekturprinzip: Bevy-freie Generatoren

Jeder Generator-Crate ist bewusst **reine Rust-Logik ohne Bevy-Abhängigkeit** unter
`[dependencies]`. Ein Generator nimmt Parameter (Seed, Regeln, Dichte, …) entgegen und
gibt eine einfache Datenstruktur zurück (z.B. eine Liste von Voxel-Positionen) — nichts
davon kennt Bevys ECS, `App`, oder Rendering.

Das ermöglicht:
- **Isolierte Tests**: `cargo test -p tree_generator` ohne die komplette Spiel-App zu starten
- **Fokussierte Entwicklung**: An genau einem System arbeiten, ohne den Rest im Kopf zu halten

Bevy wird pro Generator-Crate nur als `dev-dependency` eingebunden, sichtbar ausschließlich
für `examples/`. Ausnahme: `voxel_game` darf Bevy / `bevy_voxel_world` / `avian3d` unter
`[dependencies]` haben — dort findet die Integration statt.

```
cargo run -p tree_generator --example visualize
cargo run -p voxel_game
```

## Struktur

```
voxel-world/
├── Cargo.toml              Workspace-Definition, gemeinsame Dependency-Versionen
├── README.md                diese Datei
├── STATUS.md                aktueller Stand + nächste Schritte
├── Agents.md                 Arbeitsregeln für KI-Assistenten in diesem Projekt
├── docs/
│   └── Roadmap_Voxel_Atmosphere.md
└── crates/
    ├── tree_generator/
    ├── grass_generator/
    ├── rock_generator/
    ├── world_generator/
    └── voxel_game/         Phase-1 Bevy-Integration
```

## Status

Aktueller Stand und nächste Schritte: **[`STATUS.md`](./STATUS.md)**.

Kurz: Phase 1 / 1.5, Phase-2-Licht und Forest-Scale-Presets sind umgesetzt.
Als Nächstes: lokal Scale gegenprüfen, Unterholz-Regeln, dann Texturen (Phase 3).
