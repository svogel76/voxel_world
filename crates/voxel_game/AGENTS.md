# Agents.md — voxel_game

Extends workspace-root `AGENTS.md`.

## Role

This is the **only** crate that may depend on Bevy / `bevy_voxel_world` /
`avian3d` under `[dependencies]`. Generator crates stay Bevy-free.

## Rules

1. Explain what third-party crates abstract (chunking/meshing, physics ECS).
2. Prefer thin adapters over putting game logic into generators.
3. Height for placement must stay consistent with voxel fill (shared noise params).
4. `cargo check -p voxel_game` for the feedback loop.
5. Debug UX stays keybind + Bevy UI text (see `debug_console.rs`) unless egui
   is explicitly discussed. Day pause must not freeze Avian (`DayCycle`, not
   `Time<Virtual>::pause`).
