# Roadmap: Vom Terrain zum Concept-Art-Flair

Ziel: Der dunkle, atmosphärische "Blocky Forest"-Look mit bevy_voxel_world als Terrain-Basis.

Kernerkenntnis aus der Konzeptanalyse: Der Stil entsteht **nicht** durch feinere Voxelauflösung,
sondern durch Licht, Texturen und Postprocessing. Die Blockgröße bleibt klassisch.

### Maßstab (Weltziel, bewusst realistischer als manches Concept Art)

| Größe | Wert |
|-------|------|
| 1 Voxel-Kante | ≈ 1 m |
| Spieler | ≈ 1,8 m (Augenhöhe ~1,6 m) |
| Wald-Bäume | 15–30 m Höhe, Stammdicke ~2–5 m Basis |

Der Spieler soll sich **klein** unter dem Kronendach fühlen. Style bleibt blockig;
Proportionen dürfen vom Concept-Art-PNG abweichen (dort oft kompaktere Bäume).
Validierung zuerst in `world_generator` → `examples/reference_scene.rs` (Player-Proxy
+ Hero-Baum). Forest-Presets in `generate_chunk` nutzen die Frame-Scale aus der
Reference Scene (`biome::params_for(Forest)`); lokal unter dem Kronendach in
`voxel_game` gegenprüfen.

---

## Phase 0 — Aktuelle minimale Engine abschließen (laufend)
Lernprojekt, plugin-frei, wie bisher geplant.
- [ ] Stage 4: Perlin-Noise-Heightmap
- [ ] Stage 5: Multi-Chunk Loading/Unloading

Zweck: ECS-, Meshing- und Chunking-Fundamentals verstanden haben, bevor bevy_voxel_world
diese Arbeit übernimmt — damit die Abstraktion kein Blackbox-Gefühl hinterlässt.

---

## Phase 1 — Umstieg auf bevy_voxel_world
- [x] bevy_voxel_world in ein neues, separates Projekt einbinden (nicht die Lern-Engine umbauen)
  → Crate [`crates/voxel_game`](../crates/voxel_game) mit `bevy_voxel_world` 0.17 (Bevy 0.19)
- [x] Terrain-Generierung mit eigenem Noise-Backend an bevy_voxel_world anbinden
  → `VoxelNoiseHeight` / `SimpleNoiseTerrain` als gemeinsamer Height-Source für Voxel-Fill
  und `generate_chunk` (nicht `get_voxel` der Engine — unloaded chunks sind `Unset`)
- [x] Chunk-Streaming/LOD-Verhalten des Crates verstehen (Config-Optionen, Render-Distance)
  → `VoxelWorldCamera` + `spawning_distance`; LOD-Deep-Dive bewusst später
- [x] Kollision/Physik-Integration mit **Avian3D** grundlegend testen
  (Entscheidung: Avian statt bevy_rapier — native ECS-Integration, keine separate
  Physik-Welt, besser lesbarer Source Code; passt zu ECS-Denkweise)
  - [x] `ColliderConstructor`/`ColliderConstructorHierarchy` zur automatischen
    Collider-Generierung aus den Terrain-Chunk-Meshes einrichten
    → `ChunkWillSpawn` → `RigidBody::Static` + `ColliderConstructor::TrimeshFromMesh`
    + begehbare Capsule (WASD)

Meilenstein: Begehbares, generiertes Blockterrain — noch ohne Stil.

---

