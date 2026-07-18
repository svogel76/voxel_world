//! Simple Avian capsule that stands on terrain colliders and moves with WASD.

use avian3d::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_voxel_world::prelude::ChunkWillSpawn;

use crate::height::VoxelNoiseHeight;
use crate::lighting::mood_camera_bundle;
use crate::terrain::VoxelTerrain;
use world_generator::TerrainHeightSource;

const PLAYER_RADIUS: f32 = 0.35;
const PLAYER_HEIGHT: f32 = 1.8;
const MOVE_SPEED: f32 = 8.0;
const JUMP_IMPULSE: f32 = 6.5;
const MOUSE_SENS: f32 = 0.0025;

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let height = VoxelNoiseHeight::default_world();
    let ground = height.height_at(0.0, 0.0);
    let spawn_y = ground + PLAYER_HEIGHT * 0.5 + 0.5;

    let capsule_len = (PLAYER_HEIGHT - PLAYER_RADIUS * 2.0).max(0.1);
    let mesh = meshes.add(Capsule3d::new(PLAYER_RADIUS, capsule_len));
    let material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.85, 0.75, 0.55),
        perceptual_roughness: 0.8,
        ..default()
    });

    commands
        .spawn((
            Name::new("Player"),
            Player,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, spawn_y, 0.0),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, capsule_len),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(4.0),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            GravityScale(2.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("PlayerCamera"),
                PlayerCamera,
                mood_camera_bundle(),
                Transform::from_xyz(0.0, 1.6, 6.0).looking_at(Vec3::new(0.0, 1.2, 0.0), Vec3::Y),
            ));
        });
}

pub fn player_look(
    accumulated: Res<AccumulatedMouseMotion>,
    mut player_q: Query<&mut Transform, With<Player>>,
) {
    let Ok(mut transform) = player_q.single_mut() else {
        return;
    };
    let delta = accumulated.delta;
    if delta == Vec2::ZERO {
        return;
    }
    transform.rotate_y(-delta.x * MOUSE_SENS);
}

pub fn player_move(
    keys: Res<ButtonInput<KeyCode>>,
    mut player_q: Query<(&Transform, &mut LinearVelocity), With<Player>>,
) {
    let Ok((transform, mut velocity)) = player_q.single_mut() else {
        return;
    };

    let mut wish = Vec3::ZERO;
    if keys.pressed(KeyCode::KeyW) {
        wish += *transform.forward();
    }
    if keys.pressed(KeyCode::KeyS) {
        wish -= *transform.forward();
    }
    if keys.pressed(KeyCode::KeyA) {
        wish -= *transform.right();
    }
    if keys.pressed(KeyCode::KeyD) {
        wish += *transform.right();
    }
    wish.y = 0.0;
    if wish.length_squared() > 0.0 {
        wish = wish.normalize() * MOVE_SPEED;
        velocity.x = wish.x;
        velocity.z = wish.z;
    }

    if keys.just_pressed(KeyCode::Space) {
        velocity.y = JUMP_IMPULSE;
    }
}

/// Attach terrain mesh colliders when `bevy_voxel_world` spawns a chunk mesh entity.
///
/// In Bevy 0.19 / bvw 0.17 these are Messages (`MessageReader`), not classic Events.
pub fn attach_chunk_colliders(
    mut commands: Commands,
    mut events: MessageReader<ChunkWillSpawn<VoxelTerrain>>,
) {
    for event in events.read() {
        commands.entity(event.entity).insert((
            RigidBody::Static,
            ColliderConstructor::TrimeshFromMesh,
        ));
    }
}
