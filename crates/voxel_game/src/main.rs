//! Phase-1 playable shell: noise voxel terrain + one vegetation chunk + Avian capsule.
//!
//! `bevy_voxel_world` owns chunk streaming / meshing. `world_generator` stays Bevy-free;
//! we only adapt height and spawn its output as placeholder meshes.

mod height;
mod player;
mod terrain;
mod vegetation;

use avian3d::prelude::*;
use bevy::prelude::*;
use bevy_voxel_world::prelude::*;

use player::{attach_chunk_colliders, player_look, player_move, spawn_player, PlayerCamera};
use terrain::{TerrainCamera, VoxelTerrain};
use vegetation::spawn_vegetation_chunk;

fn main() {
    App::new()
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "voxel_game — Phase 1".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(VoxelWorldPlugin::with_config(VoxelTerrain))
        .add_plugins(PhysicsPlugins::default())
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.75, 0.80, 0.90),
            brightness: 220.0,
            ..default()
        })
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

fn setup_lights(mut commands: Commands) {
    commands.spawn((
        Name::new("Sun"),
        DirectionalLight {
            illuminance: 12_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_rotation(Quat::from_euler(EulerRot::XYZ, -0.9, 0.6, 0.0)),
    ));
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
