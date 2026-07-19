# Status — voxel_world

Kurzüberblick für Menschen und Agents: **wo stehen wir, was kommt als Nächstes**.
Detail-Checklisten und Entscheidungen bleiben in
[`docs/Roadmap_Voxel_Atmosphere.md`](./docs/Roadmap_Voxel_Atmosphere.md).

**Zuletzt aktualisiert:** 19.07.2026 (Forest-Biom-Kalibrierung + Moos-Sichtbarkeit)

---

## Aktueller Stand

| Bereich | Status |
|--------|--------|
| Phase 0 (eigene Mini-Engine) | bewusst zurückgestellt; Fundamentals via Generatoren + `voxel_game` |
| Phase 1 (`voxel_game` + Terrain + Avian) | erledigt |
| Phase 1.5 (Generator-Crates + `generate_chunk`) | Kernarbeit erledigt |
| Phase 2 (Licht-Fundament) | weitgehend erledigt; Contact Shadows bewusst offen |
| Scale-Feeling (Forest-Presets) | Presets umgesetzt |
| Unterholz-Regeln | Farne / Büsche / Fallstämme in Forest-`generate_chunk` |
| Debug Console | F1-Overlay + Keybinds in `voxel_game` |
| Tag/Nacht-Zyklus | `DayCycle` steuert Key-Sun / Ambient / Fog |
| Sky-Mini | erledigt — Dome + Sonnen-Disc an `DayCycle` |
| Phase 3.1 (Terrain-Array-Textur) | erledigt — prozedural → `voxel_game/assets/textures/terrain_array.png` |
| Phase 3.2 (Holz/Stein/Moos/Blatt) | erledigt — Einzel-PNGs + Vegetation texturiert |
| Phase 3 Moos-Übergänge (Logs) | erledigt — `WorldBlockType::Moss` auf Fallstämmen |
| Biom-Kalibrierung (Spawn) | `ROCKY_MIN_HEIGHT` 17 — Spawn nicht mehr 100 % Rocky |
| Phase 3 Rest (Moos auf Stein, Vertex-Tint) | offen |

### Was läuft schon

- **Workspace:** `tree_generator`, `grass_generator`, `rock_generator`,
  `world_generator` (Bevy-frei), `voxel_game` (Bevy-Integration)
- **Terrain / Vegetation / Physik / Phase-2-Licht** wie zuvor
- **Forest-Scale + Unterholz** in `generate_chunk`
- **Debug Console:** `F1` / `` ` `` — Pause, Speed, Scrub, Fog, SSAO, Reset
- **Tag/Nacht:** `DayCycle` (Default 10 min/Tag); pausierbar ohne Physik-Freeze
- **Sky-Mini:** Unlit-Dome + `ClearColor` + Sonnen-Kugel (`sky.rs`) — abgeschlossen
- **Kamera:** Yaw am Spieler, Pitch an `PlayerCamera` (`CameraPitch`, geclampt)
- **Terrain-Texturen (3.1):** Noise-Layer in `world_generator::voxel_textures`,
  Asset `crates/voxel_game/assets/textures/terrain_array.png`, angebunden über `voxel_texture()`
- **Vegetation-Texturen (3.2):** `wood` / `moss` / `leaf` / `stone` PNGs auf Baum-/Stein-Cubes
- **Moos-Übergänge (Logs):** `fallen_log_moss_voxels` → `WorldBlockType::Moss` mit `moss.png`

### Maßstab (Weltziel)

| Größe | Wert |
|-------|------|
| 1 Voxel-Kante | ≈ 1 m |
| Spieler | ≈ 1,8 m (Augenhöhe ~1,6 m) |
| Wald-Bäume | 15–30 m (Hero ~20–25 m in der Reference Scene) |

---

## Nächste Schritte (Reihenfolge)

1. **Phase 3 fortsetzen** — Moos auf Steinen und/oder Vertex-Tint
2. Später: Phase 4.5 (Skybox / Sterne / leichte Wolken), Phase 7 (Wetter)

Ein Schritt, eine sichtbare Veränderung. Keine Phasen überspringen
(siehe Root-[`AGENTS.MD`](./AGENTS.MD)).

---

## Bewusst zurückgestellt / offen

| Thema | Warum / wann |
|-------|----------------|
| Contact Shadows (Phase 2) | Feinschliff; nicht blockierend |
| GPU-Instancing / Mesh-Batching | wenn Vegetationsdichte FPS drückt |
| Vereinfachte Avian-Collider pro Objekttyp | vor dichter Welt |
| Baum-Hang-Filter (zu steil → kein Baum) | nur implizit über Poisson-Dichte |
| Wind, Lianen, prozedurales Layering (Phase 5) | nach Texturen / wenn Komposition steht |
| Eigenes `bush_generator` / Debris-Crate | optional; Logik inline in `understory.rs` |
| Typed command console / egui | Keybinds reichen vorerst |
| Skybox / Sterne / Mond / leichte Wolken | Phase 4.5 — nach Texturen |
| Volumetrische Wolken / Wetter | Phase 7 — nach Style-Fundament |
| Phase 0 Stages 4–5 | Lernpfad; nicht Voraussetzung für aktuelle Arbeit |

---

## Schnellbefehle

```bash
cargo run -p voxel_game
cargo run -p world_generator --example reference_scene
cargo run -p world_generator --example generate_terrain_textures
cargo check -p voxel_game
```

Debug (in `voxel_game`): `F1` Hilfe · `P` Tag pausieren · `[` / `]` Tempo · `T` +2.4 h · `F` Fog · `O` SSAO · `R` Reset

Concept Art: `docs/Blocky_Forest.png` (und verwandte PNGs in `docs/`).

---

## Pflege dieser Datei

Nach jedem abgeschlossenen Meilenstein oder Richtungswechsel:

1. Tabelle „Aktueller Stand“ und „Nächste Schritte“ anpassen
2. Datum oben aktualisieren
3. Roadmap-Checkboxen in `docs/Roadmap_Voxel_Atmosphere.md` mitziehen

Agents: bei Phasenabschluss oder neuer Priorität **diese Datei mitcommitten**,
nicht nur die Roadmap. `STATUS.md` ist die Einstiegsquelle; die Roadmap bleibt
das ausführliche Projektwissen.
