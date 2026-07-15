# tree_generator

Prozedurale Baumerzeugung per L-System — Teil des Weltgenerator-Workspace
(siehe Root-`README.md` und `Roadmap_Voxel_Atmosphere.md`, Phase 1.5).

## Architekturprinzip

Dieses Crate ist **Bevy-frei**. Es enthält reine Rust-Logik: Parameter (Seed,
Regeln, Iterationstiefe) rein, eine Voxel-Datenstruktur raus. Bevy wird nur
als `dev-dependency` eingebunden, sichtbar ausschließlich für
`examples/visualize.rs` — die visuelle Validierung, das Äquivalent zur
Unity-Testszene.

## Abhängigkeiten — bewusst minimal

Nur `rand` (für Zufallsvariation: Winkel-Jitter, stochastische Regelwahl).
Alle anderen Bausteine (L-System-Grammatik, Turtle-Interpretation,
Voxelisierung) werden **selbst implementiert**, auch wenn fertige Crates
dafür existieren (z.B. `lindenmayer`, `line_drawing`) — bewusste Entscheidung,
um externe Abhängigkeiten nur dort einzugehen, wo die Komplexität es
rechtfertigt (wie bei Avian3D), nicht bei überschaubarer Logik.

## Entwicklungsphasen

Jede Phase hat eine klare Eingabe/Ausgabe-Grenze und wird durch Unit-Tests
abgedeckt, bevor die nächste beginnt.

### Phase 1 — L-System-Grammatik
Axiom + Produktionsregeln → Zeichenkette nach N Iterationen.
- Eingabe: Axiom (`&str`), Regeln (Symbol → Ersetzungsstring), Iterationstiefe
- Ausgabe: `String`
- Zunächst deterministisch (eine Regel pro Symbol), stochastische Varianten
  (mehrere mögliche Ersetzungen mit Wahrscheinlichkeiten) folgen in Phase 5
- Testbar rein textuell — keine Geometrie nötig

### Phase 2 — Turtle-Graphics-Interpretation
Zeichenkette → Liste von 3D-Liniensegmenten ("Skelett").
- Eingabe: die generierte Zeichenkette, Schrittlänge, Rotationswinkel
- Ausgabe: `Vec<Segment>` mit

```rust
struct Segment {
    start: Vec3,
    end: Vec3,
    thickness: f32,
    depth: u32,
}
```

- Verzweigungen über Stack (`[` = push, `]` = pop)

### Phase 3 — Voxelisierung
Liniensegmente → Voxel-Positionen.
- Eingabe: `Vec<Segment>`
- Ausgabe: `Vec<(IVec3, BlockType)>` (z.B. `BlockType::Wood`)
- Selbst geschriebener 3D-Line-Walk-Algorithmus, Dicke nach `thickness`/`depth`
- **Entscheidung nach visuellem Vergleich:** `CrossSectionShape::Cube` (Quadrat
  senkrecht zur dominanten Segmentachse) ist der Standard, nicht `Sphere`.
  Sphere erzeugte bei rotierten/geneigten Ästen sichtbare Lücken ("Planken"-
  Optik); Cube liefert einen durchgängigen, kompakten Stamm und passt besser
  zum kantigen Blockstil. `Sphere` bleibt im Code als Alternative erhalten.

### Phase 4 — Blätter/Laubkronen
Voxel-Cluster an den Astenden (Endpunkte ohne Folgesegment).
- Ausgabe erweitert die Voxel-Liste aus Phase 3 um `BlockType::Leaf`-Cluster

### Phase 5 — Parametrische Variation
- Zufalls-Jitter auf Winkel/Segmentlänge (via `rand`)
- Stochastische Regelwahl in der Grammatik (Phase 1 erweitern)

### Phase 6 — Visuelle Validierung
- `examples/visualize.rs`: minimale Bevy-App, spawnt die generierten Voxel
  als einfache Cubes
- Start: `cargo run -p tree_generator --example visualize`

## Öffentliche API (Zielbild)

```rust
pub fn generate(seed: u64, params: &TreeParams) -> Vec<(IVec3, BlockType)>;
```

Wird erst nach Phase 4 final feststehen — Zwischenstände sind bewusst
kleinere, testbare Funktionen pro Phase.

## Tests

Jede Phase bekommt eigene Unit-Tests, bevor die nächste Phase beginnt
(siehe `Agents.md` für die genaue Vorgehensweise).