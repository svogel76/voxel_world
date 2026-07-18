//! Playable shell: noise voxel terrain + vegetation + Avian + mood + day/night.

mod day_night;
mod debug_console;
mod fps;
mod height;
mod lighting;
mod player;
mod terrain;
mod vegetation;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_voxel_world::prelude::*;

use day_night::DayNightPlugin;
use debug_console::DebugConsolePlugin;
use fps::FpsPlugin;
use lighting::{insert_mood_resources, setup_lights};
use player::{attach_chunk_colliders, player_look, player_move, spawn_player, PlayerCamera};
use terrain::{TerrainCamera, VoxelTerrain};
use vegetation::spawn_vegetation_chunk;

fn main() {
    let mut app = App::new();
    insert_mood_resources(&mut app);
    app.add_plugins(DefaultPlugins.set(WindowPlugin {
        primary_window: Some(Window {
            title: "voxel_game — Phase 2".into(),
            ..default()
        }),
        ..default()
    }))
    .add_plugins(FpsPlugin)
    .add_plugins(DayNightPlugin)
    .add_plugins(DebugConsolePlugin)
    .add_plugins(VoxelWorldPlugin::with_config(VoxelTerrain))
    .add_plugins(PhysicsPlugins::default())
    .add_systems(
        Startup,
        (setup_lights, spawn_vegetation_chunk, spawn_player).chain(),
    )
    .add_systems(
        Update,
        (
            mark_voxel_camera,
            player_look,
            player_move,
            attach_chunk_colliders,
        ),
    )
    .run();
}

/// `bevy_voxel_world` streams chunks around entities with [`VoxelWorldCamera`].
fn mark_voxel_camera(
    mut commands: Commands,
    q: Query<Entity, (With<PlayerCamera>, Without<VoxelWorldCamera<VoxelTerrain>>)>,
) {
    for entity in &q {
        commands
            .entity(entity)
            .insert((VoxelWorldCamera::<VoxelTerrain>::default(), TerrainCamera));
    }
}
