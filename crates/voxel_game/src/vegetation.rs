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

pub fn spawn_vegetation_chunk(
    mut commands: Commands,
    mut meshes: ResMut<Assets<Mesh>>,
    mut materials: ResMut<Assets<StandardMaterial>>,
    asset_server: Res<AssetServer>,
) {
    let height = VoxelNoiseHeight::default_world();
    let area = vegetation_area();
    let chunk = generate_chunk(WORLD_SEED, area, &height);

    let cube = meshes.add(Cuboid::from_length(1.0));
    let quad = meshes.add(Rectangle::new(1.0, 1.0));

    let wood_mat = materials.add(textured("textures/wood.png", &asset_server, false));
    let leaf_mat = materials.add(textured("textures/leaf.png", &asset_server, false));
    let stone_mat = materials.add(textured("textures/stone.png", &asset_server, false));
    // Moss tile: ferns for now; dedicated moss blocks come with Phase-3 blending.
    let grass_mat = materials.add(textured("textures/leaf.png", &asset_server, true));
    let fern_mat = materials.add(textured("textures/moss.png", &asset_server, true));

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
                // Bevy Cuboid is centered; +0.5 aligns with bvw cells [i, i+1].
                Transform::from_translation(pos.as_vec3() + Vec3::splat(0.5)),
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

fn textured(path: &'static str, assets: &AssetServer, masked: bool) -> StandardMaterial {
    let mut mat = StandardMaterial {
        base_color_texture: Some(assets.load(path)),
        perceptual_roughness: 0.9,
        ..default()
    };
    if masked {
        mat.alpha_mode = AlphaMode::Mask(0.1);
        mat.cull_mode = None;
    }
    mat
}
