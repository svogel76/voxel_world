//! Bevy preview of [`world_generator::generate_chunk`].
//!
//! Three small 10×10 areas side by side (Forest / Rocky / Clearing).
//! Each panel has a heightfield ground sampled from its `TerrainHeightSource`.
//!
//! Run:
//! ```text
//! cargo run -p world_generator --example visualize
//! ```
//!
//! Controls (Bevy `FreeCamera`):
//! - WASD: move, Q/E: down/up, Shift: run, scroll: speed
//! - Right-click or M: capture mouse for look-around

use bevy::{
    asset::RenderAssetUsages,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    mesh::{Indices, PrimitiveTopology},
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use glam::Vec2;
use world_generator::{
    classify, generate_chunk, Area, Biome, ConstantHeight, GrassInstance, SimpleNoiseTerrain,
    TerrainHeightSource, WorldBlockType, ROCKY_MIN_HEIGHT,
};

const AREA_SIZE: f32 = 10.0;
const CLUMP_WIDTH: f32 = 0.7;
const CLUMP_HEIGHT: f32 = 1.1;
/// Grid resolution for the terrain heightfield (segments per axis).
const GROUND_SEGMENTS: u32 = 20;

/// Local sampling area for every panel (center at (5, 5)).
fn local_area() -> Area {
    Area {
        min: Vec2::new(0.0, 0.0),
        max: Vec2::new(AREA_SIZE, AREA_SIZE),
    }
}

enum PanelTerrain {
    Noise(SimpleNoiseTerrain),
    Flat(ConstantHeight),
}

impl TerrainHeightSource for PanelTerrain {
    fn height_at(&self, x: f32, z: f32) -> f32 {
        match self {
            Self::Noise(t) => t.height_at(x, z),
            Self::Flat(t) => t.height_at(x, z),
        }
    }
}

struct Panel {
    label: &'static str,
    seed: u64,
    /// World-space XZ shift applied when spawning (panels sit side by side).
    display_offset: Vec3,
    terrain: PanelTerrain,
    expected_biome: Biome,
}

fn panels() -> [Panel; 3] {
    // SimpleNoiseTerrain: visible height variation for grass / tree bases.
    // Amplitude kept low so the area center stays below ROCKY_MIN_HEIGHT.
    let forest_clearing_terrain = SimpleNoiseTerrain {
        seed: 7,
        frequency: 0.08,
        amplitude: 3.0,
        base: 3.0,
    };

    [
        Panel {
            label: "Forest",
            // At local center (5,5) with this terrain, seed 0 → Forest.
            seed: 0,
            display_offset: Vec3::new(-18.0, 0.0, 0.0),
            terrain: PanelTerrain::Noise(forest_clearing_terrain),
            expected_biome: Biome::Forest,
        },
        Panel {
            label: "Rocky",
            seed: 1,
            display_offset: Vec3::new(0.0, 0.0, 0.0),
            // Constant high ground → always Rocky (classify ignores moisture).
            terrain: PanelTerrain::Flat(ConstantHeight(ROCKY_MIN_HEIGHT + 2.0)),
            expected_biome: Biome::Rocky,
        },
        Panel {
            label: "Clearing",
            // Same terrain family as Forest; seed 1 → Clearing at (5,5).
            seed: 1,
            display_offset: Vec3::new(18.0, 0.0, 0.0),
            terrain: PanelTerrain::Noise(forest_clearing_terrain),
            expected_biome: Biome::Clearing,
        },
    ]
}

fn main() {
    let mut app = App::new();
    app.add_plugins(DefaultPlugins)
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup_scene, spawn_help_text));

    // Optional one-shot capture: WORLD_GENERATOR_SCREENSHOT=path.png
    if std::env::var_os("WORLD_GENERATOR_SCREENSHOT").is_some() {
        app.add_systems(Update, auto_screenshot_and_exit);
    }

    app.run();
}

