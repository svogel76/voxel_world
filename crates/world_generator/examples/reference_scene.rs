//! Art-directed hero-shot demo for the Blocky Forest mood + world scale.
//!
//! Semi-manual: fixed seeds + fixed world positions call the sub-generators
//! directly (not `generate_chunk`). Includes a 1.8 m player proxy, a ~20–25 m
//! hero tree, fern/bush undergrowth, and a fallen mossy log (1 voxel ≈ 1 m).
//!
//! Run:
//! ```text
//! cargo run -p world_generator --example reference_scene
//! ```
//!
//! Optional screenshot:
//! ```text
//! WORLD_GENERATOR_SCREENSHOT=path.png cargo run -p world_generator --example reference_scene
//! ```
//!
//! Controls (Bevy `FreeCamera`):
//! - WASD: move, Q/E: down/up, Shift: run, scroll: speed
//! - Right-click or M: capture mouse for look-around

use bevy::{
    anti_alias::taa::TemporalAntiAliasing,
    asset::RenderAssetUsages,
    camera::Hdr,
    camera_controller::free_camera::{FreeCamera, FreeCameraPlugin},
    core_pipeline::tonemapping::Tonemapping,
    light::{
        CascadeShadowConfigBuilder, FogVolume, VolumetricFog, VolumetricLight,
    },
    mesh::{Indices, PrimitiveTopology},
    pbr::ScreenSpaceAmbientOcclusion,
    post_process::bloom::Bloom,
    prelude::*,
    render::view::screenshot::{save_to_disk, Screenshot},
};
use glam::{IVec3, Vec2};
use grass_generator::{Area, GrassInstance, GrassParams, GrassVariant, VariantWeights};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rock_generator::RockParams;
use tree_generator::{CrossSectionShape, LeafPlacement, TreeParams, TurtleParams};
use world_generator::{SimpleNoiseTerrain, TerrainHeightSource};

const CLUMP_WIDTH: f32 = 0.7;
const CLUMP_HEIGHT: f32 = 1.1;
const GROUND_SEGMENTS: u32 = 40;

/// World scale: 1 voxel edge ≈ 1 meter (see README / Roadmap).
const PLAYER_HEIGHT_M: f32 = 1.8;
const PLAYER_WIDTH_M: f32 = 0.6;
const EYE_HEIGHT_M: f32 = 1.6;

/// Slight undulation — low enough that composition stays readable.
fn terrain() -> SimpleNoiseTerrain {
    SimpleNoiseTerrain {
        seed: 42,
        frequency: 0.06,
        amplitude: 1.2,
        base: 0.0,
    }
}

/// Ground bounds in world XZ (glam `Vec2` = (x, z)).
fn ground_area() -> Area {
    Area {
        min: Vec2::new(-14.0, -2.0),
        max: Vec2::new(14.0, 40.0),
    }
}

/// Frame trees: (world_x, world_z, seed). Side pillars along the corridor.
fn tree_placements() -> &'static [(f32, f32, u64)] {
    &[
        (-8.0, 8.0, 101),
        (8.0, 8.5, 201),
        (-8.5, 16.0, 102),
        (8.5, 16.5, 202),
        (-8.0, 24.0, 103),
        (8.0, 24.5, 203),
        (-10.0, 32.0, 104),
        (10.0, 32.5, 204),
    ]
}

/// One hero tree beside the path — height target ~20–25 m (logged at spawn).
fn hero_tree_placement() -> (f32, f32, u64) {
    (-6.0, 11.0, 901)
}

/// Fixed rock placements: side anchors near the camera, not in the sightline.
fn rock_placements() -> &'static [(f32, f32, u64)] {
    &[
        (5.0, 4.0, 401),
        (-4.5, 4.0, 402),
        (6.0, 9.0, 403),
    ]
}

/// Mid-size frame trees (still tall, secondary to the hero).
fn frame_tree_params() -> TreeParams {
    TreeParams {
        depth: 3,
        turtle: TurtleParams {
            step_length: 2.0,
            angle_degrees: 18.0,
            base_thickness: 4.0,
            taper_ratio: 0.8,
        },
        cross_section: CrossSectionShape::Cube,
        leaf_placement: LeafPlacement { crown_levels: 1 },
        ..TreeParams::generic_3d()
    }
}

