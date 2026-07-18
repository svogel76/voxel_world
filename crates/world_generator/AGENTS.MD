# Agents.md — world_generator

Ergänzt die Regeln aus der Workspace-Root-`Agents.md`.

## Besonderheit dieses Crates

Anders als `tree_generator`/`grass_generator`/`rock_generator` darf dieses
Crate von den drei anderen Generator-Crates abhängen (Pfad-Dependencies im
Workspace). Es bleibt trotzdem **Bevy-frei** — keine Abhängigkeit auf
`bevy_voxel_world` oder `bevy` außerhalb von `dev-dependencies` für
`examples/`.

## Arbeitsweise

- Kleinteilig, eine Phase nach der anderen (siehe `README.md`, Phasen 1-5).
  Nach jeder Phase anhalten, erklären, auf Rückmeldung warten.
- Vor jeder neuen Phase kurz erklären, was sie tut und warum.
- Terrain-Höhe NUR über `TerrainHeightSource`-Trait, nie direkt über eine
  konkrete Terrain-Implementierung.
- Poisson-Disc-Sampling selbst implementieren (Bridson's Algorithmus),
  kein externes Crate dafür, außer explizit begründet (siehe
  Fundament/Domäne/3rd-Party-Kategorisierung in Root-`Agents.md`).
- Die `BlockType`-Vereinheitlichung (siehe README) erst in Phase 4 lösen,
  nicht vorher spekulativ.

## Tests — verbindlich pro Phase

- Für jede Phase Unit-Tests, die die Ein-/Ausgabe-Grenze abdecken.
- Für `TerrainHeightSource`-Tests: eine einfache Test-Implementierung
  (z.B. konstante Höhe oder simple Formel) in den Tests selbst definieren,
  kein Mocking-Crate nötig für ein derart simples Trait.
- Determinismus wie bei den anderen Generatoren: gleicher Seed ->
  identisches Ergebnis.
- `cargo test -p world_generator` muss nach jeder Phase grün sein.

## Struktur

- `src/lib.rs` — öffentliche API
- `src/terrain.rs` — `TerrainHeightSource`-Trait + Test-Implementierung
- `src/biome.rs` — Phase 1+2 (Zonen-Bestimmung, Parameter-Zuordnung)
- `src/placement.rs` — Phase 3 (Poisson-Disc, Hangsteigung)
- `src/chunk.rs` — Phase 4 (Orchestrierung, `ChunkContent`)
- `examples/visualize.rs` — Phase 5 Biom-/Chunk-Check (Bevy nur hier + unten)
- `examples/reference_scene.rs` — art-directed Hero Shot (feste Platzierung +
  Licht-Spike + Scale-Spike: Player-Proxy 1.8 m, Hero-Baum ~20–25 m);
  Kompositions-/Maßstabsregeln in README, **nicht** sofort in `generate_chunk`
  automatisieren / Presets global umstellen

Moderne Modulstruktur (keine `mod.rs`) gemäß Root-`Agents.md`.