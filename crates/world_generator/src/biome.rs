//! Biome classification and parameter mapping (Phases 1–2).

use grass_generator::GrassParams;
use rock_generator::RockParams;
use tree_generator::TreeParams;

use crate::noise::value_noise_2d;
use crate::terrain::TerrainHeightSource;

/// World zone derived from height and moisture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Biome {
    Forest,
    Rocky,
    Clearing,
}

/// Bundled generator parameters and densities for one biome (Phase 2).
///
/// Densities are targets for later placement (Phase 3); this phase only
/// defines the static mapping via [`params_for`].
#[derive(Debug, Clone, PartialEq)]
pub struct BiomeParams {
    pub tree_params: TreeParams,
    /// Target trees per square unit of horizontal area (Poisson input later).
    pub tree_density: f32,
    pub grass_params: GrassParams,
    pub rock_params: RockParams,
    /// Target rocks per square unit of horizontal area (placement later).
    pub rock_density: f32,
}

/// Static biome → parameter mapping. No randomness in this phase.
pub fn params_for(biome: Biome) -> BiomeParams {
    match biome {
        Biome::Forest => BiomeParams {
            // 3D branching → fuller canopy for a dense forest look.
            tree_params: TreeParams::generic_3d(),
            tree_density: 0.08,
            grass_params: GrassParams {
                density: 1.2,
                ..GrassParams::default()
            },
            // Larger / denser than rock_generator defaults so half_extent=4
            // rarely collapses to empty after the connectivity filter.
            rock_params: RockParams {
                half_extent: 4,
                threshold: 0.4,
                radial_falloff: 0.35,
                ..RockParams::default()
            },
            rock_density: 0.015,
        },
        Biome::Rocky => BiomeParams {
            // Sparse, shallow 2D scrub — trees are rare on exposed rock.
            tree_params: TreeParams {
                depth: 2,
                ..TreeParams::generic_2d()
            },
            tree_density: 0.005,
            grass_params: GrassParams {
                density: 0.35,
                ..GrassParams::default()
            },
            rock_params: RockParams {
                half_extent: 6,
                ..RockParams::default()
            },
            rock_density: 0.09,
        },
        Biome::Clearing => BiomeParams {
            // Occasional mid-sized 2D trees at meadow edges.
            tree_params: TreeParams {
                depth: 3,
                ..TreeParams::generic_2d()
            },
            tree_density: 0.012,
            grass_params: GrassParams {
                density: 2.5,
                ..GrassParams::default()
            },
            rock_params: RockParams {
                half_extent: 4,
                ..RockParams::default()
            },
            rock_density: 0.025,
        },
    }
}

/// World Y at or above this value is always [`Biome::Rocky`].
pub const ROCKY_MIN_HEIGHT: f32 = 10.0;

/// Below [`ROCKY_MIN_HEIGHT`], moisture under this value yields [`Biome::Clearing`].
pub const CLEARING_MAX_MOISTURE: f32 = 0.4;

/// Spatial frequency of the moisture noise layer.
pub const MOISTURE_FREQUENCY: f32 = 0.05;

/// Classify the biome at surface position `(x, z)`.
///
/// Rules (Phase 1):
/// 1. `height >= ROCKY_MIN_HEIGHT` → [`Biome::Rocky`]
/// 2. else if `moisture < CLEARING_MAX_MOISTURE` → [`Biome::Clearing`]
/// 3. else → [`Biome::Forest`]
pub fn classify(x: f32, z: f32, seed: u64, terrain: &impl TerrainHeightSource) -> Biome {
    let height = terrain.height_at(x, z);
    if height >= ROCKY_MIN_HEIGHT {
        return Biome::Rocky;
    }

    let moisture = moisture_at(x, z, seed);
    if moisture < CLEARING_MAX_MOISTURE {
        Biome::Clearing
    } else {
        Biome::Forest
    }
}

/// Moisture in `[0, 1]` at `(x, z)`, independent of terrain height.
pub fn moisture_at(x: f32, z: f32, seed: u64) -> f32 {
    value_noise_2d(
        x * MOISTURE_FREQUENCY,
        z * MOISTURE_FREQUENCY,
        moisture_seed(seed),
    )
}