/// Hero tree aimed at ~20–25 m crown height (1 unit = 1 m).
/// Calibrated: generic_3d, depth=3, step=2.5 → ~21 m wood / ~26 m crown (seed 901).
fn hero_tree_params() -> TreeParams {
    TreeParams {
        depth: 3,
        turtle: TurtleParams {
            step_length: 2.5,
            angle_degrees: 16.0,
            base_thickness: 5.0,
            taper_ratio: 0.82,
        },
        cross_section: CrossSectionShape::Cube,
        leaf_placement: LeafPlacement { crown_levels: 1 },
        ..TreeParams::generic_3d()
    }
}

/// Bush anchors: (world_x, world_z, seed) — keep |x| ≳ 2.5 so the path stays clear.
fn bush_placements() -> &'static [(f32, f32, u64)] {
    &[
        (-5.0, 5.0, 701),
        (-4.0, 9.0, 702),
        (-6.5, 12.0, 703),
        (-5.5, 17.0, 704),
        (4.5, 6.0, 711),
        (5.5, 10.0, 712),
        (4.0, 14.0, 713),
        (6.0, 18.0, 714),
        (-3.5, 7.5, 721),
        (3.8, 11.5, 722),
    ]
}

/// Dense fern-heavy undergrowth params (path center stays empty via area bounds).
fn fern_carpet_params(density: f32) -> GrassParams {
    GrassParams {
        density,
        variant_weights: VariantWeights {
            grass: 0.25,
            fern: 1.0,
        },
        ..GrassParams::default()
    }
}

fn rock_params() -> RockParams {
    RockParams {
        half_extent: 4,
        threshold: 0.4,
        radial_falloff: 0.35,
        ..RockParams::default()
    }
}

fn main() {
    let mut app = App::new();
    app.insert_resource(ClearColor(Color::srgb(0.06, 0.07, 0.08)))
        // Enough cool fill to read silhouettes; key light still carries the mood.
        .insert_resource(GlobalAmbientLight {
            color: Color::srgb(0.22, 0.26, 0.32),
            brightness: 55.0,
            ..default()
        })
        .add_plugins(DefaultPlugins.set(WindowPlugin {
            primary_window: Some(Window {
                title: "Reference Scene — Blocky Forest".into(),
                ..default()
            }),
            ..default()
        }))
        .add_plugins(FreeCameraPlugin)
        .add_systems(Startup, (setup_scene, spawn_help_text));

    if std::env::var_os("WORLD_GENERATOR_SCREENSHOT").is_some() {
        app.add_systems(Update, auto_screenshot_and_exit);
    }

    app.run();
}

fn auto_screenshot_and_exit(
    mut commands: Commands,
    mut frames: Local<u32>,
    mut exit: MessageWriter<AppExit>,
    mut captured: Local<bool>,
) {
    *frames += 1;
    if !*captured && *frames == 90 {
        let path = std::env::var("WORLD_GENERATOR_SCREENSHOT")
            .unwrap_or_else(|_| "reference_scene.png".into());
        eprintln!("saving screenshot to {path}");
        commands
            .spawn(Screenshot::primary_window())
            .observe(save_to_disk(path));
        *captured = true;
    }
    if *captured && *frames == 140 {
        exit.write(AppExit::Success);
    }
}

