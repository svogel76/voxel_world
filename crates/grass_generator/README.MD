# grass_generator

Prozedurale Platzierung von Gras/Farn-Büscheln — Teil des Weltgenerator-
Workspace (siehe Root-`README.md` und `Roadmap_Voxel_Atmosphere.md`, Phase 1.5).

## Architekturprinzip

Wie `tree_generator`: **Bevy-frei**. Anders als `tree_generator` liefert
dieses Crate aber **keine Voxel-Positionen**, sondern reine
**Platzierungsdaten** — die eigentliche Geometrie (Kreuzquads) entsteht
erst in der späteren Bevy-Integrationsschicht.

## Entscheidung: statische Kreuzquads, kein echtes Billboarding

Zwei senkrecht zueinander stehende Ebenen pro Büschel, mit zufälliger
Rotation um die Y-Achse — wie in Minecraft. **Kein** kamera-ausgerichtetes
Echtzeit-Billboarding (das würde Pro-Frame-Rotation in der Bevy-Schicht
brauchen und ist für dieses Ziel unnötiger Aufwand). Begründung siehe
Roadmap-Eintrag "Vegetationsgenerator": Performance bei hoher Dichte,
Atmosphäre kommt aus Licht/Textur statt Geometrie.

## Abhängigkeiten

Nur `rand` (Zufallsrotation, Zufallsskalierung, Varianten-Auswahl) —
gleiche Linie wie `tree_generator`: möglichst wenig externe Crates,
`glam` für Vektortypen ist als "Fundament" erlaubt (siehe Root-`Agents.md`).

## Entscheidung: eigenes RNG pro Pipeline-Phase

`place_positions` und `build_instances` seeden jeweils eine **eigene**
`StdRng` aus dem übergebenen `seed` (`StdRng::seed_from_u64(seed)`).
Anders als [`tree_generator::generate`], das **einen** `&mut StdRng` durch
die gesamte Pipeline fädelt (L-System → Turtle → Voxelize).

**Bewusst so gewählt:**

- Phasen sind unabhängig testbar und optional einzeln nutzbar — z. B.
  `build_instances` auf von außen gelieferten Positionen, ohne Phase 1.
- Änderungen an Phase 1 (Dichte, Sampling-Logik) verschieben nicht den
  Zufallsstrom von Phase 2: Instanz *i* bekommt bei gleichem `seed` immer
  dieselbe Rotation/Skalierung/Variante, unabhängig davon, wie viele
  Positionen Phase 1 insgesamt erzeugt.
- `world_generator` kann Phasen getrennt aufrufen, filtern oder später
  nur eine Phase ersetzen, ohne die andere neu zu seeden.

Eine zukünftige Top-Level-Funktion `generate(seed, area, params)` kann bei
Bedarf intern **einen** RNG durchreichen. Die phasenweisen APIs behalten ihr
eigenes Re-Seeding für Modularität.

**`generate()`:** übergibt denselben `seed` an beide Phasen (keine abgeleiteten
Sub-Seeds). Jede Phase startet trotzdem einen eigenen `StdRng::seed_from_u64(seed)` —
das Ergebnis ist bit-identisch zu manuellem Aufruf von `place_positions` und
`build_instances` mit gleichem Seed. „Gleicher Seed + gleiche Params → gleiches
Ergebnis“ gilt damit für `generate()` als Ganzes.

## Entwicklungsphasen

### Phase 1 — Dichte-basierte Platzierung
Positionen ohne Geometrie, gesteuert durch ein Dichte-Signal (Noise).
- Eingabe: Fläche/Bereich (z.B. Chunk-Grenzen als min/max X,Z), ein
  Dichte-Wert oder eine Dichte-Funktion (Noise kommt später aus
  `world_generator`/Biom — hier zunächst als einfacher `f32`-Parameter),
  `seed: u64`
- Ausgabe: `Vec<Vec3>` (nur Positionen, Y noch nicht an Terrain-Höhe
  gekoppelt — das ist Aufgabe der späteren Platzierungslogik/Integration)
- Zufalls-Sampling (kein Poisson-Disc-Mindestabstand nötig — Gras darf dichter
  stehen als Bäume, Überlappung ist bei Kreuzquads visuell unproblematisch)

### Phase 2 — Instanz-Variation
- Eingabe: die Positionen aus Phase 1, `seed: u64`
- Ausgabe: `Vec<GrassInstance>` mit

```rust
struct GrassInstance {
    position: Vec3,
    rotation_y: f32,   // Kreuzquad-Ausrichtung, zufällig gestreut
    scale: f32,        // leichte Größenvariation
    variant: GrassVariant,
}
```

- `GrassVariant`: einfaches Enum (z.B. `Grass`, `Fern`) für spätere
  Textur-/Farbvarianten — Auswahl per Zufall, gewichtet wie bei
  `ProductionRule` in `tree_generator` möglich, aber erstmal einfache
  Gleichverteilung ausreichend
- Determinismus wie in `tree_generator` Phase 5: `StdRng::seed_from_u64(seed)`
  explizit durchgereicht, kein `thread_rng()`

### Phase 3 — Visuelle Validierung
- `examples/visualize.rs`: minimale Bevy-App, baut aus jeder `GrassInstance`
  zwei senkrechte Quads (einfache Vertexfarbe, keine Textur — Texturen sind
  erst Phase 3 der Haupt-Roadmap)
- Start: `cargo run -p grass_generator --example visualize`

## Öffentliche API

```rust
pub fn generate(seed: u64, area: Area, params: &GrassParams) -> Vec<GrassInstance>;
```

`GrassParams` bündelt Dichte, Skalierungsbereich und Varianten-Gewichte.
Preset: `GrassParams::default()` (`density: 1.5`, `scale_range: (0.8, 1.2)`,
gleichgewichtete `Grass`/`Fern`-Varianten).

Einzelbausteine (`place_positions`, `build_instances`, `Area`, …) bleiben öffentlich
für Experimente, Tests und getrennte Pipeline-Schritte in `world_generator`.

Start: `cargo run -p grass_generator --example visualize`

## Tests

Jede Phase bekommt eigene Unit-Tests, bevor die nächste beginnt (siehe
`Agents.md`).