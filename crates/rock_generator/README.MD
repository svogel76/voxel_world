# rock_generator

Prozedurale Erzeugung kleiner, verstreuter Boulder — Teil des
Weltgenerator-Workspace (siehe Root-`README.md` und
`Roadmap_Voxel_Atmosphere.md`, Phase 1.5).

## Scope-Abgrenzung

Dieses Crate ist NUR für kleine, unregelmäßige Boulder zuständig — keine
großen Monumente/Steinpfeiler mit eingeritzten Symbolen (die gehören zu
einem separaten, noch nicht begonnenen Struktur-/Ruinen-Generator oder
werden als feste Hand-Prefabs gelöst).

## Architekturprinzip

Wie `tree_generator`: **Bevy-frei**, Ausgabe als Voxel-Positionen
(`Vec<(IVec3, BlockType)>`), Bevy nur als `dev-dependency` für
`examples/visualize.rs`.

## Entscheidung: Noise-Schwellenwert statt Ellipsoid/Random-Walk

Ein 3D-Perlin/Simplex-Noise-Feld über eine Bounding-Box; jeder Punkt mit
Noise-Wert über einer Schwelle wird zu einem Stein-Voxel. Kein
Ellipsoid-Grundkörper (der einen glatten runden Kern behalten würde), kein
Random-Walk (schwer parametrisierbar, tendiert zu wurmartigen Formen).
Kantigkeit direkt über Noise-Frequenz steuerbar — gleiches Prinzip wie die
2D-Terrain-Heightmap aus Phase 0, nur in 3D.

## Offene Design-Frage für Phase 1: Noise selbst implementieren oder Crate?

Passend zur Kategorisierung in der Root-`Agents.md` ("Fundament" /
"Domäne" / "3rd Party"): Perlin/Simplex-Noise wurde in Phase 0 (minimale
Engine, Project 1) bereits als 2D-Heightmap selbst implementiert. Diese
Erfahrung spricht dafür, die 3D-Erweiterung ebenfalls selbst zu schreiben
("Domäne", überschaubare Komplexität), statt ein Noise-Crate (z.B. `noise`)
einzubinden. Der Coding Agent soll das im Design-Schritt von Phase 1
explizit vorschlagen und begründen, nicht stillschweigend entscheiden.

## Abhängigkeiten

`rand`, `glam` (Fundament, wie bei den anderen Generatoren). Noise-Logik
selbst geschrieben (siehe oben) — kein externes Noise-Crate, solange die
Komplexität überschaubar bleibt.

## Entwicklungsphasen

### Phase 1 — Noise-basierte Klumpenform
- Eingabe: `seed: u64`, Bounding-Box-Größe (z.B. Radius oder
  min/max je Achse), Schwellenwert, Noise-Frequenz
- Ausgabe: `Vec<(IVec3, BlockType)>` (`BlockType::Stone`), zentriert um den
  Ursprung — Weltposition ist späterer Integrationsschritt (wie bei
  `tree_generator`/`grass_generator`)
- 3D-Noise-Feld selbst implementiert (siehe oben), Schwellenwert-Test pro
  Voxel innerhalb der Bounding-Box
- Determinismus: `StdRng`/Noise-Seed konsistent mit `seed`, keine
  `thread_rng()`

### Phase 2 — Größen-/Formvariation
- Eingabe: die Form-Parameter aus Phase 1, `seed: u64`
- Nicht-uniforme Skalierung je Achse (flachere oder länglichere Boulder,
  nicht nur gleichmäßig größer/kleiner)
- Radialer Dichte-Bias (`noise − falloff · Abstand`) für dichteren Kern /
  dünneren Rand
- Parametrische Streuung über Seed (wie `tree_generator` Phase 5), keine
  L-System-Grammatik nötig — deutlich einfacher als Bäume

#### Entscheidung: zwei Seed-Mechanismen (Noise-Hash vs. `StdRng`)

Bewusst getrennt — im Geist von `grass_generator` (unabhängige Zufallsströme
aus demselben `seed`), aber mit dem Mechanismus, der zur jeweiligen Aufgabe
passt:

- **Noise-Feld (Phase 1):** Hash über Gitterkoordinaten + `seed`. Value Noise
  braucht zufällige Werte an beliebigen Lattice-Punkten in O(1); ein
  sequentieller `StdRng`-Strom wäre dafür ungeeignet.
- **Achsen-Skalierung (Phase 2):** `StdRng::seed_from_u64(seed)` für die
  wenigen Uniform-Ziehungen pro Boulder (natürliche API, wie bei
  grass/tree-Variation).

Beide Schritte erhalten denselben `seed` unabhängig (kein gemeinsamer
Strom). Die Achsen-Draws verschieben das Noise-Feld nicht und umgekehrt —
gleicher Seed → gleiche Form.

### Phase 3 — Visuelle Validierung
- `examples/visualize.rs`: minimale Bevy-App, mehrere Boulder mit
  unterschiedlichen Seeds nebeneinander
- Start: `cargo run -p rock_generator --example visualize`

## Öffentliche API (Zielbild)

```rust
pub fn generate(seed: u64, params: &RockParams) -> Vec<(IVec3, BlockType)>;
```

## Tests

Jede Phase bekommt eigene Unit-Tests, bevor die nächste beginnt (siehe
`Agents.md`).