fn setup_scene(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let terrain = terrain();
    let area = ground_area();
    let frame_params = frame_tree_params();
    let hero_params = hero_tree_params();
    let rock_params = rock_params();

    let cube_mesh = meshes.add(Cuboid::new(1.0, 1.0, 1.0));
    let wood_material = materials.add(StandardMaterial {
        // Dark bark brown — must read against green canopy (Concept Art).
        base_color: Color::srgb(0.20, 0.11, 0.05),
        perceptual_roughness: 0.94,
        ..default()
    });
    let leaf_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.16, 0.40, 0.12),
        perceptual_roughness: 0.95,
        ..default()
    });
    let bush_material = materials.add(StandardMaterial {
        // Darker / more saturated than canopy leaf — reads as undergrowth.
        base_color: Color::srgb(0.10, 0.34, 0.09),
        perceptual_roughness: 0.96,
        ..default()
    });
    let moss_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.14, 0.38, 0.12),
        perceptual_roughness: 0.98,
        ..default()
    });
    let stone_material = materials.add(StandardMaterial {
        base_color: Color::srgb(0.42, 0.40, 0.38),
        perceptual_roughness: 0.95,
        ..default()
    });
    let ground_material = materials.add(StandardMaterial {
        // Lighter earth so the open path is a readable midtone under canopy shadow.
        base_color: Color::srgb(0.42, 0.34, 0.22),
        perceptual_roughness: 0.95,
        cull_mode: None,
        ..default()
    });
    let player_material = materials.add(StandardMaterial {
        // Bright accent so the 1.8 m proxy reads against dark trunks.
        base_color: Color::srgb(0.85, 0.55, 0.25),
        perceptual_roughness: 0.7,
        ..default()
    });
    let quad_mesh = meshes.add(Rectangle::new(CLUMP_WIDTH, CLUMP_HEIGHT));
    let grass_material = materials.add(variant_material(GrassVariant::Grass));
    let fern_material = materials.add(variant_material(GrassVariant::Fern));

    // Heightfield ground
    let ground_mesh = meshes.add(heightfield_mesh(&terrain, &area, GROUND_SEGMENTS));
    commands.spawn((
        Mesh3d(ground_mesh),
        MeshMaterial3d(ground_material),
        Transform::IDENTITY,
    ));

    // Scale anchor: ~1.8 m player proxy on the path edge (no controller).
    let proxy_x = 2.2;
    let proxy_z = 5.5;
    let ground_y = terrain.height_at(proxy_x, proxy_z);
    let proxy_mesh = meshes.add(Cuboid::new(PLAYER_WIDTH_M, PLAYER_HEIGHT_M, PLAYER_WIDTH_M));
    commands.spawn((
        Mesh3d(proxy_mesh),
        MeshMaterial3d(player_material),
        Transform::from_xyz(proxy_x, ground_y + PLAYER_HEIGHT_M * 0.5, proxy_z),
    ));

    // Hero tree (~20–25 m target) — primary scale statement vs. the proxy.
    let (hx, hz, hseed) = hero_tree_placement();
    let hero_voxels = tree_generator::generate(hseed, &hero_params);
    let wood_ys: Vec<i32> = hero_voxels
        .iter()
        .filter(|(_, b)| *b == tree_generator::BlockType::Wood)
        .map(|(p, _)| p.y)
        .collect();
    let crown_h = hero_voxels.iter().map(|(p, _)| p.y).max().unwrap_or(0)
        - hero_voxels.iter().map(|(p, _)| p.y).min().unwrap_or(0)
        + 1;
    let wood_h = wood_ys.iter().copied().max().unwrap_or(0)
        - wood_ys.iter().copied().min().unwrap_or(0)
        + 1;
    eprintln!(
        "reference_scene scale: player={PLAYER_HEIGHT_M}m eye={EYE_HEIGHT_M}m hero wood≈{wood_h}m crown≈{crown_h}m voxels={}",
        hero_voxels.len()
    );
    let hero_origin = world_origin(hx, hz, &terrain);
    spawn_voxels(
        &mut commands,
        &cube_mesh,
        &wood_material,
        &leaf_material,
        &stone_material,
        hero_voxels.into_iter().map(|(p, b)| {
            let block = match b {
                tree_generator::BlockType::Wood => BlockKind::Wood,
                tree_generator::BlockType::Leaf => BlockKind::Leaf,
            };
            (p + hero_origin, block)
        }),
    );

    // Frame trees (secondary height).
    let mut tree_voxel_count = 0usize;
    for &(x, z, seed) in tree_placements() {
        let voxels = tree_generator::generate(seed, &frame_params);
        tree_voxel_count += voxels.len();
        let origin = world_origin(x, z, &terrain);
        spawn_voxels(
            &mut commands,
            &cube_mesh,
            &wood_material,
            &leaf_material,
            &stone_material,
            voxels.into_iter().map(|(p, b)| {
                let block = match b {
                    tree_generator::BlockType::Wood => BlockKind::Wood,
                    tree_generator::BlockType::Leaf => BlockKind::Leaf,
                };
                (p + origin, block)
            }),
        );
    }

    // Hand-placed rocks (foreground)
    let mut rock_voxel_count = 0usize;
    for &(x, z, seed) in rock_placements() {
        let voxels = rock_generator::generate(seed, &rock_params);
        rock_voxel_count += voxels.len();
        let origin = world_origin(x, z, &terrain);
        spawn_voxels(
            &mut commands,
            &cube_mesh,
            &wood_material,
            &leaf_material,
            &stone_material,
            voxels.into_iter().map(|(p, _)| (p + origin, BlockKind::Stone)),
        );
    }

    // Layer 1 — fern carpet on corridor edges + denser strips at trunk feet.
    // Path center (|x| < ~2.5) stays clear for the sightline.
    let fern_params = fern_carpet_params(2.4);
    let trunk_fern_params = fern_carpet_params(3.2);
    let left_grass = grass_generator::generate(
        501,
        Area {
            min: Vec2::new(-11.0, 2.0),
            max: Vec2::new(-3.0, 24.0),
        },
        &fern_params,
    );
    let right_grass = grass_generator::generate(
        502,
        Area {
            min: Vec2::new(3.0, 2.0),
            max: Vec2::new(11.0, 24.0),
        },
        &fern_params,
    );
    // Narrow belts near the hero / frame trunks.
    let left_trunk_ferns = grass_generator::generate(
        511,
        Area {
            min: Vec2::new(-8.5, 9.0),
            max: Vec2::new(-4.5, 14.0),
        },
        &trunk_fern_params,
    );
    let right_trunk_ferns = grass_generator::generate(
        512,
        Area {
            min: Vec2::new(4.5, 8.0),
            max: Vec2::new(8.5, 13.0),
        },
        &trunk_fern_params,
    );
    let mut grass_count = 0usize;
    for mut instance in left_grass
        .into_iter()
        .chain(right_grass)
        .chain(left_trunk_ferns)
        .chain(right_trunk_ferns)
    {
        instance.position.y = terrain.height_at(instance.position.x, instance.position.z);
        spawn_cross_quad(
            &mut commands,
            &quad_mesh,
            &instance,
            &grass_material,
            &fern_material,
        );
        grass_count += 1;
    }

    // Layer 1 — hand-placed bush clusters (leaf voxels, ~1–2 m).
    let mut bush_voxel_count = 0usize;
    for &(x, z, seed) in bush_placements() {
        let origin_y = terrain.height_at(x, z).round() as i32;
        let origin = IVec3::new(x.round() as i32, origin_y, z.round() as i32);
        let voxels = bush_cluster_voxels(seed);
        bush_voxel_count += voxels.len();
        for pos in voxels {
            commands.spawn((
                Mesh3d(cube_mesh.clone()),
                MeshMaterial3d(bush_material.clone()),
                Transform::from_translation((pos + origin).as_vec3()),
            ));
        }
    }

    // Layer 2 — fallen mossy log along the right path edge (does not block sightline).
    let fallen_log = fallen_log_voxels();
    let log_moss = fallen_log_moss_voxels();
    for pos in &fallen_log {
        let y = terrain.height_at(pos.x as f32, pos.z as f32).round() as i32;
        let world = IVec3::new(pos.x, y + pos.y, pos.z);
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(wood_material.clone()),
            Transform::from_translation(world.as_vec3()),
        ));
    }
    for pos in &log_moss {
        let y = terrain.height_at(pos.x as f32, pos.z as f32).round() as i32;
        let world = IVec3::new(pos.x, y + pos.y, pos.z);
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(moss_material.clone()),
            Transform::from_translation(world.as_vec3()),
        ));
    }

    eprintln!(
        "reference_scene undergrowth: ferns/grass={grass_count} bush_voxels={bush_voxel_count} log_wood={} log_moss={}",
        fallen_log.len(),
        log_moss.len()
    );
    eprintln!(
        "reference_scene: frame_tree_voxels={tree_voxel_count} rock_voxels={rock_voxel_count}"
    );

    // Sparse overhead canopy with a center gap — casts shafts into the fog.
    spawn_canopy_slabs(
        &mut commands,
        &cube_mesh,
        &leaf_material,
        &terrain,
    );

    // Far bright clearing: lit ground patch + emissive portal so the sightline
    // ends in light (Concept Art light-hole), not clear-color void.
    let clearing_y = terrain.height_at(0.0, 36.0);
    let clearing_mat = materials.add(StandardMaterial {
        base_color: Color::srgb(0.95, 0.92, 0.82),
        emissive: LinearRgba::rgb(6.0, 5.5, 4.0),
        perceptual_roughness: 1.0,
        unlit: false,
        ..default()
    });
    let portal_mesh = meshes.add(Cuboid::new(16.0, 14.0, 1.0));
    commands.spawn((
        Mesh3d(portal_mesh),
        MeshMaterial3d(clearing_mat),
        Transform::from_xyz(0.0, clearing_y + 7.0, 37.0),
    ));
    commands.spawn((
        PointLight {
            intensity: 900_000.0,
            range: 50.0,
            color: Color::srgb(1.0, 0.95, 0.85),
            shadow_maps_enabled: false,
            ..default()
        },
        Transform::from_xyz(0.0, clearing_y + 8.0, 34.0),
    ));

    // Fog fills the corridor — god rays need a participating medium in view.
    commands.spawn((
        FogVolume {
            fog_color: Color::srgb(0.92, 0.94, 0.97),
            density_factor: 0.08,
            absorption: 0.12,
            scattering: 0.7,
            ..default()
        },
        Transform::from_xyz(0.0, 12.0, 18.0).with_scale(Vec3::new(32.0, 40.0, 50.0)),
    ));

    // Key light behind the far opening, shining toward the camera through the gap.
    let look_target = Vec3::new(0.0, 4.0, 16.0);
    commands.spawn((
        DirectionalLight {
            illuminance: 80_000.0,
            shadow_maps_enabled: true,
            color: Color::srgb(1.0, 0.96, 0.88),
            ..default()
        },
        Transform::from_xyz(1.5, 30.0, 48.0).looking_at(look_target, Vec3::Y),
        CascadeShadowConfigBuilder {
            num_cascades: 4,
            minimum_distance: 0.5,
            maximum_distance: 90.0,
            first_cascade_far_bound: 18.0,
            ..default()
        }
        .build(),
        VolumetricLight,
    ));

    // Cool fill so near trunks stay readable in the dark half of the shot.
    commands.spawn((
        DirectionalLight {
            illuminance: 5_500.0,
            shadow_maps_enabled: false,
            color: Color::srgb(0.40, 0.48, 0.60),
            ..default()
        },
        Transform::from_xyz(-10.0, 14.0, -2.0).looking_at(look_target, Vec3::Y),
    ));

    // True eye height (~1.6 m): feel small under the canopy, looking toward the light hole.
    let cam_x = 0.0;
    let cam_z = 3.0;
    let cam_y = terrain.height_at(cam_x, cam_z) + EYE_HEIGHT_M;
    commands.spawn((
        Camera3d::default(),
        Hdr,
        Msaa::Off,
        Tonemapping::TonyMcMapface,
        Bloom {
            intensity: 0.18,
            ..Bloom::NATURAL
        },
        VolumetricFog {
            ambient_intensity: 0.0,
            step_count: 72,
            ..default()
        },
        ScreenSpaceAmbientOcclusion::default(),
        TemporalAntiAliasing::default(),
        Transform::from_xyz(cam_x, cam_y, cam_z).looking_at(
            Vec3::new(0.0, cam_y + 1.5, 30.0),
            Vec3::Y,
        ),
        FreeCamera {
            sensitivity: 0.12,
            walk_speed: 6.0,
            run_speed: 16.0,
            friction: 20.0,
            ..default()
        },
    ));
}

