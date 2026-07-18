# voxel_game

Phase-2 game crate: Bevy app that integrates `bevy_voxel_world` terrain with
`world_generator` vegetation, Avian3D physics, mood lighting, and a day/night
cycle with a lightweight debug overlay.

## What this crate owns

- Bevy app entrypoint and plugins
- Noise-driven voxel terrain via `bevy_voxel_world` (`VoxelTerrain` config)
- `TerrainHeightSource` backed by the **same** height function as the voxels
- Spawning one `generate_chunk` area as placeholder cubes / grass quads
- Avian static colliders on chunk meshes + a simple walkable capsule
- Phase-2 mood: CSM key/fill lights, volumetric fog, Bloom, SSAO, FPS overlay
- Day/night cycle (`day_night.rs`) driving sun / ambient / fog
- Debug console overlay (`debug_console.rs`) — keybinds, no egui

## What it deliberately does not own

- L-System / grass / rock algorithms (stay in generator crates)
- Textures / atlas (Phase 3)
- Reference-scene composition (hand-placed hero shot stays in `world_generator` examples)

## Run

```bash
cargo run -p voxel_game
```

Controls: WASD move, Space jump, mouse yaw. Third-person capsule on Avian physics.

### Debug / day-night

| Key | Action |
|-----|--------|
| `F1` / `` ` `` | Toggle debug panel |
| `P` | Pause / resume day cycle (physics keeps running) |
| `[` / `]` | Day speed ×0.5 / ×2 |
| `T` | Scrub time +2.4 h |
| `-` / `=` | Longer / shorter day length |
| `F` | Fog on/off |
| `O` | SSAO on/off |
| `R` | Reset day defaults |

Default day length: **10 minutes** realtime per full cycle.

## Scale

`1` voxel edge ≈ `1` meter (see root Roadmap / `world_generator` README).