/// After a few frames (so the heightfield has rendered), save a PNG and quit.
fn auto_screenshot_and_exit(
    mut commands: Commands,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
    mut captured: Local<bool>,
) {
    *frames += 1;
    if !*captured && *frames == 45 {
        let path = std::env::var("WORLD_GENERATOR_SCREENSHOT")
            .unwrap_or_else(|_| "forest_ground_contact.png".into());
        eprintln!("saving screenshot to {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        *captured = true;
    }
    // Give the async capture a few frames to finish writing.
    if *captured && *frames == 90 {
        exit.write(AppExit::Success);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let wood_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.45, 0.28, 0.12),
        perceptual_roughness: 0.9,
        ..default()
    });
    let leaf_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.22, 0.58, 0.18),
        perceptual_roughness: 0.95,
        ..default()
    });
    let stone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.48, 0.46, 0.44),
        perceptual_roughness: 0.95,
        ..default()
    });
    let quad_mesh = meshes.add(Rectangle::new(CLUMP_WIDTH, CLUMP_HEIGHT));
    let grass_material = materials.add(variant_material(grass_generator::GrassVariant::Grass));
    let fern_material = materials.add(variant_material(grass_generator::GrassVariant::Fern));
    let ground_material = materials.add(StandardMaterial {
        // Bright earth tone — must stay readable under canopy shadows.
        base_color: Color::srgb(0.62, 0.52, 0.34),
        perceptual_roughness: 0.9,
        cull_mode: None,
        ..default()
    });

    let area = local_area();
    let center = Vec2::new(AREA_SIZE * 0.5, AREA_SIZE * 0.5);

    for panel in panels() {
        let biome = classify(center.x, center.y, panel.seed, &panel.terrain);
        assert_eq!(
            biome, panel.expected_biome,
            "panel {} seed {} expected {:?}, got {:?}",
            panel.label, panel.seed, panel.expected_biome, biome
        );

        let content = generate_chunk(panel.seed, area, &panel.terrain);
        let stone = content
            .tree_and_rock_voxels
            .iter()
            .filter(|(_, b)| *b == WorldBlockType::Stone)
            .count();
        let wood = content
            .tree_and_rock_voxels
            .iter()
            .filter(|(_, b)| *b == WorldBlockType::Wood)
            .count();
        let leaf = content
            .tree_and_rock_voxels
            .iter()
            .filter(|(_, b)| *b == WorldBlockType::Leaf)
            .count();

        eprintln!(
            "panel {}: biome={:?} seed={} voxels={} (wood={} leaf={} stone={}) grass={}",
            panel.label,
            biome,
            panel.seed,
            content.tree_and_rock_voxels.len(),
            wood,
            leaf,
            stone,
            content.grass_instances.len(),
        );

        let ground_mesh = meshes.add(heightfield_mesh(
            &panel.terrain,
            &area,
            GROUND_SEGMENTS,
            panel.display_offset,
        ));
        commands.spawn((
            Mesh3d(ground_mesh),
            MeshMaterial3d(ground_material.clone()),
            Transform::IDENTITY,
        ));

        for (pos, block) in content.tree_and_rock_voxels {
            let material = match block {
                WorldBlockType::Wood => wood_material.clone(),
                WorldBlockType::Leaf => leaf_material.clone(),
                WorldBlockType::Stone => stone_material.clone(),
            };
            let translation = pos.as_vec3() + panel.display_offset;
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_translation(translation),
            ));
        }

        for instance in content.grass_instances {
            let shifted = GrassInstance {
                position: instance.position + panel.display_offset,
                ..instance
            };
            spawn_cross_quad(
                &mut commands,
                &quad_mesh,
                &shifted,
                &grass_material,
                &fern_material,
            );
        }
    }

    // True side view of Forest: same X as panel center, standing outside on +Z,
    // looking horizontally toward the patch so the heightfield reads as a band
    // and trunks/grass meet it (not a worm's-eye under the canopy).
    let forest_look = Vec3::new(-13.0, 3.5, 5.0);
    commands.spawn((
        DirectionalLight {
            illuminance: 18_000.0,
            shadow_maps_enabled: true,
            ..default()
        },
        Transform::from_xyz(-8.0, 14.0, 10.0).looking_at(forest_look, Vec3::Y),
    ));

    commands.spawn((
        Camera3d::default(),
        Transform::from_xyz(-13.0, 5.5, 22.0).looking_at(forest_look, Vec3::Y),
        FreeCamera {
            sensitivity: 0.15,
            walk_speed: 10.0,
            run_speed: 28.0,
            friction: 20.0,
            ..default()
        },
    ));
}