/// Derive a dedicated seed for the moisture layer so it is not identical
/// to a height-noise field that might use the same world seed later.
fn moisture_seed(world_seed: u64) -> u64 {
    world_seed
        .wrapping_mul(0xD6E8_FEB8_6659_FD93)
        .wrapping_add(0xA076_1D64_78BD_642F)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::ConstantHeight;
    use std::collections::HashSet;

    // --- Phase 2: params_for ---

    #[test]
    fn params_for_covers_all_three_biomes() {
        let _ = params_for(Biome::Forest);
        let _ = params_for(Biome::Rocky);
        let _ = params_for(Biome::Clearing);
    }

    #[test]
    fn rocky_has_higher_rock_density_than_forest_and_clearing() {
        let forest = params_for(Biome::Forest);
        let rocky = params_for(Biome::Rocky);
        let clearing = params_for(Biome::Clearing);

        assert!(rocky.rock_density > forest.rock_density);
        assert!(rocky.rock_density > clearing.rock_density);
    }

    #[test]
    fn forest_has_highest_tree_density() {
        let forest = params_for(Biome::Forest);
        let rocky = params_for(Biome::Rocky);
        let clearing = params_for(Biome::Clearing);

        assert!(forest.tree_density > rocky.tree_density);
        assert!(forest.tree_density > clearing.tree_density);
        assert!(rocky.tree_density < clearing.tree_density);
    }

    #[test]
    fn clearing_has_highest_grass_density() {
        let forest = params_for(Biome::Forest);
        let rocky = params_for(Biome::Rocky);
        let clearing = params_for(Biome::Clearing);

        assert!(clearing.grass_params.density > forest.grass_params.density);
        assert!(clearing.grass_params.density > rocky.grass_params.density);
        assert!(forest.grass_params.density > rocky.grass_params.density);
    }

    #[test]
    fn forest_uses_fuller_3d_tree_preset() {
        let forest = params_for(Biome::Forest);
        let reference = TreeParams::generic_3d();
        assert_eq!(forest.tree_params.depth, reference.depth);
        assert_eq!(
            forest.tree_params.turtle.angle_degrees,
            reference.turtle.angle_degrees
        );
    }

    #[test]
    fn rocky_trees_are_shallower_than_forest() {
        let forest = params_for(Biome::Forest);
        let rocky = params_for(Biome::Rocky);
        assert!(rocky.tree_params.depth < forest.tree_params.depth);
    }

    #[test]
    fn rocky_rocks_are_larger_than_forest() {
        let forest = params_for(Biome::Forest);
        let rocky = params_for(Biome::Rocky);
        assert!(rocky.rock_params.half_extent > forest.rock_params.half_extent);
    }

    #[test]
    fn params_for_is_deterministic() {
        assert_eq!(params_for(Biome::Forest), params_for(Biome::Forest));
        assert_eq!(params_for(Biome::Rocky), params_for(Biome::Rocky));
        assert_eq!(params_for(Biome::Clearing), params_for(Biome::Clearing));
    }

    // --- Phase 1: classify ---

    #[test]
    fn same_position_and_seed_is_deterministic() {
        let terrain = ConstantHeight(5.0);
        let a = classify(12.0, -7.0, 42, &terrain);
        let b = classify(12.0, -7.0, 42, &terrain);
        assert_eq!(a, b);
    }

    #[test]
    fn high_height_classifies_as_rocky() {
        let terrain = ConstantHeight(ROCKY_MIN_HEIGHT + 1.0);
        // Moisture varies across the plane; height rule must still win.
        for i in 0..20 {
            let x = i as f32 * 17.0;
            let z = i as f32 * -9.0;
            assert_eq!(
                classify(x, z, 99, &terrain),
                Biome::Rocky,
                "expected Rocky at ({x}, {z})"
            );
        }
    }

    #[test]
    fn low_moisture_on_flat_land_classifies_as_clearing() {
        let terrain = ConstantHeight(5.0);
        let seed = 7u64;
        let (x, z) = find_position(seed, |m| m < CLEARING_MAX_MOISTURE)
            .expect("should find a dry moisture sample");
        assert_eq!(classify(x, z, seed, &terrain), Biome::Clearing);
        assert!(moisture_at(x, z, seed) < CLEARING_MAX_MOISTURE);
    }

    #[test]
    fn high_moisture_on_flat_land_classifies_as_forest() {
        let terrain = ConstantHeight(5.0);
        let seed = 7u64;
        let (x, z) = find_position(seed, |m| m >= CLEARING_MAX_MOISTURE)
            .expect("should find a wet moisture sample");
        assert_eq!(classify(x, z, seed, &terrain), Biome::Forest);
        assert!(moisture_at(x, z, seed) >= CLEARING_MAX_MOISTURE);
    }

    #[test]
    fn all_three_biomes_occur_across_positions_and_seeds() {
        let flat = ConstantHeight(5.0);
        let high = ConstantHeight(20.0);
        let mut seen = HashSet::new();

        for seed in [1u64, 2, 3, 11, 42] {
            seen.insert(classify(0.0, 0.0, seed, &high));
            for ix in 0..40 {
                for iz in 0..40 {
                    let x = ix as f32 * 13.0;
                    let z = iz as f32 * 17.0;
                    seen.insert(classify(x, z, seed, &flat));
                }
            }
        }

        assert!(seen.contains(&Biome::Rocky), "missing Rocky: {seen:?}");
        assert!(seen.contains(&Biome::Clearing), "missing Clearing: {seen:?}");
        assert!(seen.contains(&Biome::Forest), "missing Forest: {seen:?}");
    }

    fn find_position(seed: u64, predicate: impl Fn(f32) -> bool) -> Option<(f32, f32)> {
        for ix in 0..80 {
            for iz in 0..80 {
                let x = ix as f32 * 11.0;
                let z = iz as f32 * 13.0;
                if predicate(moisture_at(x, z, seed)) {
                    return Some((x, z));
                }
            }
        }
        None
    }
}
