# voxel_game

Phase-2 game crate: Bevy app that integrates `bevy_voxel_world` terrain with
`world_generator` vegetation, Avian3D physics, and reference-scene mood lighting.

## What this crate owns

- Bevy app entrypoint and plugins
- Noise-driven voxel terrain via `bevy_voxel_world` (`VoxelTerrain` config)
- `TerrainHeightSource` backed by the **same** height function as the voxels
- Spawning one `generate_chunk` area as placeholder cubes / grass quads
- Avian static colliders on chunk meshes + a simple walkable capsule
- Phase-2 mood: CSM key/fill lights, volumetric fog, Bloom, SSAO, FPS overlay

## What it deliberately does not own

- L-System / grass / rock algorithms (stay in generator crates)
- Textures / atlas (Phase 3)
- Reference-scene composition (hand-placed hero shot stays in `world_generator` examples)

## Run

```bash
cargo run -p voxel_game
```

Controls: WASD move, Space jump, mouse yaw. Third-person capsule on Avian physics.

## Scale

`1` voxel edge ≈ `1` meter (see root Roadmap / `world_generator` README).
