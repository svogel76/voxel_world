# Agents.md — tree_generator

Ergänzt die Regeln aus der Workspace-Root-`Agents.md` um crate-spezifische
Vorgaben.

## Arbeitsweise

- **Kleinteilig, eine Phase nach der anderen** (siehe `README.md` für die
  Phasen 1–6). Nach jeder Phase anhalten, das Ergebnis erklären, auf
  Rückmeldung warten — nicht mehrere Phasen in einer Session durchziehen.
- **Vor jeder neuen Phase**: kurz erklären, was diese Phase konzeptionell
  tut und warum sie so geschnitten ist, bevor Code entsteht.
- **Fundament-Crates ohne Vorab-Diskussion:** `glam` (Vektor-/Gitter-Mathe),
  `rand` (Zufall). **Domänenlogik selbst implementieren:** L-System-Grammatik,
  Turtle-Interpretation, Voxelisierung — auch wenn fertige Crates existieren.
- **Weitere 3rd-Party-Crates vorher besprechen** und Pro/Contra abwägen
  (Aufwand, Risiko, Blackbox-Anteil, Alternativen).

## Tests — verbindlicher Bestandteil jeder Phase

- **Für jede Phase werden Unit-Tests geschrieben**, die die Ein-/Ausgabe-
  Grenze dieser Phase abdecken — nicht optional, nicht nachträglich.
- Reihenfolge pro Phase: Funktion/Struktur mit klarer Signatur anlegen →
  Testfälle mit bekannten, von Hand nachvollziehbaren Beispielen schreiben
  (z.B. "Axiom `F`, Regel `F→F[+F]F`, Tiefe 1 → erwarte String `F[+F]F`") →
  Implementierung, bis Tests grün sind.
- Tests decken auch **Randfälle** ab: leere Regeln, Tiefe 0, unbalancierte
  Klammern (sollte nicht vorkommen, aber Verhalten sollte definiert sein),
  Segmente mit Länge 0 in der Voxelisierung.
- Tests liegen in `#[cfg(test)] mod tests` direkt in der jeweiligen Datei
  (Standard-Rust-Konvention), nicht in einer separaten `tests/`-Datei,
  solange es sich um reine Unit-Tests einzelner Funktionen handelt.
- `cargo test -p tree_generator` muss nach jeder Phase grün sein, bevor
  die nächste Phase beginnt.

## Struktur

- `src/lib.rs` — öffentliche API, Modul-Deklarationen
- `src/grammar.rs` — Phase 1 (L-System)
- `src/turtle.rs` — Phase 2 (Interpretation → Skeleton)
- `src/voxelize.rs` — Phase 3+4 (Voxelisierung, Blätter)
- `src/types.rs` — gemeinsame Typen (`Segment`, `BlockType`, `TreeParams`)
- `examples/visualize.rs` — Phase 6, einzige Stelle mit Bevy-Bezug

Moderne Modulstruktur (keine `mod.rs`) gemäß Root-`Agents.md`.