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

## Entscheidung: `WorldBlockType` (Phase 4)

`tree_generator` und `rock_generator` behalten jeweils ihr **eigenes**
`BlockType`-Enum (`Wood`/`Leaf` bzw. `Stone`) — bewusste Crate-Isolation.
`world_generator` führt sie erst bei der Orchestrierung zusammen:

```rust
pub enum WorldBlockType { Wood, Leaf, Stone }
// From<tree_generator::BlockType> / From<rock_generator::BlockType>
```

Kein gemeinsames Enum in den Sub-Crates; Mapping nur im Orchestrator.

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

Forest-`RockParams` (nach Sanity-Check gegen Leer-Boulder):

| Feld | Wert | Begründung |
|------|------|------------|
| `half_extent` | **4** | mehr Volumen (9³=729 statt 7³=343) |
| `threshold` | **0.4** | mehr Noise-Zellen überschreiten die Schwelle |
| `radial_falloff` | **0.35** | etwas weicherer Rand, weniger Kern-Kollaps |
| übrige Felder | Defaults (`frequency=0.35`, Achsen `0.7..=1.3`) | |

Defaults (`half_extent=3`, `threshold=0.5`, `falloff=0.45`) erzeugten oft
0–3 Voxel nach dem Connectivity-Filter — unsichtbare „Steine“.

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
- `WorldBlockType` + `ChunkContent` (siehe oben)
- Ausgabe: Voxel (Bäume/Steine) und Gras-Instanzen in Weltkoordinaten
  (`TerrainHeightSource` setzt Y)

#### Bekannte Einschränkung: genau ein Biom pro Area

`generate_chunk` bestimmt das Biom **nur am Mittelpunkt** der Area
(`classify`) und wendet die zugehörigen `BiomeParams` auf die gesamte Area
an. Keine Biom-Übergänge oder -Mischung innerhalb einer Area. Später
auflösbar durch kleinere Sub-Areas mit jeweils eigenem Biom — nicht Teil
dieses ersten Durchstichs.

#### Seed-Ableitung pro Baum/Stein

```text
feature_seed(world_seed, kind, x, z)
  = mix(world_seed, kind, round(x·1000), round(z·1000))
```

`kind` trennt Baum- und Stein-Ströme; Milli-Quantisierung hält Float-
Positionen stabil. Gleicher World-Seed + gleiche Position → gleiche Form.

### Phase 5 — Visuelle Validierung
- `examples/visualize.rs`: drei **10×10**-Areas nebeneinander (Forest /
  Rocky / Clearing), kein echtes `bevy_voxel_world`
- Forest & Clearing: `SimpleNoiseTerrain` (Höhenvariation); Rocky:
  `ConstantHeight(12)` für zuverlässige Biom-Klassifikation
- Seeds am lokalen Zentrum `(5,5)`: Forest=`0`, Clearing=`1`, Rocky=beliebig
  bei hoher Höhe
- Pro Panel: **Heightfield-Boden** (Gitter-Mesh, Y aus `height_at()`), damit
  Objekt-Boden-Kontakt prüfbar ist — Kamera startet seitlich auf Forest
- Optional Screenshot: `WORLD_GENERATOR_SCREENSHOT=path.png cargo run -p
  world_generator --example visualize`
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