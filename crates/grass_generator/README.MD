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

## Öffentliche API (Zielbild)

```rust
pub fn generate(seed: u64, area: Area, params: &GrassParams) -> Vec<GrassInstance>;
```

## Tests

Jede Phase bekommt eigene Unit-Tests, bevor die nächste beginnt (siehe
`Agents.md`).