/// Build a triangle mesh whose vertex Y values come from `terrain.height_at`.
fn heightfield_mesh(
    terrain: &impl TerrainHeightSource,
    area: &Area,
    segments: u32,
    display_offset: Vec3,
) -> Mesh {
    let seg = segments as usize;
    let stride = seg + 1;
    let mut positions: Vec<[f32; 3]> = Vec::with_capacity(stride * stride);
    let mut uvs: Vec<[f32; 2]> = Vec::with_capacity(stride * stride);

    for iz in 0..=seg {
        for ix in 0..=seg {
            let u = ix as f32 / seg as f32;
            let v = iz as f32 / seg as f32;
            let x = area.min.x + u * area.width();
            let z = area.min.y + v * area.depth();
            let y = terrain.height_at(x, z);
            positions.push([
                x + display_offset.x,
                y + display_offset.y,
                z + display_offset.z,
            ]);
            uvs.push([u, v]);
        }
    }

    let mut indices: Vec<u32> = Vec::with_capacity(seg * seg * 6);
    for iz in 0..seg {
        for ix in 0..seg {
            let i0 = (iz * stride + ix) as u32;
            let i1 = i0 + 1;
            let i2 = i0 + stride as u32;
            let i3 = i2 + 1;
            // Winding so normals point roughly upward for a heightfield.
            indices.extend_from_slice(&[i0, i2, i1, i1, i2, i3]);
        }
    }

    let normals = compute_smooth_normals(&positions, &indices);

    Mesh::new(
        PrimitiveTopology::TriangleList,
        RenderAssetUsages::default(),
    )
    .with_inserted_attribute(Mesh::ATTRIBUTE_POSITION, positions)
    .with_inserted_attribute(Mesh::ATTRIBUTE_NORMAL, normals)
    .with_inserted_attribute(Mesh::ATTRIBUTE_UV_0, uvs)
    .with_inserted_indices(Indices::U32(indices))
}

fn compute_smooth_normals(positions: &[[f32; 3]], indices: &[u32]) -> Vec<[f32; 3]> {
    let mut normals = vec![[0.0, 0.0, 0.0]; positions.len()];
    for tri in indices.chunks_exact(3) {
        let i0 = tri[0] as usize;
        let i1 = tri[1] as usize;
        let i2 = tri[2] as usize;
        let p0 = Vec3::from_array(positions[i0]);
        let p1 = Vec3::from_array(positions[i1]);
        let p2 = Vec3::from_array(positions[i2]);
        let n = (p1 - p0).cross(p2 - p0);
        for i in [i0, i1, i2] {
            normals[i][0] += n.x;
            normals[i][1] += n.y;
            normals[i][2] += n.z;
        }
    }
    for n in &mut normals {
        let v = Vec3::from_array(*n);
        *n = if v.length_squared() > 0.0 {
            v.normalize().to_array()
        } else {
            [0.0, 1.0, 0.0]
        };
    }
    normals
}

fn variant_material(variant: grass_generator::GrassVariant) -> StandardMaterial {
    let base_color = match variant {
        grass_generator::GrassVariant::Grass => Color::srgb(0.28, 0.62, 0.18),
        grass_generator::GrassVariant::Fern => Color::srgb(0.16, 0.48, 0.14),
    };
    StandardMaterial {
        base_color,
        double_sided: true,
        perceptual_roughness: 0.95,
        ..default()
    }
}

fn spawn_cross_quad(
    commands: &mut Commands,
    quad_mesh: &Handle<Mesh>,
    instance: &GrassInstance,
    grass_material: &Handle<StandardMaterial>,
    fern_material: &Handle<StandardMaterial>,
) {
    let material = match instance.variant {
        grass_generator::GrassVariant::Grass => grass_material.clone(),
        grass_generator::GrassVariant::Fern => fern_material.clone(),
    };

    commands
        .spawn((
            Transform::from_translation(instance.position)
                .with_rotation(Quat::from_rotation_y(instance.rotation_y))
                .with_scale(Vec3::splat(instance.scale)),
            Visibility::default(),
        ))
        .with_children(|parent| {
            parent.spawn((
                Mesh3d(quad_mesh.clone()),
                MeshMaterial3d(material.clone()),
                Transform::IDENTITY,
            ));
            parent.spawn((
                Mesh3d(quad_mesh.clone()),
                MeshMaterial3d(material),
                Transform::from_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
            ));
        });
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
            "World preview: generate_chunk(), heightfield ground from height_at()\n",
            "Camera starts on Forest (side view) | Rocky center | Clearing right\n",
            "Wood=brown Leaf=green Stone=grey | grass=cross-quads\n",
            "FreeCamera: WASD | Q/E | Shift | Scroll | Right-click/M look"
        ))],
    ));
}
