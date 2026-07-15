//! Bevy preview of the tree generator (Phases 1–4).
//!
//! Spawns four trees with different L-system / turtle parameters side by side.
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

use std::collections::HashMap;

use bevy::{
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    prelude::*,
};
use tree_generator::{
    add_leaves, interpret, voxelize_with_shape, BlockType, CrossSectionShape, IVec3,
    LeafPlacement, LSystemGrammar, TurtleParams,
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
    offset: Vec3,
    axiom: &'static str,
    rules: &'static [(&'static str, &'static str)],
    depth: u32,
    params: TurtleParams,
    wood_color: Color,
}

const TREE_PRESETS: &[TreePreset] = &[
    TreePreset {
        offset: Vec3::new(-27.0, 0.0, 0.0),
        axiom: "F",
        rules: &[("F", "F[+F]F[-F]F")],
        depth: 4,
        params: TurtleParams {
            step_length: 1.0,
            angle_degrees: 22.0,
            base_thickness: 2.0,
            taper_ratio: 0.72,
        },
        wood_color: Color::srgb(0.45, 0.28, 0.12),
    },
    TreePreset {
        offset: Vec3::new(-9.0, 0.0, 0.0),
        axiom: "F",
        rules: &[("F", "F[+F]F[-F]F")],
        depth: 3,
        params: TurtleParams {
            step_length: 1.0,
            angle_degrees: 35.0,
            base_thickness: 1.5,
            taper_ratio: 0.65,
        },
        wood_color: Color::srgb(0.55, 0.35, 0.18),
    },
    TreePreset {
        offset: Vec3::new(9.0, 0.0, 0.0),
        axiom: "F",
        rules: &[("F", "F[+F][-F]F")],
        depth: 4,
        params: TurtleParams {
            step_length: 1.0,
            angle_degrees: 30.0,
            base_thickness: 1.8,
            taper_ratio: 0.7,
        },
        wood_color: Color::srgb(0.38, 0.24, 0.10),
    },
    TreePreset {
        offset: Vec3::new(27.0, 0.0, 0.0),
        axiom: "F",
        rules: &[("F", "F[+F][&F][-F][^F]F")],
        depth: 3,
        params: TurtleParams {
            step_length: 1.0,
            angle_degrees: 28.0,
            base_thickness: 1.8,
            taper_ratio: 0.7,
        },
        wood_color: Color::srgb(0.42, 0.32, 0.22),
    },
];

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

    for preset in TREE_PRESETS {
        let wood_material = materials.add(StandardMaterial {
            base_color: preset.wood_color,
            perceptual_roughness: 0.9,
            ..default()
        });

        for (position, block_type) in generate_tree_voxels(preset, &settings) {
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
                "Preview: shape={} thickness={:.1} | wood + leaf clusters\n",
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

fn generate_tree_voxels(
    preset: &TreePreset,
    settings: &PreviewSettings,
) -> Vec<(IVec3, BlockType)> {
    let rules = preset
        .rules
        .iter()
        .map(|(symbol, replacement)| (symbol.chars().next().unwrap(), replacement.to_string()))
        .collect::<HashMap<_, _>>();

    let grammar = LSystemGrammar::new(preset.axiom, rules);
    let l_string = grammar.expand(preset.depth);

    let mut params = preset.params;
    params.base_thickness = settings.base_thickness;

    let segments = interpret(&l_string, &params);
    let offset = voxel_offset(preset.offset);
    let wood = voxelize_with_shape(&segments, settings.shape);
    let voxels = add_leaves(&wood, &segments, LeafPlacement::default());

    voxels
        .into_iter()
        .map(|(position, block_type)| (position + offset, block_type))
        .collect()
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