/// Irregular leaf cluster (~1–2 m) relative to bush origin on the ground.
fn bush_cluster_voxels(seed: u64) -> Vec<IVec3> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut voxels = Vec::new();
    let height = rng.gen_range(1..=2);
    let radius = rng.gen_range(1..=2);
    for y in 0..=height {
        let layer_r = radius - (y / 2);
        let r = layer_r.max(1);
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz > r * r {
                    continue;
                }
                // Sparse holes so bushes don't look like solid cubes.
                if rng.gen_bool(0.72) {
                    voxels.push(IVec3::new(dx, y, dz));
                }
            }
        }
    }
    voxels
}

/// Fallen log along the right path edge: thick wood line from near-camera toward mid.
/// Positions use world XZ; Y is height above local terrain (0 = resting on ground).
fn fallen_log_voxels() -> Vec<IVec3> {
    // From (3, 6) toward (-1, 11) — ~8 m, sits beside the path without blocking the hole.
    let start = IVec3::new(3, 0, 6);
    let end = IVec3::new(-1, 0, 11);
    let mut voxels = Vec::new();
    let steps = 10;
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let cx = start.x as f32 + (end.x - start.x) as f32 * t;
        let cz = start.z as f32 + (end.z - start.z) as f32 * t;
        let center = IVec3::new(cx.round() as i32, 0, cz.round() as i32);
        // 2×2 cross-section (lying trunk).
        for dy in 0..2 {
            for d in -1i32..=0 {
                voxels.push(center + IVec3::new(d, dy, 0));
                voxels.push(center + IVec3::new(0, dy, d));
            }
            voxels.push(center + IVec3::new(0, dy, 0));
        }
    }
    voxels.sort_by_key(|p| (p.x, p.y, p.z));
    voxels.dedup();
    voxels
}

