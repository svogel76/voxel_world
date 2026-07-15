//! Bevy preview of the tree generator via [`tree_generator::generate`].
//!
//! Spawns four trees with different `TreeParams` presets side by side.
//! Each tree uses a fixed seed passed to `generate()`.
//!
//! Run:
//! ```text
//! cargo run -p tree_generator --example visualize
//! cargo run -p tree_generator --example visualize -- --shape cube --thickness 4
//! cargo run -p tree_generator --example visualize -- --shape sphere --thickness 2
//! ```
//!
//! Controls (Bevy `FreeCamera`):
//! - WASD: move, Q/E: down/up, Shift: run, scroll: speed
//! - Right-click or M: capture mouse for look-around

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use tree_generator::{
    generate, BlockType, CrossSectionShape, IVec3, TreeParams, TurtleParams,
};

#[derive(Resource, Clone, Copy)]
struct PreviewSettings {
    shape: CrossSectionShape,
    base_thickness: f32,
}

impl Default for PreviewSettings {
    fn default() -> Self {
        Self {
            shape: CrossSectionShape::Cube,
            base_thickness: 4.0,
        }
    }
}

fn main() {
    let settings = PreviewSettings::from_args();

    App::new()
        .add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .insert_resource(settings)
        .add_systems(Startup, (setup_scene, spawn_help_text))
        .run();
}

impl PreviewSettings {
    fn from_args() -> Self {
        let mut settings = Self::default();
        let mut args = std::env::args().skip(1);

        while let Some(arg) = args.next() {
            match arg.as_str() {
                "--shape" => {
                    if let Some(value) = args.next() {
                        settings.shape = parse_shape(&value);
                    }
                }
                "--thickness" => {
                    if let Some(value) = args.next() {
                        if let Ok(thickness) = value.parse::<f32>() {
                            settings.base_thickness = thickness;
                        }
                    }
                }
                other => eprintln!("unknown argument: {other}"),
            }
        }

        settings
    }
}

fn parse_shape(value: &str) -> CrossSectionShape {
    match value.to_ascii_lowercase().as_str() {
        "cube" | "square" => CrossSectionShape::Cube,
        "sphere" | "ball" => CrossSectionShape::Sphere,
        other => {
            eprintln!("unknown shape '{other}', using cube");
            CrossSectionShape::Cube
        }
    }
}

struct TreePreset {
    seed: u64,
    offset: Vec3,
    params: TreeParams,
    wood_color: Color,
}

fn tree_presets(settings: &PreviewSettings) -> [TreePreset; 4] {
    [
        TreePreset {
            seed: 1,
            offset: Vec3::new(-27.0, 0.0, 0.0),
            params: TreeParams {
                turtle: TurtleParams {
                    angle_degrees: 22.0,
                    base_thickness: settings.base_thickness,
                    ..TreeParams::generic_2d().turtle
                },
                cross_section: settings.shape,
                depth: 4,
                ..TreeParams::generic_2d()
            },
            wood_color: Color::srgb(0.45, 0.28, 0.12),
        },
        TreePreset {
            seed: 2,
            offset: Vec3::new(-9.0, 0.0, 0.0),
            params: TreeParams {
                turtle: TurtleParams {
                    angle_degrees: 35.0,
                    base_thickness: settings.base_thickness,
                    taper_ratio: 0.65,
                    ..TreeParams::generic_2d().turtle
                },
                cross_section: settings.shape,
                depth: 3,
                ..TreeParams::generic_2d()
            },
            wood_color: Color::srgb(0.55, 0.35, 0.18),
        },
        TreePreset {
            seed: 3,
            offset: Vec3::new(9.0, 0.0, 0.0),
            params: TreeParams {
                turtle: TurtleParams {
                    angle_degrees: 30.0,
                    base_thickness: settings.base_thickness,
                    taper_ratio: 0.7,
                    ..TreeParams::generic_2d().turtle
                },
                cross_section: settings.shape,
                depth: 4,
                ..TreeParams::generic_2d()
            },
            wood_color: Color::srgb(0.38, 0.24, 0.10),
        },
        TreePreset {
            seed: 4,
            offset: Vec3::new(27.0, 0.0, 0.0),
            params: TreeParams {
                turtle: TurtleParams {
                    base_thickness: settings.base_thickness,
                    ..TreeParams::generic_3d().turtle
                },
                cross_section: settings.shape,
                ..TreeParams::generic_3d()
            },
            wood_color: Color::srgb(0.42, 0.32, 0.22),
        },
    ]
}

fn setup_scene(
    settings: Res<PreviewSettings>,
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let leaf_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.58, 0.18),
        perceptual_roughness: 0.95,
        ..default()
    });

    for preset in tree_presets(&settings) {
        let wood_material = materials.add(StandardMaterial {
            base_color: preset.wood_color,
            perceptual_roughness: 0.9,
            ..default()
        });

        let offset = voxel_offset(preset.offset);
        let voxels = generate(preset.seed, &preset.params)
            .into_iter()
            .map(|(position, block_type)| (position + offset, block_type))
            .collect::<Vec<_>>();

        if preset.seed == 1 {
            eprintln!(
                "determinism fingerprint tree seed 1: {}",
                voxel_fingerprint(&voxels)
            );
        }

        for (position, block_type) in voxels {
            let material = match block_type {
                BlockType::Wood => wood_material.clone(),
                BlockType::Leaf => leaf_material.clone(),
            };

            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(voxel_to_bevy_vec3(position)),
            ));
        }
    }

    commands.spawn((
        DirectionalLight {
            illuminance: 14_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(8.0, 20.0, 12.0).looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(0.0, 18.0, 52.0).looking_at(Vec3::new(0.0, 10.0, 0.0), Vec3::Y),
        FreeCamera {
            sensitivity: 0.15,
            walk_speed: 8.0,
            run_speed: 24.0,
            friction: 20.0,
            ..default()
        },
    ));
}

fn spawn_help_text(mut commands: Commands, settings: Res<PreviewSettings>) {
    let shape_label = match settings.shape {
        CrossSectionShape::Sphere => "sphere",
        CrossSectionShape::Cube => "cube",
    };

    commands.spawn((
        Node {
            position_type: PositionType::Absolute,
            bottom: px(12),
            left: px(12),
            ..default()
        },
        children![Text::new(format!(
            concat!(
                "Preview: shape={} thickness={:.1} | generate() API, seeds 1-4\n",
                "CLI: --shape sphere|cube  --thickness 2|4\n",
                "FreeCamera: WASD move | Q/E up/down | Shift run | Scroll speed\n",
                "Right-click or M: mouse look\n",
                "Trees (L→R): narrow 2D | wide 2D | forked 2D | 3D (uses &/^ pitch)"
            ),
            shape_label,
            settings.base_thickness
        ))],
    ));
}

fn voxel_fingerprint(voxels: &[(IVec3, BlockType)]) -> u64 {
    let mut hash = voxels.len() as u64;
    for (position, block_type) in voxels {
        hash = hash
            .wrapping_mul(1_000_003)
            .wrapping_add(position.x as u64)
            .wrapping_mul(1_000_003)
            .wrapping_add(position.y as u64)
            .wrapping_mul(1_000_003)
            .wrapping_add(position.z as u64)
            .wrapping_mul(1_000_003)
            .wrapping_add(match block_type {
                BlockType::Wood => 0,
                BlockType::Leaf => 1,
            });
    }
    hash
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
