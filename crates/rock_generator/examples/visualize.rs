//! Bevy preview of [`rock_generator::generate`].
//!
//! Spawns several boulders with different seeds side by side.
//!
//! Run:
//! ```text
//! cargo run -p rock_generator --example visualize
//! ```
//!
//! Controls (Bevy `FreeCamera`):
//! - WASD: move, Q/E: down/up, Shift: run, scroll: speed
//! - Right-click or M: capture mouse for look-around

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use rock_generator::{generate, BlockType, IVec3, RockParams};

struct BoulderPreset {
    seed: u64,
    offset: Vec3,
    params: RockParams,
    color: Color,
}

fn main() {
    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup_scene, spawn_help_text))
        .run();
}

fn boulder_presets() -> [BoulderPreset; 4] {
    let base = RockParams::default();
    [
        BoulderPreset {
            seed: 1,
            offset: Vec3::new(-18.0, 0.0, 0.0),
            params: RockParams {
                half_extent: 5,
                ..base.clone()
            },
            color: Color::srgb(0.45, 0.44, 0.42),
        },
        BoulderPreset {
            seed: 2,
            offset: Vec3::new(-6.0, 0.0, 0.0),
            params: RockParams {
                half_extent: 6,
                threshold: 0.42,
                radial_falloff: 0.5,
                ..base.clone()
            },
            color: Color::srgb(0.52, 0.48, 0.44),
        },
        BoulderPreset {
            seed: 3,
            offset: Vec3::new(6.0, 0.0, 0.0),
            params: RockParams {
                half_extent: 4,
                noise_frequency: 0.45,
                axis_scale_min: 0.55,
                axis_scale_max: 1.45,
                ..base.clone()
            },
            color: Color::srgb(0.38, 0.40, 0.36),
        },
        BoulderPreset {
            seed: 4,
            offset: Vec3::new(18.0, 0.0, 0.0),
            params: RockParams {
                half_extent: 7,
                threshold: 0.48,
                radial_falloff: 0.4,
                ..base
            },
            color: Color::srgb(0.55, 0.50, 0.46),
        },
    ]
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));

    for preset in boulder_presets() {
        let material = materials.add(StandardMaterial {
            base_color: preset.color,
            perceptual_roughness: 0.95,
            ..default()
        });

        let offset = voxel_offset(preset.offset);
        let voxels = generate(preset.seed, &preset.params);
        eprintln!(
            "boulder seed {}: {} stone voxels, extents {:?}",
            preset.seed,
            voxels.len(),
            rock_generator::axis_extents(preset.seed, &preset.params)
        );

        for (position, block_type) in voxels {
            assert_eq!(block_type, BlockType::Stone);
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::from_translation(voxel_to_bevy_vec3(position + offset)),
            ));
        }
    }

    let ground_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.28, 0.18),
        perceptual_roughness: 1.0,
        ..default()
    });
    commands.spawn((
        Mesh3d(
            meshes.add(
                Plane3d::new(Vec3::Y, Vec2::splat(1.0))
                    .mesh()
                    .size(60.0, 40.0),
            ),
        ),
        MeshMaterial3d(ground_material),
        Transform::from_xyz(0.0, -0.01, 0.0),
    ));

    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(10.0, 18.0, 12.0).looking_at(Vec3::ZERO, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 10.0, 32.0).looking_at(Vec3::new(0.0, 2.0, 0.0), Vec3::Y),
        FreeCamera {
            sensitivity: 0.15,
            walk_speed: 8.0,
            run_speed: 24.0,
            friction: 20.0,
            ..default()
        },
    ));
}

fn spawn_help_text(mut commands: Commands) {
    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
        children![Text::new(concat!(
            "Rock preview: generate() API, seeds 1-4 (L→R)\n",
            "FreeCamera: WASD move | Q/E up/down | Shift run | Scroll speed\n",
            "Right-click or M: mouse look"
        ))],
    ));
}

fn voxel_offset(offset: Vec3) -> IVec3 {
    IVec3::new(
        offset.x.round() as i32,
        offset.y.round() as i32,
        offset.z.round() as i32,
    )
}

fn voxel_to_bevy_vec3(position: IVec3) -> Vec3 {
    position.as_vec3()
}