/// Moss patches on the upper face of the fallen log (placeholder tint, no textures).
fn fallen_log_moss_voxels() -> Vec<IVec3> {
    let log = fallen_log_voxels();
    let mut moss = Vec::new();
    let mut rng = StdRng::seed_from_u64(8801);
    for p in log {
        if p.y >= 1 && rng.gen_bool(0.55) {
            moss.push(IVec3::new(p.x, p.y + 1, p.z));
        }
    }
    moss.sort_by_key(|p| (p.x, p.y, p.z));
    moss.dedup();
    moss
}

/// Hand-placed leaf cubes forming a broken canopy over the path (gaps = shafts).
fn spawn_canopy_slabs(
    commands: &mut Commands,
    cube_mesh: &Handle<Mesh>,
    leaf: &Handle<StandardMaterial>,
    terrain: &impl TerrainHeightSource,
) {
    // (x, z) centers — leave |x| < 2 mostly open for the light hole.
    let slabs: &[(i32, i32)] = &[
        (-5, 9),
        (-4, 10),
        (-5, 11),
        (4, 9),
        (5, 10),
        (4, 11),
        (-6, 15),
        (-5, 16),
        (5, 15),
        (6, 16),
        (-4, 21),
        (-5, 22),
        (4, 21),
        (5, 22),
        // Thin bridge with a gap at x=0
        (-2, 12),
        (2, 12),
        (-2, 18),
        (2, 18),
    ];
    for &(x, z) in slabs {
        // High canopy (~18 m) so 20–25 m trees still read as towering pillars.
        let base_y = terrain.height_at(x as f32, z as f32).round() as i32 + 18;
        for dy in 0..3 {
            for dx in -1i32..=1 {
                for dz in -1i32..=1 {
                    if dx.abs() + dz.abs() > 2 {
                        continue;
                    }
                    let pos = IVec3::new(x + dx, base_y + dy, z + dz);
                    commands.spawn((
                        Mesh3d(cube_mesh.clone()),
                        MeshMaterial3d(leaf.clone()),
                        Transform::from_translation(pos.as_vec3()),
                    ));
                }
            }
        }
    }
}

