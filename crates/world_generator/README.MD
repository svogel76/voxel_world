# world_generator

Orchestrator, der `tree_generator`, `grass_generator` und `rock_generator`
pro Biom/Chunk zusammenführt — Teil des Weltgenerator-Workspace (siehe
Root-`README.md` und `Roadmap_Voxel_Atmosphere.md`, Phase 1.5).

## Architekturprinzip

Wie die anderen Generatoren: **Bevy-frei**. Anders als diese hängt
`world_generator` aber zusätzlich von den drei Sub-Generator-Crates ab
(Pfad-Dependencies im Workspace), nicht nur von `rand`/`glam`.

## Entscheidung: Terrain-Höhe über austauschbares Trait

`world_generator` hängt NICHT direkt von `bevy_voxel_world` ab (bliebe
sonst nicht Bevy-frei, und das Terrain-Projekt aus Phase 1 der
Haupt-Roadmap existiert zum jetzigen Zeitpunkt ohnehin noch nicht als
fertiges Projekt). Stattdessen:

```rust
pub trait TerrainHeightSource {
    fn height_at(&self, x: f32, z: f32) -> f32;
}
```

Für Entwicklung/Tests: eine einfache Test-Implementierung (z.B. eigene
Platzhalter-Noise-Funktion, ähnlich `rock_generator::noise`). Die echte
Anbindung an `bevy_voxel_world` entsteht später im `voxel_game`-Crate.

## Bekannte offene Frage für spätere Phasen: `BlockType`-Vereinheitlichung

`tree_generator` und `rock_generator` haben jeweils ihr **eigenes**
`BlockType`-Enum (`Wood`/`Leaf` bzw. `Stone`) — das war eine bewusste
Konsequenz der Crate-Isolation. Sobald `world_generator` die Ausgaben
mehrerer Sub-Generatoren kombiniert (Phase 4), muss diese Fragmentierung
gelöst werden (z.B. ein vereinheitlichendes `WorldBlockType`-Enum in
`world_generator`, das die einzelnen Sub-Typen umwandelt/wrapped). Wird
nicht vorab gelöst, sondern erst wenn Phase 4 tatsächlich ansteht.

## Abhängigkeiten

`rand`, `glam` (Fundament), plus Pfad-Dependencies auf `tree_generator`,
`grass_generator`, `rock_generator`.

## Entwicklungsphasen

### Phase 1 — Biom-/Zonen-Bestimmung
- Eingabe: Position `(x, z)`, `seed: u64`, `&impl TerrainHeightSource`
- Zweiter Noise-Layer (Feuchtigkeit), selbst implementiert (2D, analog zu
  `rock_generator::noise`, aber eigenständig — kein Cross-Crate-Import
  interner Module)
- `Biome`-Enum (mindestens `Forest`, `Rocky`, `Clearing`), abgeleitet aus
  Höhe (von `TerrainHeightSource`) + Feuchtigkeit kombiniert
- Ausgabe: `Biome` für eine gegebene Position

### Phase 2 — Biom-Parameter-Zuordnung
- Jedes `Biome` bekommt eigene Parameter: welche `TreeParams`/`GrassParams`/
  `RockParams`-Presets, welche Dichten
- Einfache statische Zuordnung (`match biome { ... }`) reicht zunächst

### Phase 3 — Platzierungslogik
- Poisson-Disc-Sampling für Bäume (Mindestabstand, selbst implementiert —
  Bridson's Algorithmus, überschaubare Komplexität, siehe Kategorisierung
  in Root-`Agents.md`)
- Hangsteigung aus `TerrainHeightSource` ableiten (Höhendifferenz
  benachbarter Punkte) für Stein-Verteilung (mehr Steine an Hängen)
- Gras nutzt weiterhin die eigene Dichte-Logik aus `grass_generator`,
  keine Änderung nötig

#### Entscheidung: `tree_density` → Poisson-Mindestabstand `r`

`BiomeParams::tree_density` ist **Punkte pro Flächeneinheit** auf der XZ-Ebene
(gleiche Einheit wie `GrassParams::density`). Der Bridson-Mindestabstand
zwischen zwei Baumzentren ist:

```text
r = 1 / sqrt(tree_density)
```

Begründung: bei Dichte `d` gehört zu jedem Punkt im Mittel die Fläche
`1/d`; die charakteristische Seitenlänge dieser Fläche ist `1/sqrt(d)` und
wird als Mindestabstand verwendet (übliche Näherung; hexagonales Packen
läge nur um einen kleinen Faktor daneben).

Beispiele aus Phase-2-Richtwerten:

| Biom     | `tree_density` | `r ≈` |
|----------|----------------|-------|
| Forest   | 0.08           | 3.54  |
| Clearing | 0.012          | 9.13  |
| Rocky    | 0.005          | 14.14 |

`Area` kommt aus `grass_generator` (wiederverwendet, keine zweite Bounds-
Struct) — Konvention `Vec2 = (world_x, world_z)`.

#### Hangsteigung → `rock_density`-Multiplikator

Steigung = Betrag des Höhengradienten (zentrale Differenzen). Multiplikator
`min(1 + 0.5 · slope, 3.0)`: flach → `1.0`, steile Hänge bis höchstens
dreifache Steindichte. Die Multiplikation mit `rock_density` erfolgt in
Phase 4 bei der Orchestrierung.

### Phase 4 — Orchestrierung: `generate_chunk()`
- Kombiniert Biom-Bestimmung + Platzierungslogik + Aufrufe an alle drei
  Sub-Generatoren für einen gegebenen Bereich
- Löst die `BlockType`-Vereinheitlichung (siehe oben)
- Ausgabe: eine kombinierte Struktur mit Voxel-Daten (Bäume/Steine) und
  Platzierungsdaten (Gras), in Weltkoordinaten (unter Nutzung von
  `TerrainHeightSource` für die Y-Position)

### Phase 5 — Visuelle Validierung
- `examples/visualize.rs`: nutzt eine einfache Test-`TerrainHeightSource`
  (kein echtes `bevy_voxel_world`), zeigt einen kompletten generierten
  Bereich mit Bäumen, Gras und Steinen
- Start: `cargo run -p world_generator --example visualize`

## Öffentliche API (Zielbild)

```rust
pub fn generate_chunk(
    seed: u64,
    area: Area,
    terrain: &impl TerrainHeightSource,
) -> ChunkContent;
```

## Tests

Jede Phase bekommt eigene Unit-Tests, bevor die nächste beginnt (siehe
`Agents.md`).