# Status — voxel_world

Kurzüberblick für Menschen und Agents: **wo stehen wir, was kommt als Nächstes**.
Detail-Checklisten und Entscheidungen bleiben in
[`docs/Roadmap_Voxel_Atmosphere.md`](./docs/Roadmap_Voxel_Atmosphere.md).

**Zuletzt aktualisiert:** 18.07.2026 (Unterholz-Regeln in `generate_chunk`)

---

## Aktueller Stand

| Bereich | Status |
|--------|--------|
| Phase 0 (eigene Mini-Engine) | bewusst zurückgestellt; Fundamentals via Generatoren + `voxel_game` |
| Phase 1 (`voxel_game` + Terrain + Avian) | erledigt |
| Phase 1.5 (Generator-Crates + `generate_chunk`) | Kernarbeit erledigt |
| Phase 2 (Licht-Fundament) | weitgehend erledigt; Contact Shadows bewusst offen |
| Scale-Feeling (Forest-Presets) | Presets umgesetzt — lokal unter Kronendach gegenprüfen |
| Unterholz-Regeln | Farne / Büsche / Fallstämme in Forest-`generate_chunk` |
| Phase 3–6 | noch nicht begonnen |

### Was läuft schon

- **Workspace:** `tree_generator`, `grass_generator`, `rock_generator`,
  `world_generator` (Bevy-frei), `voxel_game` (Bevy-Integration)
- **Terrain:** `bevy_voxel_world` + gemeinsamer Noise-Height-Source für Voxel-Fill
  und Vegetation-Platzierung
- **Vegetation:** ein `generate_chunk`-Spawn in `voxel_game` (Cubes / Gras-Quads)
- **Physik:** Avian-Trimesh auf Chunks + begehbare Capsule (WASD, Space, Maus)
- **Licht (Phase 2):** CSM Key/Fill, Volumetrics, Bloom, SSAO — in
  `reference_scene` *und* portiert nach `voxel_game` (`lighting.rs`)
- **Debug:** FPS-Overlay in `voxel_game`
- **Reference Scene:** art-directed Hero Shot inkl. Scale-/Unterholz-Spike
  (`cargo run -p world_generator --example reference_scene`)
- **Forest-Scale:** `params_for(Forest)` nutzt Frame-Turtle aus der Reference
  Scene (`step_length: 2.0`, `base_thickness: 4.0`, `tree_density: 0.02`)
- **Unterholz (Forest):** fern-lastiger Boden (`density: 2.0`), dichtere
  Stammfuß-Farne, Leaf-Büsche pro Baum, gelegentliche Fallstämme
  (`world_generator::understory`)

### Maßstab (Weltziel)

| Größe | Wert |
|-------|------|
| 1 Voxel-Kante | ≈ 1 m |
| Spieler | ≈ 1,8 m (Augenhöhe ~1,6 m) |
| Wald-Bäume | 15–30 m (Hero ~20–25 m in der Reference Scene) |

---

## Nächste Schritte (Reihenfolge)

1. **Lokal gegenprüfen** — `cargo run -p voxel_game`: Scale + Unterholz unter Kronendach
2. **Phase 3 — Texturen & Materialien** — Atlas/PBR, nicht vorher vorziehen

Ein Schritt, eine sichtbare Veränderung. Keine Phasen überspringen
(siehe Root-[`AGENTS.MD`](./AGENTS.MD)).

---

## Bewusst zurückgestellt / offen

| Thema | Warum / wann |
|-------|----------------|
| Contact Shadows (Phase 2) | Feinschliff; nicht blockierend |
| GPU-Instancing / Mesh-Batching | wenn Vegetationsdichte FPS drückt |
| Vereinfachte Avian-Collider pro Objekttyp | nach Scale/Unterholz, vor dichter Welt |
| Baum-Hang-Filter (zu steil → kein Baum) | nur implizit über Poisson-Dichte |
| Wind, Lianen, prozedurales Layering (Phase 5) | nach Texturen / wenn Komposition steht |
| Eigenes `bush_generator` / Debris-Crate | optional; Logik ist inline in `understory.rs` |
| Moos als eigener Blocktyp | Fallstamm-Moos nur in Reference Scene (Tint) |
| Phase 0 Stages 4–5 | Lernpfad; nicht Voraussetzung für aktuelle Arbeit |

---

## Schnellbefehle

```bash
cargo run -p voxel_game
cargo run -p world_generator --example reference_scene
cargo run -p world_generator --example visualize
cargo check -p voxel_game
```

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