#[derive(Clone, Copy)]
enum BlockKind {
    Wood,
    Leaf,
    Stone,
}

fn world_origin(x: f32, z: f32, terrain: &impl TerrainHeightSource) -> IVec3 {
    IVec3::new(
        x.round() as i32,
        terrain.height_at(x, z).round() as i32,
        z.round() as i32,
    )
}

fn spawn_voxels(
    commands: &mut Commands,
    cube_mesh: &Handle<Mesh>,
    wood: &Handle<StandardMaterial>,
    leaf: &Handle<StandardMaterial>,
    stone: &Handle<StandardMaterial>,
    voxels: impl Iterator<Item = (IVec3, BlockKind)>,
) {
    for (pos, kind) in voxels {
        let material = match kind {
            BlockKind::Wood => wood.clone(),
            BlockKind::Leaf => leaf.clone(),
            BlockKind::Stone => stone.clone(),
        };
        commands.spawn((
            Mesh3d(cube_mesh.clone()),
            MeshMaterial3d(material),
            Transform::from_translation(pos.as_vec3()),
        ));
    }
}

fn heightfield_mesh(
    terrain: &impl TerrainHeightSource,
    area: &Area,
    segments: u32,
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
            positions.push([x, y, z]);
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

fn variant_material(variant: GrassVariant) -> StandardMaterial {
    let base_color = match variant {
        GrassVariant::Grass => Color::srgb(0.22, 0.48, 0.14),
        GrassVariant::Fern => Color::srgb(0.12, 0.38, 0.12),
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
        GrassVariant::Grass => grass_material.clone(),
        GrassVariant::Fern => fern_material.clone(),
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
            "Reference Scene — undergrowth: ferns + bushes + fallen mossy log\n",
            "Scale: 1 block≈1m | proxy 1.8m | hero ~20–25m | eye ~1.6m | FreeCamera WASD\n",
            "Light: CSM + VolumetricFog/Light + Bloom + SSAO"
        ))],
    ));
}
