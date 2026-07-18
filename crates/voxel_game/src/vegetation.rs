//! Spawn one fixed `world_generator` chunk as cubes / grass cross-quads on the noise terrain.

use bevy::prelude::*;
use grass_generator::GrassVariant;
use world_generator::{generate_chunk, Area, WorldBlockType};

use crate::height::{WORLD_SEED, VoxelNoiseHeight};

/// Fixed area near the origin so Phase 1 stays easy to find in-game.
fn vegetation_area() -> Area {
    Area {
        min: Vec2::new(-8.0, -8.0),
        max: Vec2::new(8.0, 8.0),
    }
}

const COLOR_WOOD: Color = Color::srgb(0.42, 0.28, 0.16);
const COLOR_LEAF: Color = Color::srgb(0.22, 0.48, 0.20);
const COLOR_STONE: Color = Color::srgb(0.48, 0.48, 0.50);
const COLOR_GRASS: Color = Color::srgb(0.30, 0.55, 0.22);
const COLOR_FERN: Color = Color::srgb(0.18, 0.42, 0.22);

pub fn spawn_vegetation_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
) {
    let height = VoxelNoiseHeight::default_world();
    let area = vegetation_area();
    let chunk = generate_chunk(WORLD_SEED, area, &height);

    let cube = meshes.add(Cuboid::from_length(1.0));
    let quad = meshes.add(Rectangle::new(1.0, 1.0));

    let wood_mat = materials.add(unlitish(COLOR_WOOD));
    let leaf_mat = materials.add(unlitish(COLOR_LEAF));
    let stone_mat = materials.add(unlitish(COLOR_STONE));
    let grass_mat = materials.add(cross_material(COLOR_GRASS));
    let fern_mat = materials.add(cross_material(COLOR_FERN));

    let root = commands
        .spawn((
            Name::new("VegetationChunk"),
            Transform::default(),
            Visibility::default(),
        ))
        .id();

    for (pos, block) in &chunk.tree_and_rock_voxels {
        let mat = match block {
            WorldBlockType::Wood => wood_mat.clone(),
            WorldBlockType::Leaf => leaf_mat.clone(),
            WorldBlockType::Stone => stone_mat.clone(),
        };
        let child = commands
            .spawn((
                Mesh3d(cube.clone()),
                MeshMaterial3d(mat),
                Transform::from_translation(pos.as_vec3()),
            ))
            .id();
        commands.entity(root).add_child(child);
    }

    for instance in &chunk.grass_instances {
        let mat = match instance.variant {
            GrassVariant::Grass => grass_mat.clone(),
            GrassVariant::Fern => fern_mat.clone(),
        };
        let holder = commands
            .spawn((
                Transform::from_translation(instance.position)
                    .with_rotation(Quat::from_rotation_y(instance.rotation_y))
                    .with_scale(Vec3::splat(instance.scale)),
                Visibility::default(),
            ))
            .with_children(|parent| {
                parent.spawn((
                    Mesh3d(quad.clone()),
                    MeshMaterial3d(mat.clone()),
                    Transform::from_translation(Vec3::new(0.0, 0.5, 0.0)),
                ));
                parent.spawn((
                    Mesh3d(quad.clone()),
                    MeshMaterial3d(mat),
                    Transform::from_translation(Vec3::new(0.0, 0.5, 0.0))
                        .with_rotation(Quat::from_rotation_y(std::f32::consts::FRAC_PI_2)),
                ));
            })
            .id();
        commands.entity(root).add_child(holder);
    }

    info!(
        "spawned vegetation chunk: {} voxels, {} grass in {:?}",
        chunk.tree_and_rock_voxels.len(),
        chunk.grass_instances.len(),
        area
    );
}

fn unlitish(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        perceptual_roughness: 0.9,
        ..default()
    }
}

fn cross_material(color: Color) -> StandardMaterial {
    StandardMaterial {
        base_color: color,
        alpha_mode: AlphaMode::Mask(0.1),
        cull_mode: None,
        perceptual_roughness: 0.9,
        ..default()
    }
}
