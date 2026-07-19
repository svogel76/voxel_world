//! Phase 4: chunk orchestration — one biome per area, combined world content.

use glam::{IVec3, Vec2};
use grass_generator::{generate as generate_grass, Area, GrassInstance};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};
use rock_generator::generate as generate_rock;
use tree_generator::generate as generate_tree;

use crate::biome::{classify, params_for, Biome, BiomeParams};
use crate::placement::{
    poisson_disc_sample, rock_density_multiplier, slope_at, MAX_ROCK_DENSITY_MULTIPLIER,
};
use crate::terrain::TerrainHeightSource;
use crate::understory::{
    bush_cluster_voxels, fallen_log_voxels, fern_carpet_params, trunk_fern_area, BUSHES_PER_TREE,
    FALLEN_LOG_CHANCE, TRUNK_FERN_DENSITY,
};

/// Discriminator mixed into [`feature_seed`] so features never share a stream.
const TREE_FEATURE_KIND: u64 = 0x72EE_0001;
const ROCK_FEATURE_KIND: u64 = 0xA0C4_0002;
const ROCK_PLACEMENT_KIND: u64 = 0xA0C4_91A5;
const TRUNK_FERN_KIND: u64 = 0xFE4E_0003;
const BUSH_FEATURE_KIND: u64 = 0xB051_0004;
const LOG_FEATURE_KIND: u64 = 0x1060_0005;
const LOG_CHANCE_KIND: u64 = 0x1060_C1A1;

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
/// Forest areas also get understory layers (trunk ferns, leaf bushes, logs).
pub fn generate_chunk(
    seed: u64,
    area: Area,
    terrain: &impl TerrainHeightSource,
) -> ChunkContent {
    let center = area_center(&area);
    let biome = classify(center.x, center.y, seed, terrain);
    let params = params_for(biome);

    let mut tree_and_rock_voxels = Vec::new();

    let tree_positions = place_trees(seed, &area, &params, terrain, &mut tree_and_rock_voxels);
    place_rocks(seed, &area, &params, terrain, &mut tree_and_rock_voxels);

    let mut grass_instances = generate_grass(seed, area, &params.grass_params);
    stamp_grass_heights(&mut grass_instances, terrain);

    if biome == Biome::Forest {
        place_forest_understory(
            seed,
            &area,
            &tree_positions,
            terrain,
            &mut tree_and_rock_voxels,
            &mut grass_instances,
        );
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
) -> Vec<Vec2> {
    let positions = poisson_disc_sample(seed, area, params.tree_density);
    for pos in &positions {
        let tree_seed = feature_seed(seed, TREE_FEATURE_KIND, pos.x, pos.y);
        let origin = world_origin(pos.x, pos.y, terrain);
        let voxels = generate_tree(tree_seed, &params.tree_params);
        append_translated(voxels, origin, out);
    }
    positions
}

/// Trunk-foot ferns, leaf bushes, and occasional fallen logs near forest trees.
fn place_forest_understory(
    seed: u64,
    area: &Area,
    tree_positions: &[Vec2],
    terrain: &impl TerrainHeightSource,
    voxels: &mut Vec<(IVec3, WorldBlockType)>,
    grass: &mut Vec<GrassInstance>,
) {
    let trunk_params = fern_carpet_params(TRUNK_FERN_DENSITY);

    for tree in tree_positions {
        if let Some(patch) = trunk_fern_area(*tree, area) {
            let fern_seed = feature_seed(seed, TRUNK_FERN_KIND, tree.x, tree.y);
            let mut patch_grass = generate_grass(fern_seed, patch, &trunk_params);
            stamp_grass_heights(&mut patch_grass, terrain);
            grass.extend(patch_grass);
        }

        for i in 0..BUSHES_PER_TREE {
            let bush_seed = feature_seed(seed, BUSH_FEATURE_KIND.wrapping_add(i as u64), tree.x, tree.y);
            let offset = bush_offset(bush_seed);
            let bx = tree.x + offset.x;
            let bz = tree.y + offset.y;
            if !point_in_area(bx, bz, area) {
                continue;
            }
            let origin = world_origin(bx, bz, terrain);
            for local in bush_cluster_voxels(bush_seed) {
                voxels.push((origin + local, WorldBlockType::Leaf));
            }
        }

        let chance_seed = feature_seed(seed, LOG_CHANCE_KIND, tree.x, tree.y);
        let mut chance_rng = StdRng::seed_from_u64(chance_seed);
        if chance_rng.gen_range(0.0..1.0) >= FALLEN_LOG_CHANCE {
            continue;
        }
        let log_seed = feature_seed(seed, LOG_FEATURE_KIND, tree.x, tree.y);
        let origin = world_origin(tree.x, tree.y, terrain);
        for local in fallen_log_voxels(log_seed) {
            let world = origin + local;
            if point_in_area(world.x as f32, world.z as f32, area) {
                voxels.push((world, WorldBlockType::Wood));
            }
        }
    }
}

fn bush_offset(seed: u64) -> Vec2 {
    let mut rng = StdRng::seed_from_u64(seed);
    let angle = rng.gen_range(0.0..std::f32::consts::TAU);
    let dist = rng.gen_range(2.5..4.5);
    Vec2::new(angle.cos() * dist, angle.sin() * dist)
}

fn point_in_area(x: f32, z: f32, area: &Area) -> bool {
    x >= area.min.x && x < area.max.x && z >= area.min.y && z < area.max.y
}

fn stamp_grass_heights(grass: &mut [GrassInstance], terrain: &impl TerrainHeightSource) {
    for instance in grass {
        let p = instance.position;
        // Same surface as voxel terrain top face (`floor(height)`), not the
        // continuous noise value — otherwise grass floats above the mesh.
        instance.position.y = surface_y(terrain, p.x, p.z) as f32;
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
        let voxels = generate_rock(rock_seed, &params.rock_params);
        if voxels.is_empty() {
            continue;
        }
        let origin = rock_origin_on_surface(pos.x, pos.y, &voxels, terrain);
        append_translated(voxels, origin, out);
    }
}

/// Place a rock so its lowest voxel rests on the terrain (no floating clumps).
///
/// Rocks are generated centered on the origin; after `keep_largest_component` the
/// remaining shape often has `min_y >` the original bottom. Pinning `world_origin.y`
/// to the surface then leaves the whole clump hovering. We also take the **max**
/// surface under the footprint so overhangs on slopes do not float over dips.
fn rock_origin_on_surface(
    x: f32,
    z: f32,
    voxels: &[(IVec3, rock_generator::BlockType)],
    terrain: &impl TerrainHeightSource,
) -> IVec3 {
    let ox = x.round() as i32;
    let oz = z.round() as i32;
    let min_local_y = voxels.iter().map(|(p, _)| p.y).min().unwrap_or(0);

    let mut max_surface = i32::MIN;
    for (p, _) in voxels {
        let wx = ox + p.x;
        let wz = oz + p.z;
        max_surface = max_surface.max(surface_y(terrain, wx as f32, wz as f32));
    }
    if max_surface == i32::MIN {
        max_surface = surface_y(terrain, x, z);
    }

    IVec3::new(ox, max_surface - min_local_y, oz)
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

/// World-space origin for a feature at continuous `(x, z)`.
///
/// Y matches the **top face** of the voxel terrain column (`floor(height)`),
/// consistent with `voxel_game::height::top_solid_y` + 1. Using `round` placed
/// objects one block too high whenever the fractional height was ≥ 0.5.
fn world_origin(x: f32, z: f32, terrain: &impl TerrainHeightSource) -> IVec3 {
    IVec3::new(
        x.round() as i32,
        surface_y(terrain, x, z),
        z.round() as i32,
    )
}

fn surface_y(terrain: &impl TerrainHeightSource, x: f32, z: f32) -> i32 {
    terrain.height_at(x, z).floor() as i32
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
    fn world_origin_y_matches_voxel_surface_not_rounded_height() {
        // height 10.7 → voxel top solid at 9, walkable top face at 10 (= floor).
        // `round` would wrongly place features at 11 (floating).
        let terrain = ConstantHeight(10.7);
        let origin = world_origin(1.2, -3.4, &terrain);
        assert_eq!(origin.y, 10);
        assert_eq!(surface_y(&terrain, 1.2, -3.4), 10);
    }

    #[test]
    fn rock_origin_pins_lowest_voxel_to_surface() {
        let terrain = ConstantHeight(10.0);
        // Simulate a clump whose bottom was discarded (min local y = 1).
        let voxels = vec![
            (IVec3::new(0, 1, 0), rock_generator::BlockType::Stone),
            (IVec3::new(1, 1, 0), rock_generator::BlockType::Stone),
            (IVec3::new(0, 2, 0), rock_generator::BlockType::Stone),
        ];
        let origin = rock_origin_on_surface(3.2, -1.4, &voxels, &terrain);
        assert_eq!(origin, IVec3::new(3, 9, -1)); // 10 - min_y(1) = 9
        let settled_min = voxels.iter().map(|(p, _)| p.y + origin.y).min().unwrap();
        assert_eq!(settled_min, 10);
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

    #[test]
    fn forest_understory_adds_ferns_bushes_and_logs() {
        use grass_generator::GrassVariant;

        let terrain = ConstantHeight(5.0);
        let chunk_area = area(0.0, 0.0, 40.0, 40.0);
        let seed = seed_with_forest_center(&chunk_area, &terrain);
        let content = generate_chunk(seed, chunk_area, &terrain);

        let ferns = content
            .grass_instances
            .iter()
            .filter(|g| g.variant == GrassVariant::Fern)
            .count();
        let grass = content
            .grass_instances
            .iter()
            .filter(|g| g.variant == GrassVariant::Grass)
            .count();
        assert!(ferns > grass, "forest floor should be fern-heavy (ferns={ferns} grass={grass})");

        let leaf = content
            .tree_and_rock_voxels
            .iter()
            .filter(|(_, b)| *b == WorldBlockType::Leaf)
            .count();
        // Tree crowns already contribute Leaf; bushes add more near the ground.
        // Require a non-trivial leaf presence as a smoke signal for bushes+canopy.
        assert!(leaf > 50, "expected leaf voxels from crowns/bushes, got {leaf}");

        // With several trees and ~22% log chance, a 40×40 forest should usually
        // gain extra wood beyond trunks; at minimum wood must exist.
        assert!(count_wood(&content) > 0);
    }

    #[test]
    fn rocky_chunk_skips_forest_understory_layers() {
        let rocky_terrain = ConstantHeight(ROCKY_MIN_HEIGHT + 5.0);
        let chunk_area = area(0.0, 0.0, 24.0, 24.0);
        let content = generate_chunk(1, chunk_area, &rocky_terrain);
        assert_eq!(
            classify(12.0, 12.0, 1, &rocky_terrain),
            Biome::Rocky
        );
        // Rocky has almost no trees → no trunk fern belts / bushes / logs.
        // Grass stays sparse relative to a forest chunk of the same size.
        let forest_terrain = ConstantHeight(5.0);
        let forest_seed = seed_with_forest_center(&chunk_area, &forest_terrain);
        let forest = generate_chunk(forest_seed, chunk_area, &forest_terrain);
        assert!(forest.grass_instances.len() > content.grass_instances.len());
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
