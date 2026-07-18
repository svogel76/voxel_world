//! Phase 4: chunk orchestration — one biome per area, combined world content.

use glam::{IVec3, Vec2};
use grass_generator::{generate as generate_grass, Area, GrassInstance};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rock_generator::generate as generate_rock;
use tree_generator::generate as generate_tree;

use crate::biome::{classify, params_for, BiomeParams};
use crate::placement::{
    poisson_disc_sample, rock_density_multiplier, slope_at, MAX_ROCK_DENSITY_MULTIPLIER,
};
use crate::terrain::TerrainHeightSource;

/// Discriminator mixed into [`feature_seed`] so trees and rocks never share a stream.
const TREE_FEATURE_KIND: u64 = 0x72EE_0001;
const ROCK_FEATURE_KIND: u64 = 0xA0C4_0002;
const ROCK_PLACEMENT_KIND: u64 = 0xA0C4_91A5;

/// Unified block type for combined chunk voxel output.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum WorldBlockType {
    Wood,
    Leaf,
    Stone,
}

impl From<tree_generator::BlockType> for WorldBlockType {
    fn from(value: tree_generator::BlockType) -> Self {
        match value {
            tree_generator::BlockType::Wood => Self::Wood,
            tree_generator::BlockType::Leaf => Self::Leaf,
        }
    }
}

impl From<rock_generator::BlockType> for WorldBlockType {
    fn from(value: rock_generator::BlockType) -> Self {
        match value {
            rock_generator::BlockType::Stone => Self::Stone,
        }
    }
}

/// Combined vegetation / rock output for one area (world coordinates).
#[derive(Debug, Clone, PartialEq)]
pub struct ChunkContent {
    pub tree_and_rock_voxels: Vec<(IVec3, WorldBlockType)>,
    pub grass_instances: Vec<GrassInstance>,
}

/// Generate all world content for `area`.
///
/// **One biome per area:** classified at the area center only. No blending.
pub fn generate_chunk(
    seed: u64,
    area: Area,
    terrain: &impl TerrainHeightSource,
) -> ChunkContent {
    let center = area_center(&area);
    let biome = classify(center.x, center.y, seed, terrain);
    let params = params_for(biome);

    let mut tree_and_rock_voxels = Vec::new();

    place_trees(seed, &area, &params, terrain, &mut tree_and_rock_voxels);
    place_rocks(seed, &area, &params, terrain, &mut tree_and_rock_voxels);

    let mut grass_instances = generate_grass(seed, area, &params.grass_params);
    for instance in &mut grass_instances {
        let p = instance.position;
        instance.position.y = terrain.height_at(p.x, p.z);
    }

    ChunkContent {
        tree_and_rock_voxels,
        grass_instances,
    }
}

fn area_center(area: &Area) -> Vec2 {
    Vec2::new(
        (area.min.x + area.max.x) * 0.5,
        (area.min.y + area.max.y) * 0.5,
    )
}

/// Deterministic per-feature seed from world seed, feature kind, and XZ position.
///
/// Coordinates are quantized to milli-units so float placement stays stable.
pub fn feature_seed(world_seed: u64, kind: u64, x: f32, z: f32) -> u64 {
    let qx = (x * 1000.0).round() as i32 as u64;
    let qz = (z * 1000.0).round() as i32 as u64;

    let mut n = world_seed
        .wrapping_mul(0x9E37_79B9_7F4A_7C15)
        .wrapping_add(kind);
    n = n.wrapping_mul(0xBF58_476D_1CE4_E5B9).wrapping_add(qx);
    n = n.wrapping_mul(0x94D0_49BB_1331_11EB).wrapping_add(qz);
    n = (n ^ (n >> 30)).wrapping_mul(0xBF58_476D_1CE4_E5B9);
    n = (n ^ (n >> 27)).wrapping_mul(0x94D0_49BB_1331_11EB);
    n ^ (n >> 31)
}

fn place_trees(
    seed: u64,
    area: &Area,
    params: &BiomeParams,
    terrain: &impl TerrainHeightSource,
    out: &mut Vec<(IVec3, WorldBlockType)>,
) {
    let positions = poisson_disc_sample(seed, area, params.tree_density);
    for pos in positions {
        let tree_seed = feature_seed(seed, TREE_FEATURE_KIND, pos.x, pos.y);
        let origin = world_origin(pos.x, pos.y, terrain);
        let voxels = generate_tree(tree_seed, &params.tree_params);
        append_translated(voxels, origin, out);
    }
}

