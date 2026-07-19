//! Simple Avian capsule that stands on terrain colliders and moves with WASD.

use avian3d::prelude::*;
use bevy::input::mouse::AccumulatedMouseMotion;
use bevy::prelude::*;
use bevy_voxel_world::custom_meshing::CHUNK_SIZE_I;
use bevy_voxel_world::prelude::Chunk;

use crate::height::{top_solid_y, VoxelNoiseHeight};
use crate::lighting::mood_camera_bundle;
use crate::terrain::VoxelTerrain;

const PLAYER_RADIUS: f32 = 0.35;
const PLAYER_HEIGHT: f32 = 1.8;
const MOVE_SPEED: f32 = 8.0;
const JUMP_IMPULSE: f32 = 6.5;
const MOUSE_SENS: f32 = 0.0025;
/// Keep pitch away from ±90° so the view does not flip.
const PITCH_LIMIT: f32 = std::f32::consts::FRAC_PI_2 - 0.05;
const PLAYER_GRAVITY: f32 = 2.0;
const SPAWN_CLEARANCE: f32 = 0.1;
/// Third-person camera offset in player-local space (behind + slightly above).
const CAMERA_OFFSET: Vec3 = Vec3::new(0.0, 1.6, 6.0);

#[derive(Component)]
pub struct Player;

#[derive(Component)]
pub struct PlayerCamera;

/// Vertical look angle in radians (positive = look up). Applied only on the camera.
#[derive(Component)]
pub struct CameraPitch(pub f32);

/// Gravity stays off until a terrain chunk collider covers the player.
#[derive(Component)]
pub struct WaitingForTerrain;

fn spawn_y_at(x: i32, z: i32) -> f32 {
    let height = VoxelNoiseHeight::default_world();
    let ground = (top_solid_y(&height, x, z) + 1) as f32;
    ground + PLAYER_HEIGHT * 0.5 + SPAWN_CLEARANCE
}

fn chunk_pos_containing(world: Vec3) -> IVec3 {
    IVec3::new(
        (world.x as i32).div_euclid(CHUNK_SIZE_I),
        (world.y as i32).div_euclid(CHUNK_SIZE_I),
        (world.z as i32).div_euclid(CHUNK_SIZE_I),
    )
}

pub fn spawn_player(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let spawn_y = spawn_y_at(0, 0);

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
            WaitingForTerrain,
            Mesh3d(mesh),
            MeshMaterial3d(material),
            Transform::from_xyz(0.0, spawn_y, 0.0),
            RigidBody::Dynamic,
            Collider::capsule(PLAYER_RADIUS, capsule_len),
            LockedAxes::ROTATION_LOCKED,
            LinearDamping(4.0),
            Friction::ZERO.with_combine_rule(CoefficientCombine::Min),
            Restitution::ZERO.with_combine_rule(CoefficientCombine::Min),
            // Off until `enable_player_on_terrain` — otherwise we fall through the
            // hollow trimesh before chunk colliders exist.
            GravityScale(0.0),
        ))
        .with_children(|parent| {
            parent.spawn((
                Name::new("PlayerCamera"),
                PlayerCamera,
                CameraPitch(0.0),
                mood_camera_bundle(),
                // Face local −Z (world forward once the parent yaws). Pitch is applied in `player_look`.
                Transform::from_translation(CAMERA_OFFSET),
            ));
        });
}

/// Turn gravity on once the chunk under the player has a real Avian `Collider`.
pub fn enable_player_on_terrain(
    mut commands: Commands,
    mut player_q: Query<(Entity, &mut Transform, &mut LinearVelocity), With<WaitingForTerrain>>,
    chunks: Query<&Chunk<VoxelTerrain>, With<Collider>>,
) {
    let Ok((entity, mut transform, mut velocity)) = player_q.single_mut() else {
        return;
    };

    let player_chunk = chunk_pos_containing(transform.translation);
    let ready = chunks.iter().any(|chunk| chunk.position == player_chunk);
    if !ready {
        return;
    }

    transform.translation.y = spawn_y_at(
        transform.translation.x.floor() as i32,
        transform.translation.z.floor() as i32,
    );
    *velocity = LinearVelocity::ZERO;
    commands
        .entity(entity)
        .insert(GravityScale(PLAYER_GRAVITY))
        .remove::<WaitingForTerrain>();
}

/// Yaw on the player body, pitch on the camera (clamped so the view cannot flip).
pub fn player_look(
    accumulated: Res<AccumulatedMouseMotion>,
    mut player_q: Query<&mut Transform, With<Player>>,
    mut camera_q: Query<(&mut Transform, &mut CameraPitch), (With<PlayerCamera>, Without<Player>)>,
) {
    let delta = accumulated.delta;
    if delta == Vec2::ZERO {
        return;
    }

    if let Ok(mut player_tf) = player_q.single_mut() {
        player_tf.rotate_y(-delta.x * MOUSE_SENS);
    }

    if let Ok((mut cam_tf, mut pitch)) = camera_q.single_mut() {
        // Mouse up (negative delta.y) increases pitch → look toward the sky.
        pitch.0 = (pitch.0 - delta.y * MOUSE_SENS).clamp(-PITCH_LIMIT, PITCH_LIMIT);
        cam_tf.translation = CAMERA_OFFSET;
        cam_tf.rotation = Quat::from_rotation_x(pitch.0);
    }
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

/// Attach static trimesh colliders once a terrain chunk actually has a `Mesh3d`.
///
/// `ChunkWillSpawn` alone is not enough: empty/full chunks fire it without a mesh,
/// and Avian's `TrimeshFromMesh` panics if `Mesh3d` is missing.
pub fn attach_chunk_colliders(
    mut commands: Commands,
    q: Query<
        Entity,
        (
            With<Chunk<VoxelTerrain>>,
            With<Mesh3d>,
            Without<RigidBody>,
        ),
    >,
) {
    for entity in &q {
        commands.entity(entity).insert((
            RigidBody::Static,
            ColliderConstructor::TrimeshFromMesh,
        ));
    }
}
