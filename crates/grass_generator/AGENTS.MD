# Agents.md — grass_generator

Ergänzt die Regeln aus der Workspace-Root-`Agents.md` und orientiert sich
an `crates/tree_generator/Agents.md` (gleiches Vorgehen, andere Domäne).

## Arbeitsweise

- Kleinteilig, eine Phase nach der anderen (siehe `README.md`, Phasen 1-3).
  Nach jeder Phase anhalten, erklären, auf Rückmeldung warten.
- Vor jeder neuen Phase kurz erklären, was sie tut und warum, bevor Code
  entsteht.
- Keine externen Crates außer `rand` und `glam` (siehe Root-`Agents.md`,
  Kategorie "Fundament").
- **Wichtig, anders als bei `tree_generator`:** Dieses Crate erzeugt KEINE
  Voxel-Positionen. Ausgabe sind Platzierungsdaten (`GrassInstance`), keine
  `(IVec3, BlockType)`-Listen. Die Quad-Geometrie entsteht erst in der
  Bevy-Integrationsschicht (`examples/visualize.rs` bzw. später
  `voxel_game`).

## Tests — verbindlich pro Phase

- Für jede Phase Unit-Tests, die die Ein-/Ausgabe-Grenze abdecken.
- Determinismus testen wie in `tree_generator` Phase 5: gleicher Seed ->
  identisches Ergebnis; unterschiedliche Seeds -> unterschiedliches Ergebnis.
- Randfälle: Dichte 0 -> leere Ausgabe; sehr kleine Fläche -> plausible,
  nicht negative Anzahl an Instanzen.
- `cargo test -p grass_generator` muss nach jeder Phase grün sein.

## Struktur

- `src/lib.rs` — öffentliche API
- `src/placement.rs` — Phase 1 (Dichte-basierte Positionen)
- `src/instance.rs` — Phase 2 (`GrassInstance`, `GrassVariant`, Variation)
- `examples/visualize.rs` — Phase 3, einzige Stelle mit Bevy-Bezug

Moderne Modulstruktur (keine `mod.rs`) gemäß Root-`Agents.md`.