## Phase 1.5 — Weltgenerator: Objekt-Platzierung
Übergeordnetes Konzept: Ein **Weltgenerator** orchestriert pro Chunk mehrere spezialisierte
Sub-Generatoren, die alle dieselbe Grundfrage beantworten ("wo darf hier etwas stehen,
und was genau?"), aber mit eigenen Regeln. Terrain (Phase 1) ist der erste Sub-Generator,
die folgenden kommen jetzt dazu. Notwendig, bevor Licht/Postprocessing (Phase 2) an einer
echten Szene mit Bäumen/Blätterdach getestet werden kann.

### Architektur: Cargo Workspace mit isolierten Generator-Crates
Entscheidung: Jeder Sub-Generator (Baum, Gras, Stein, Weltgenerator als Orchestrator)
wird ein eigenständiges Crate in einem gemeinsamen Cargo Workspace — kein Monolith.
- [x] Workspace-Grundgerüst anlegen (`Cargo.toml` mit `[workspace] members = [...]`)
- [x] Jeder Generator ist bewusst **Bevy-frei**: reine Rust-Logik (Seed + Parameter rein,
  Datenstruktur wie `Vec<VoxelPos>` raus), damit er isoliert mit `cargo test -p <crate>`
  testbar ist, ohne die komplette Bevy-App zu starten
- [x] Bevy wird pro Generator-Crate nur als `dev-dependency` eingebunden — sichtbar nur
  für `examples/`, nicht Teil der eigentlichen Library-API
- [x] Visuelle Validierung pro Generator über ein eigenes `examples/visualize.rs`
  (Äquivalent zur "Testszene" aus Unity), gestartet mit
  `cargo run -p <crate> --example visualize`
- [x] Erst ein dünner Integrations-Layer im späteren Haupt-Spiel-Crate übersetzt die
  generierten Voxel-Daten in echte Bevy-Entities/Components
  → `voxel_game::vegetation` spawnt eine feste Area aus `generate_chunk` als Cubes /
  Gras-Cross-Quads auf dem Noise-Terrain

### Biom-/Zonen-Bestimmung
- [x] Zweiter Noise-Layer (z.B. Feuchtigkeit) zusätzlich zur Höhenkarte
- [x] Einfache Zonen ableiten (Wald, Fels, Lichtung) aus Höhe + Feuchtigkeit kombiniert
- [x] Jedes Biom bekommt eigene Parameter (Baumdichte, Baumarten/L-System-Regelsätze,
  Grasdichte, Steindichte), die an die jeweiligen Sub-Generatoren weitergereicht werden —
  der Weltgenerator entscheidet *was* und *wie viel*, der Generator entscheidet *wie*
- **Entscheidung — Terrain-Höhen-Zugriff:** `world_generator` bleibt Bevy-frei und
  hängt nicht direkt von `bevy_voxel_world` ab. Terrain-Höhe (und später Hangsteigung
  für Steine) wird über ein austauschbares `TerrainHeightSource`-Trait abstrahiert.
  Für Entwicklung/Tests jetzt: einfache Test-Implementierung (z.B. eigene Platzhalter-
  Noise-Funktion). Die echte Anbindung an `bevy_voxel_world` entsteht erst später im
  `voxel_game`-Crate, nicht in `world_generator` selbst.

### Baumgenerator (mit Formgenerierung, nicht nur Platzierung)
Entscheidung: Bäume werden prozedural per **L-System** erzeugt, nicht aus fertigen
Modellen platziert — passt zum Voxel-Stil und ist eigenständig lehrreich.
- [x] L-System-Grundlagen verstehen: Axiom, Produktionsregeln, Iterationstiefe
  (isoliert testen, z.B. nur als 2D-Textausgabe/Zeichenkette, bevor irgendwas 3D wird)
- [x] Turtle-Graphics-Interpretation: Zeichenkette → Liste von 3D-Liniensegmenten
  (Vorwärts, Abbiegen, Verzweigen/Stack push-pop für Äste)
- [x] Voxelisierung: Liniensegmente → Voxel-Blöcke (Stamm dicker/gerader,
  Äste dünner, Bresenham-artiges Line-Voxelization-Verfahren)
- [x] Blätter/Laubkronen hinzufügen (z.B. Voxel-Cluster an Astenden)
- [x] Parametrische Variation (zufällige Winkel/Iterationstiefe innerhalb Grenzen,
  damit nicht jeder Baum identisch aussieht)

### Vegetationsgenerator (Farne, Gras)
- [x] Einfachere prozedurale Form (kein volles L-System nötig, z.B. Billboard-Quads
  oder simple Voxel-Cluster) — bewusst geringerer Aufwand als Bäume
- [x] Dichte-basierte Verteilung (Zufalls-Sampling proportional zur Fläche, kein
  echtes Noise-Feld nötig — bewusste Vereinfachung gegenüber Bäumen/Steinen, siehe
  `grass_generator`-README)
- **Entscheidung:** Statische Kreuzquads (zwei senkrecht zueinander stehende
  Ebenen, per Zufallsrotation um Y variiert — wie in Minecraft), kein
  kamera-ausgerichtetes Echtzeit-Billboarding, keine Micro-Voxel-Cluster.
  Begründung: passt zum Grundsatz "Atmosphäre durch Licht/Textur, nicht durch
  Geometrie" (siehe Concept-Art-Analyse ganz am Anfang), und ist bei
  tausenden Grasbüscheln pro Chunk performance-kritisch — 2 Quads (4
  Dreiecke) pro Büschel statt Dutzender/Hunderter Voxel. `grass_generator`
  selbst liefert nur Platzierungsdaten (Position, Y-Rotation, Skalierung,
  Variante); die eigentliche Quad-Geometrie entsteht erst in der
  Bevy-Integrationsschicht.

### Steingenerator
- [x] Einfache Voxel-Cluster-Formen mit Größen-/Rotationsvariation
- [x] Verteilungsregel an Terrain-Steigung koppeln (mehr Steine an Hängen,
  via `rock_density_multiplier(slope)` in `world_generator`)
- **Entscheidung — Technik:** 3D-Noise-Schwellenwert (Perlin/Simplex-Feld über
  eine Box, Voxel wird "Stein" wenn Noise-Wert > Schwelle), statt Ellipsoid
  mit Rand-Jitter oder 3D-Random-Walk. Begründung: Ellipsoid-Jitter behält
  einen glatten, runden Kern (nur der Rand wird uneben) — passt nicht zum
  Blockstil, besonders bei größeren Boulder. Noise-Schwellenwert hat keinen
  Rundheits-Bias, die Kantigkeit ist direkt über die Noise-Frequenz steuerbar,
  und das Prinzip ist aus der Terrain-Heightmap (Phase 0) bereits bekannt
  (nur diesmal 3D statt 2D).
- **Scope-Abgrenzung:** `rock_generator` ist NUR für kleine, verstreute
  Boulder zuständig. Große, bewusst geformte Monumente/Steinpfeiler mit
  eingeritzten Symbolen (wie im Concept Art, gestapelte rechteckige Blöcke)
  gehören NICHT hierher — das wäre ein eigener, späterer
  Struktur-/Ruinen-Generator (noch nicht in Arbeit) oder feste Hand-Prefabs.

### Platzierungslogik (gilt für alle Sub-Generatoren)
- [x] Zugriff auf Terrain-Höhe an Position (X,Z) konsistent mit Voxel-Terrain
  — `VoxelNoiseHeight` im `voxel_game`-Crate teilt denselben Height-Source mit dem
  Voxel-Lookup (Engine-`get_voxel` bleibt für unloaded Chunks ungeeignet)
- [x] Poisson-Disc-Sampling für natürlichen Mindestabstand bei Bäumen (Bridson's
  Algorithmus, selbst implementiert in `world_generator`)
- [x] Regeln an Terrain-Eigenschaften koppeln — für Steine umgesetzt (Hangsteigung
  → Dichte-Multiplikator); für Bäume (zu steile Hänge → kein Baum) noch nicht
  explizit umgesetzt, nur implizit über Poisson-Disc-Dichte

### Performance & Kollision
- [ ] GPU-Instancing/Mesh-Batching prüfen, sobald viele gleichartige Objekte
  (Bäume, Gras) in der Szene stehen — sonst bricht die Framerate früh ein
- [ ] Vereinfachte Avian3D-Collider pro Objekttyp (z.B. Zylinder für Baumstamm,
  statt teurer Mesh-genauer Kollision)

**Status:** Kernarbeit abgeschlossen (18.07.2026). Alle vier Crates
(`tree_generator`, `grass_generator`, `rock_generator`, `world_generator`)
phasenweise entwickelt, getestet und visuell validiert — zuletzt gemeinsam als
`generate_chunk()` mit drei unterschiedlichen Biomen (Forest/Rocky/Clearing) in
einer Szene mit terrain-höhengekoppelten Objekten. Offen bleiben: echte
`bevy_voxel_world`-Anbindung (Teil von Phase 1), Performance/Kollision (später).

Meilenstein: Ein Testchunk mit prozedural generierten, unterschiedlichen Bäumen,
Farnen, Gras und Steinen — die Grundlage, an der Phase 2 (Licht) sinnvoll getestet
werden kann (dein "Lichtstrahl durchs Blätterdach"-Ziel braucht ein Blätterdach).

---

## Phase 2 — Licht-Fundament
Das ist der wichtigste Hebel für die Atmosphäre. Reihenfolge bewusst vor Texturen,
weil Licht die Grundstimmung bestimmt, die Texturen später nur unterstützen.

**Arbeitsweise (18.07.2026):** Zuerst eine art-directed **Reference Scene**
(`world_generator` → `examples/reference_scene.rs`): Generator-Bausteine mit
festen Seeds/Positionen handkomponieren (Rahmen, Blickachse, Kronenlücken),
dann Licht an genau dieser Szene drehen. Erst danach Regeln zurück in den
prozeduralen `world_generator` (nicht umgekehrt). Vergleichsbasis:
`docs/Blocky_forest.png`.

- [x] Directional Light + Schattenwurf sauber konfigurieren (Kaskaden-Shadow-Maps)
  — in `reference_scene` via `CascadeShadowConfigBuilder`
- [x] Volumetrisches Licht aktivieren und an einer Lichtungs-/Waldkanten-Szene testen
  — `VolumetricFog` + `VolumetricLight` + `FogVolume` in `reference_scene`
- [ ] Contact Shadows für Nahbereich-Details (Wurzeln, Blätter am Boden)
- [x] SSAO für Tiefe in dichten Vegetations-Clustern — in `reference_scene`
  (mit TAA / `Msaa::Off`)
- [x] Bloom, dezent — in `reference_scene` (`Bloom::NATURAL`, niedrige Intensity)

Meilenstein: Eine einzelne Test-Szene (z.B. Lichtstrahl durch Blätterdach) die dem
Concept Art schon nahekommt — rein durch Licht, mit Platzhalter-Texturen.
Erste Version: `cargo run -p world_generator --example reference_scene`.

### Zwischenschritt — Debug Console + Tag/Nacht (in `voxel_game`)
Ohne lokale GPU-Validierung schwer zu iterieren; daher vor Phase 3:
- [x] Leichte Debug-Console (F1-Overlay + Keybinds, kein egui) — Fog/SSAO/Day
- [x] Tag/Nacht-Zyklus — Key-Sun-Orbit, Ambient/Clear/Fog über `DayCycle`
  (Default 10 min/Tag; Pause friert nicht die Physik ein)

### Sky-Mini (in `voxel_game`, vor Phase 3) — erledigt
Tag/Nacht steuerte bisher nur Licht/Fog/`ClearColor` — der Himmel blieb schwarz.
Ein kleiner Abschluss macht den Zyklus lesbar, ohne Wetter-Engine:

- [x] Sky-Dome oder Gradient-Himmel an `DayCycle` koppeln
  (Tag kühles Blau → Abend warm → Nacht dunkel)
  → `sky.rs`: invertierte Unlit-Kugel + `ClearColor` über `sky_zenith_color`
- [x] Optional: Sonnen-Billboard entlang der Key-Sun-Richtung
  (nur Optik; das Directional Light bleibt die echte Lichtquelle)
  → `SkySun`-Kugel folgt Kamera + KeySun-Richtung, unsichtbar unter dem Horizont

**Bewusst nicht in diesem Schritt:** Cubemap-Skybox, Sternenhimmel,
volumetrische Wolken, Wetter — siehe Phase 4.5 und Phase 7.

Meilenstein: Mit `T` / Speed sieht man den Tageswechsel auch am Himmel,
nicht nur an Schatten und Console.

---

## Phase 3 — Texturen & Materialien
- [ ] Hochauflösendes Textur-Set für Holz/Stein/Moos beschaffen oder erstellen
  (PBR: Albedo, Normal, Roughness — mindestens Normal Maps für Oberflächendetail)
- [ ] Texture-Atlas/Array-Setup für Voxel-Faces in bevy_voxel_world einrichten
- [ ] Moos-Übergänge zwischen Blocktypen (Blending oder Übergangs-Blocktypen)
- [ ] Vertex-Color oder Tinting für natürliche Variation (kein repetitives Kachel-Muster)

Meilenstein: Terrain-Oberflächen wirken materialhaft statt kachelig.

---

## Phase 4 — Farbgebung & Postprocessing
- [ ] Farbgrading Richtung dunkel/kontrastreich (Tonemapping-Einstellungen in Bevy)
- [ ] Nebel/Height-Fog für Tiefenwirkung in Waldschluchten
- [ ] Vignette dezent testen
- [ ] Selektive Highlights: bewusst wenige, helle Bildbereiche (wie das Wasser im Concept Art)

Meilenstein: Screenshots aus der Engine sind stimmungsmäßig mit dem Concept Art vergleichbar.

---

## Phase 4.5 — Himmel ausbauen (nach Texturen / wenn FPS stabiler)
Baut auf Sky-Mini auf. Erst wenn Phase-3-Materialien und die Grundstimmung stehen,
damit Himmel-Arbeit nicht gegen unfertige Böden und teure Volumetrics konkurriert.

- [ ] Richtige Skybox / reichhaltigerer Dome (Cubemap oder prozeduraler Shader)
- [ ] Sternenhimmel bei Nacht (ausfadend mit `day_factor`)
- [ ] Mond-Scheibe (optional, analog zur Sonnen-Billboard)
- [ ] Leichte Bewölkung — bewusst einfach zuerst
  (2D-/Dome-Billboards oder simpler Cloud-Layer, **keine** volumetrischen Wolken)

Meilenstein: Tag und Nacht sind am Himmel klar lesbar; leichte Wolken ohne
Wetter-System.

---

## Phase 5 — Vegetationsdichte & Layering
Baut auf den Objekten aus Phase 1.5 auf (Bäume, Farne, Gras, Steine existieren bereits) —
hier geht es um Komposition/Layering, nicht mehr um Erzeugung der Objekte selbst.

**Vorarbeit aus der Reference Scene** (teilweise prozedural): dichter
Stamm-Rahmen / offene Mitte / sparse Far canopy bleiben art-directed.
Maßstab und **Unterholz** (Farne, Büsche, Fallstämme) stecken in
`world_generator::understory` und laufen für Forest-Chunks in `generate_chunk`.

- [x] Bush-/Blattwerk-Cluster als Layer — manuell in `reference_scene` erprobt
  und prozedural als Leaf-Büsche in `understory` / Forest-`generate_chunk`;
  eigenes Crate / Instancing noch offen
- [ ] Wind-Bewegung auf Blattwerk (Vertex-Shader-Animation, leicht)
- [ ] Hängende Ranken/Lianen als eigene Objektklasse
- [x] Dichteverteilung: dicht am Bildrand/Vordergrund, offener in der Bildmitte
  — manuell in `reference_scene`; Forest-Boden fern-lastig + Stammfuß-Farne
  prozedural; art-directed Pfadachse noch nicht automatisiert
- [x] Totholz / Fallstamm — bemooster Log in `reference_scene` (Platzhalter-Moos);
  gelegentliche Wood-Logs in Forest-`generate_chunk`; Debris-Crate später

Meilenstein: Komposition lenkt den Blick wie im Concept Art (dunkler Vordergrund,
heller Fluchtpunkt).

---

## Phase 6 — Feinschliff & Wiederholung
- [ ] Referenz-Screenshot vs. Concept Art direkt gegenüberstellen
- [ ] Iterativ an Licht/Farbe/Dichte nachjustieren
- [ ] Performance-Check bei voller Vegetationsdichte (Draw Calls, Chunk-Radius)

---

## Phase 7 — Wetter (eigenes Thema, nach Style-Fundament)
Erst wenn Himmel (4.5), Texturen und Performance grob stehen. Wetter ist kein
Sky-Mini-Aufsatz, sondern eigener Meilenstein (Regeln, Optik, Kosten).

- [ ] Wetter-Zustände modellieren (klar / bedeckt / Regen — bewusst klein starten)
- [ ] Wolken-Dichte und Ambient an Wetter koppeln (auf Phase-4.5-Bewölkung aufbauen)
- [ ] Volumetrische Wolken nur als optionaler Spike, wenn FPS es hergibt
- [ ] Regen/Partikel und Boden-Nässe (falls stilistisch nötig) — nach Optik-Grundlage

Meilenstein: Wetter ändert Stimmung und Lesbarkeit, ohne den Blocky-Forest-Look
zu sprengen.

---

## Bewusst ausgeklammert (spätere/separate Projekte)
- Micro-Voxel/SVO-Raytracing (Teardown-Style) — anderes Ziel (Zerstörbarkeit, Mini-Detail),
  nicht notwendig für dieses Stimmungsbild
- Tierdesign/Animation — eigener Bereich, nach Etablierung des visuellen Stils
- Terrain-Diffusion-Modelle — Forschungsstand, kein produktionsreifer Bevy-Workflow

---

## Prinzip für den Weg dorthin
Ein Schritt, eine sichtbare Veränderung. Nach jeder Phase: Screenshot machen,
mit dem Concept Art vergleichen, bevor die nächste Phase beginnt.
