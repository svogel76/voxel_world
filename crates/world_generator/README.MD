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

| Biom     | `tree_density` | `r ≈` | Hinweis |
|----------|----------------|-------|---------|
| Forest   | 0.02           | 7.07  | World-Scale-Bäume (`step=2.0`, Stamm `4.0`) |
| Clearing | 0.012          | 9.13  | kleinere 2D-Bäume |
| Rocky    | 0.005          | 14.14 | sparse scrub |

Früher Forest `0.08` (`r≈3.5`) war für Lab-Bäume (`generic_3d`, ~8–10 m)
gedacht — zu eng für 15–30 m Kronen.

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

### Reference Scene — art-directed hero shot (vor Phase-2-Feinschliff)

`examples/reference_scene.rs` ist **kein** Chunk-/Biom-Test, sondern eine
bewusst komponierte Demo gegen `docs/Blocky_forest.png`.

#### Maßstabskonvention (Weltziel)

| Größe | Wert | Bedeutung |
|-------|------|-----------|
| 1 Voxel-Kante | **≈ 1 m** | klassische Blockgröße, kein Micro-Voxel |
| Spieler | **≈ 1,8 m** (Augenhöhe ~1,6 m) | Maßstabsanker |
| Wald-Bäume | **15–30 m** | deutsche Waldhöhe; Spieler soll sich klein fühlen |
| Stammdicke | **~2–5 m** Basis | sonst wirken hohe Bäume wie Streichhölzer |

Stil bleibt blockig (wie Concept Art); die **Proportion** darf realistischer
sein als im PNG (dort eher kompakte Illustrations-Bäume). Forest-`TreeParams`
in `generate_chunk` nutzen seit dem Scale-Meilenstein die Frame-Werte aus dieser
Demo (`step_length: 2.0`, `base_thickness: 4.0`); der Hero-Spike (`2.5` / `5.0`)
bleibt nur hier in der Reference Scene.

- **Semi-manuell:** feste Seeds + feste `(x, z)`-Positionen rufen
  `tree_generator::generate` / `rock_generator::generate` /
  `grass_generator::generate` direkt auf — **nicht** `generate_chunk()`
- **Komposition (was der Generator später lernen soll):**
  - dichter Stamm-Rahmen links/rechts; ein **Hero-Baum ~20–25 m** als Höhenanker
  - offene Blickachse in der Mitte (kein Gras/keine Bäume auf dem Pfad)
  - **Player-Proxy** (~0,6×1,8×0,6) am Pfadrand — kein Character-Controller
  - Kamera auf **Augenhöhe (~1,6 m)** im Korridor
  - sparse Far trees / Kronenlücken für Lichtstrahlen
  - **Unterholz:** Demo bleibt art-directed; prozedural in `understory.rs` /
    `generate_chunk` (Forest): fern-lastiger Boden, Stammfuß-Farne, Leaf-Büsche,
    gelegentliche Fallstämme. Moos-Tint und Pfad-Komposition nur in dieser Demo.
  - Steine als Seitenanker
- **Licht (Phase-2-Spike an dieser Szene):** dunkles `GlobalAmbientLight`,
  DirectionalLight + Cascaded Shadow Maps + `VolumetricLight`/`VolumetricFog`/
  `FogVolume`, dezentes Bloom, SSAO (+ TAA)
- **Bewusst nicht in der Demo:** spielbare Figur, Wasserlauf, Runenstein/Ruinen,
  echte `bevy_voxel_world`-Meshes, PBR-Texturen; keine globale Preset-Umstellung;
  kein eigener `bush_generator` / Debris-Crate (Leaf-Büsche + Logs sind inline
  in `understory.rs`; eigenes Crate nur bei Bedarf)
- Screenshots: `docs/reference_scene_scale.png`, `docs/reference_scene_undergrowth.png`
- Start: `cargo run -p world_generator --example reference_scene`

Erst wenn die Reference Scene stimmungsmäßig **und** maßstäblich passt, Regeln
zurück in `generate_chunk` / Phase 5 (Layering) — nicht umgekehrt.

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