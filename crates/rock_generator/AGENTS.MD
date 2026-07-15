# Agents.md — rock_generator

Ergänzt die Regeln aus der Workspace-Root-`Agents.md`, orientiert an
`crates/tree_generator/Agents.md`.

## Scope

NUR kleine, verstreute Boulder. Keine Monumente/Strukturen mit
eingeritzten Symbolen — das ist explizit außerhalb des Scopes dieses
Crates (siehe README).

## Arbeitsweise

- Kleinteilig, eine Phase nach der anderen (siehe `README.md`, Phasen 1-3).
  Nach jeder Phase anhalten, erklären, auf Rückmeldung warten.
- Vor jeder neuen Phase kurz erklären, was sie tut und warum.
- Keine externen Crates außer `rand`/`glam` ("Fundament"). Noise-Logik wird
  selbst implementiert (siehe README, "Offene Design-Frage") — falls ein
  Noise-Crate erwogen wird, das explizit vorschlagen und begründen, nicht
  stillschweigend einbinden.
- Ausgabe ist `Vec<(IVec3, BlockType)>` wie bei `tree_generator` (nicht
  Platzierungsdaten wie bei `grass_generator`).

## Tests — verbindlich pro Phase

- Für jede Phase Unit-Tests, die die Ein-/Ausgabe-Grenze abdecken.
- Determinismus: gleicher Seed -> identische Form; unterschiedliche Seeds
  -> unterschiedliche Form.
- Randfälle: sehr kleine Bounding-Box, Schwellenwert außerhalb des
  Noise-Wertebereichs (-> leere oder volle Ausgabe, Verhalten definieren
  und testen).
- `cargo test -p rock_generator` muss nach jeder Phase grün sein.

## Struktur

- `src/lib.rs` — öffentliche API
- `src/noise.rs` — selbstgeschriebenes 3D-Noise-Feld
- `src/shape.rs` — Phase 1 (Schwellenwert-Sampling in Voxel-Positionen)
- `src/variation.rs` — Phase 2 (Größen-/Formvariation)
- `examples/visualize.rs` — Phase 3, einzige Stelle mit Bevy-Bezug

Moderne Modulstruktur (keine `mod.rs`) gemäß Root-`Agents.md`.