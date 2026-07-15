# voxel-world — Weltgenerator-Workspace

Dieser Cargo Workspace enthält die Sub-Generatoren, die zusammen den **Weltgenerator**
für das atmosphärische Voxel-Waldprojekt bilden (Phase 1.5 der
[Roadmap](./Roadmap_Voxel_Atmosphere.md)).

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

## Architekturprinzip: Bevy-freie Generatoren

Jeder Generator-Crate ist bewusst **reine Rust-Logik ohne Bevy-Abhängigkeit** unter
`[dependencies]`. Ein Generator nimmt Parameter (Seed, Regeln, Dichte, …) entgegen und
gibt eine einfache Datenstruktur zurück (z.B. eine Liste von Voxel-Positionen) — nichts
davon kennt Bevys ECS, `App`, oder Rendering.

Das ermöglicht:
- **Isolierte Tests**: `cargo test -p tree_generator` ohne die komplette Spiel-App zu starten
- **Fokussierte Entwicklung**: An genau einem System arbeiten, ohne den Rest im Kopf zu halten

Bevy wird pro Crate nur als `dev-dependency` eingebunden, sichtbar ausschließlich für
`examples/`. Jeder Generator bekommt so ein eigenes `examples/visualize.rs` — das
Äquivalent zur Unity-Testszene, gestartet mit:

```
cargo run -p tree_generator --example visualize
```

Erst ein späteres Haupt-Spiel-Crate (`voxel_game`, kommt in einer späteren Phase dazu)
übersetzt die von den Generatoren gelieferten Daten in echte Bevy-Entities.

## Struktur

```
voxel-world/
├── Cargo.toml              Workspace-Definition, gemeinsame Dependency-Versionen
├── README.md                diese Datei
├── Agents.md                 Arbeitsregeln für KI-Assistenten in diesem Projekt
└── crates/
    ├── tree_generator/
    ├── grass_generator/
    ├── rock_generator/
    └── world_generator/
```

## Status

Aktuell im Aufbau. Reihenfolge laut Roadmap: zuerst `tree_generator` (L-System-Grundlagen
isoliert, dann Turtle-Interpretation, dann Voxelisierung), danach die einfacheren
Generatoren, zuletzt `world_generator` als Orchestrator.