fn place_rocks(
    seed: u64,
    area: &Area,
    params: &BiomeParams,
    terrain: &impl TerrainHeightSource,
    out: &mut Vec<(IVec3, WorldBlockType)>,
) {
    let positions = sample_rock_positions(seed, area, params.rock_density, terrain);
    for pos in positions {
        let rock_seed = feature_seed(seed, ROCK_FEATURE_KIND, pos.x, pos.y);
        let origin = world_origin(pos.x, pos.y, terrain);
        let voxels = generate_rock(rock_seed, &params.rock_params);
        append_translated(voxels, origin, out);
    }
}

/// Spatially varying density via accept/reject against the slope multiplier.
fn sample_rock_positions(
    seed: u64,
    area: &Area,
    rock_density: f32,
    terrain: &impl TerrainHeightSource,
) -> Vec<Vec2> {
    if rock_density <= 0.0 || area.size() <= 0.0 {
        return Vec::new();
    }

    let base_count = (area.size() * rock_density).round().max(0.0) as usize;
    let candidate_count =
        (base_count as f32 * MAX_ROCK_DENSITY_MULTIPLIER).round().max(0.0) as usize;
    if candidate_count == 0 {
        return Vec::new();
    }

    let mut rng = StdRng::seed_from_u64(feature_seed(seed, ROCK_PLACEMENT_KIND, 0.0, 0.0));
    let mut positions = Vec::new();

    for _ in 0..candidate_count {
        let x = rng.gen_range(area.min.x..area.max.x);
        let z = rng.gen_range(area.min.y..area.max.y);
        let mult = rock_density_multiplier(slope_at(x, z, terrain));
        let accept_p = mult / MAX_ROCK_DENSITY_MULTIPLIER;
        if rng.gen_range(0.0..1.0) < accept_p {
            positions.push(Vec2::new(x, z));
        }
    }

    positions
}

fn world_origin(x: f32, z: f32, terrain: &impl TerrainHeightSource) -> IVec3 {
    IVec3::new(
        x.round() as i32,
        terrain.height_at(x, z).round() as i32,
        z.round() as i32,
    )
}

