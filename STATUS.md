# Status — voxel_world

Kurzüberblick für Menschen und Agents: **wo stehen wir, was kommt als Nächstes**.
Detail-Checklisten und Entscheidungen bleiben in
[`docs/Roadmap_Voxel_Atmosphere.md`](./docs/Roadmap_Voxel_Atmosphere.md).

**Zuletzt aktualisiert:** 18.07.2026

---

## Aktueller Stand

| Bereich | Status |
|--------|--------|
| Phase 0 (eigene Mini-Engine) | bewusst zurückgestellt; Fundamentals via Generatoren + `voxel_game` |
| Phase 1 (`voxel_game` + Terrain + Avian) | erledigt |
| Phase 1.5 (Generator-Crates + `generate_chunk`) | Kernarbeit erledigt |
| Phase 2 (Licht-Fundament) | weitgehend erledigt; Contact Shadows bewusst offen |
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

### Maßstab (Weltziel)

| Größe | Wert |
|-------|------|
| 1 Voxel-Kante | ≈ 1 m |
| Spieler | ≈ 1,8 m (Augenhöhe ~1,6 m) |
| Wald-Bäume | 15–30 m (Hero ~20–25 m in der Reference Scene) |

Forest-`TreeParams` in `generate_chunk` sind **noch nicht** global auf diesen
Maßstab umgestellt — erst Scale-Feeling bestätigen.

---

## Nächste Schritte (Reihenfolge)

1. **Lokal validieren** — `cargo run -p voxel_game` (Stimmung, FPS, Licht auf echtem Terrain)
2. **Scale-Feeling bestätigen** — Spieler klein unter Kronendach; Presets erst nach OK anpassen
3. **Unterholz-Regeln** — Farne / Büsche / Fallstamm aus der Reference Scene in Generator-Logik
4. **Phase 3 — Texturen & Materialien** — Atlas/PBR, nicht vorher vorziehen

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
| `bush_generator` / Debris-Crate | erst nach visueller Validierung der Demo |
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