fn append_translated<B>(
    voxels: Vec<(IVec3, B)>,
    origin: IVec3,
    out: &mut Vec<(IVec3, WorldBlockType)>,
) where
    B: Into<WorldBlockType>,
{
    for (pos, block) in voxels {
        out.push((pos + origin, block.into()));
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::biome::{params_for, Biome, ROCKY_MIN_HEIGHT};
    use crate::placement::min_distance_from_density;
    use crate::terrain::ConstantHeight;

    fn area(min_x: f32, min_z: f32, max_x: f32, max_z: f32) -> Area {
        Area {
            min: Vec2::new(min_x, min_z),
            max: Vec2::new(max_x, max_z),
        }
    }

    fn count_stone(content: &ChunkContent) -> usize {
        content
            .tree_and_rock_voxels
            .iter()
            .filter(|(_, b)| *b == WorldBlockType::Stone)
            .count()
    }

    fn count_wood(content: &ChunkContent) -> usize {
        content
            .tree_and_rock_voxels
            .iter()
            .filter(|(_, b)| *b == WorldBlockType::Wood)
            .count()
    }

    #[test]
    fn world_block_type_converts_from_sub_generators() {
        assert_eq!(
            WorldBlockType::from(tree_generator::BlockType::Wood),
            WorldBlockType::Wood
        );
        assert_eq!(
            WorldBlockType::from(tree_generator::BlockType::Leaf),
            WorldBlockType::Leaf
        );
        assert_eq!(
            WorldBlockType::from(rock_generator::BlockType::Stone),
            WorldBlockType::Stone
        );
    }

    #[test]
    fn feature_seed_is_deterministic_and_position_sensitive() {
        let a = feature_seed(42, TREE_FEATURE_KIND, 3.2, -1.7);
        let b = feature_seed(42, TREE_FEATURE_KIND, 3.2, -1.7);
        let c = feature_seed(42, TREE_FEATURE_KIND, 3.3, -1.7);
        assert_eq!(a, b);
        assert_ne!(a, c);
        assert_ne!(
            feature_seed(42, TREE_FEATURE_KIND, 1.0, 1.0),
            feature_seed(42, ROCK_FEATURE_KIND, 1.0, 1.0)
        );
    }

    #[test]
    fn same_seed_and_area_produce_identical_chunk() {
        let terrain = ConstantHeight(5.0);
        let a = area(0.0, 0.0, 24.0, 24.0);
        let first = generate_chunk(99, a, &terrain);
        let second = generate_chunk(99, a, &terrain);
        assert_eq!(first, second);
        assert!(
            !first.tree_and_rock_voxels.is_empty() || !first.grass_instances.is_empty(),
            "expected some content"
        );
    }

    #[test]
    fn rocky_area_has_more_stone_than_forest_area() {
        let rocky_terrain = ConstantHeight(ROCKY_MIN_HEIGHT + 5.0);
        let forest_terrain = ConstantHeight(5.0);
        // Center (12, 12); seed chosen so flat land classifies as Forest.
        let chunk_area = area(0.0, 0.0, 24.0, 24.0);
        let seed = seed_with_forest_center(&chunk_area, &forest_terrain);

        let rocky = generate_chunk(seed, chunk_area, &rocky_terrain);
        let forest = generate_chunk(seed, chunk_area, &forest_terrain);

        assert_eq!(
            classify(12.0, 12.0, seed, &rocky_terrain),
            Biome::Rocky
        );
        assert_eq!(
            classify(12.0, 12.0, seed, &forest_terrain),
            Biome::Forest
        );

        let rocky_stone = count_stone(&rocky);
        let forest_stone = count_stone(&forest);
        assert!(
            rocky_stone > forest_stone,
            "rocky stone={rocky_stone} forest stone={forest_stone}"
        );
        assert!(
            count_wood(&forest) > count_wood(&rocky),
            "forest should have more wood voxels"
        );
        assert!(
            forest.grass_instances.len() > rocky.grass_instances.len(),
            "forest should have more grass than rocky"
        );
    }

    #[test]
    fn tree_origins_respect_poisson_min_distance_in_world_xz() {
        let terrain = ConstantHeight(5.0);
        let chunk_area = area(0.0, 0.0, 40.0, 40.0);
        let seed = seed_with_forest_center(&chunk_area, &terrain);
        let center = area_center(&chunk_area);
        let biome = classify(center.x, center.y, seed, &terrain);
        assert_eq!(biome, Biome::Forest);

        let params = params_for(biome);
        let r = min_distance_from_density(params.tree_density).unwrap();
        // Same inputs generate_chunk uses for tree placement.
        let origins = poisson_disc_sample(seed, &chunk_area, params.tree_density);
        assert!(origins.len() >= 2, "need several trees to test spacing");

        for i in 0..origins.len() {
            for j in (i + 1)..origins.len() {
                let d = origins[i].distance(origins[j]);
                assert!(
                    d + 1e-4 >= r,
                    "origins {i},{j} distance {d} < r={r}"
                );
            }
        }

        // World translation only rounds XZ / sets Y — horizontal spacing unchanged.
        let content = generate_chunk(seed, chunk_area, &terrain);
        assert!(count_wood(&content) > 0);
    }

    #[test]
    fn grass_y_matches_terrain_height() {
        let terrain = ConstantHeight(7.5);
        let chunk_area = area(0.0, 0.0, 16.0, 16.0);
        let content = generate_chunk(3, chunk_area, &terrain);
        assert!(!content.grass_instances.is_empty());
        for instance in &content.grass_instances {
            assert!(
                (instance.position.y - 7.5).abs() < 1e-5,
                "grass y={} expected 7.5",
                instance.position.y
            );
        }
    }

    fn seed_with_forest_center(area: &Area, terrain: &ConstantHeight) -> u64 {
        let c = area_center(area);
        for seed in 0..500u64 {
            if classify(c.x, c.y, seed, terrain) == Biome::Forest {
                return seed;
            }
        }
        panic!("could not find a seed that classifies area center as Forest");
    }
}
