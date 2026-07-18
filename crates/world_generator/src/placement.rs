//! Placement helpers for Phase 3: Poisson-disc trees and slope-based rocks.
//!
//! Grass placement stays in `grass_generator` (`place_positions`).

use glam::Vec2;
use grass_generator::Area;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::terrain::TerrainHeightSource;

/// Half-step for central-difference slope samples (world units).
pub const SLOPE_EPSILON: f32 = 0.5;

/// How strongly slope increases the rock-density multiplier.
pub const SLOPE_WEIGHT: f32 = 0.5;

/// Upper cap for [`rock_density_multiplier`] (flat terrain stays at 1.0).
pub const MAX_ROCK_DENSITY_MULTIPLIER: f32 = 3.0;

/// Bridson candidate attempts per active sample.
const POISSON_K: usize = 30;

/// Convert points-per-area density into a Poisson-disc minimum distance.
///
/// ```text
/// r = 1 / sqrt(density)
/// ```
///
/// See crate README (Phase 3). Non-positive density yields `None`.
pub fn min_distance_from_density(density: f32) -> Option<f32> {
    if density <= 0.0 {
        None
    } else {
        Some(1.0 / density.sqrt())
    }
}

/// Bridson Poisson-disc sampling on the XZ plane.
///
/// - `density`: points per square unit → minimum distance via
///   [`min_distance_from_density`]
/// - Returned `Vec2` uses the same `(world_x, world_z)` convention as [`Area`]
///
/// Empty when density or area size is non-positive.
pub fn poisson_disc_sample(seed: u64, area: &Area, density: f32) -> Vec<Vec2> {
    let Some(min_dist) = min_distance_from_density(density) else {
        return Vec::new();
    };
    if area.size() <= 0.0 {
        return Vec::new();
    }

    poisson_disc_bridson(seed, area, min_dist)
}

fn poisson_disc_bridson(seed: u64, area: &Area, min_dist: f32) -> Vec<Vec2> {
    let mut rng = StdRng::seed_from_u64(seed);
    let cell_size = min_dist / std::f32::consts::SQRT_2;
    let grid_w = ((area.width() / cell_size).ceil() as i32).max(1);
    let grid_d = ((area.depth() / cell_size).ceil() as i32).max(1);

    let mut grid: Vec<Option<usize>> = vec![None; (grid_w * grid_d) as usize];
    let mut samples: Vec<Vec2> = Vec::new();
    let mut active: Vec<usize> = Vec::new();

    let start = Vec2::new(
        rng.gen_range(area.min.x..area.max.x),
        rng.gen_range(area.min.y..area.max.y),
    );
    insert_sample(start, area, cell_size, grid_w, &mut grid, &mut samples, &mut active);

    let min_dist_sq = min_dist * min_dist;

    while !active.is_empty() {
        let active_idx = rng.gen_range(0..active.len());
        let sample_idx = active[active_idx];
        let origin = samples[sample_idx];
        let mut found = false;

        for _ in 0..POISSON_K {
            let candidate = random_annulus_point(&mut rng, origin, min_dist);
            if !in_area(candidate, area) {
                continue;
            }
            if is_far_enough(
                candidate,
                area,
                cell_size,
                grid_w,
                grid_d,
                &grid,
                &samples,
                min_dist_sq,
            ) {
                insert_sample(
                    candidate,
                    area,
                    cell_size,
                    grid_w,
                    &mut grid,
                    &mut samples,
                    &mut active,
                );
                found = true;
                break;
            }
        }

        if !found {
            active.swap_remove(active_idx);
        }
    }

    samples
}

fn random_annulus_point(rng: &mut StdRng, origin: Vec2, min_dist: f32) -> Vec2 {
    let radius = rng.gen_range(min_dist..(2.0 * min_dist));
    let angle = rng.gen_range(0.0..(std::f32::consts::TAU));
    origin + Vec2::new(angle.cos(), angle.sin()) * radius
}

fn in_area(point: Vec2, area: &Area) -> bool {
    point.x >= area.min.x
        && point.x < area.max.x
        && point.y >= area.min.y
        && point.y < area.max.y
}

fn insert_sample(
    point: Vec2,
    area: &Area,
    cell_size: f32,
    grid_w: i32,
    grid: &mut [Option<usize>],
    samples: &mut Vec<Vec2>,
    active: &mut Vec<usize>,
) {
    let index = samples.len();
    samples.push(point);
    active.push(index);
    let (cx, cz) = cell_coords(point, area, cell_size);
    if let Some(slot) = grid.get_mut((cz * grid_w + cx) as usize) {
        *slot = Some(index);
    }
}

fn cell_coords(point: Vec2, area: &Area, cell_size: f32) -> (i32, i32) {
    let cx = ((point.x - area.min.x) / cell_size).floor() as i32;
    let cz = ((point.y - area.min.y) / cell_size).floor() as i32;
    (cx, cz)
}

fn is_far_enough(
    point: Vec2,
    area: &Area,
    cell_size: f32,
    grid_w: i32,
    grid_d: i32,
    grid: &[Option<usize>],
    samples: &[Vec2],
    min_dist_sq: f32,
) -> bool {
    let (cx, cz) = cell_coords(point, area, cell_size);
    for dz in -2..=2 {
        for dx in -2..=2 {
            let nx = cx + dx;
            let nz = cz + dz;
            if nx < 0 || nz < 0 || nx >= grid_w || nz >= grid_d {
                continue;
            }
            if let Some(Some(other_idx)) = grid.get((nz * grid_w + nx) as usize) {
                if samples[*other_idx].distance_squared(point) < min_dist_sq {
                    return false;
                }
            }
        }
    }
    true
}

/// Magnitude of the terrain height gradient at `(x, z)` via central differences.
///
/// For a plane `h = a·x + b·z + c` this equals `sqrt(a² + b²)`.
pub fn slope_at(x: f32, z: f32, terrain: &impl TerrainHeightSource) -> f32 {
    let e = SLOPE_EPSILON;
    let dh_dx = (terrain.height_at(x + e, z) - terrain.height_at(x - e, z)) / (2.0 * e);
    let dh_dz = (terrain.height_at(x, z + e) - terrain.height_at(x, z - e)) / (2.0 * e);
    (dh_dx * dh_dx + dh_dz * dh_dz).sqrt()
}

/// Map a slope magnitude to a rock-density multiplier.
///
/// Flat terrain (`slope == 0`) → `1.0`. Steeper slopes raise the multiplier
/// linearly with [`SLOPE_WEIGHT`], capped at [`MAX_ROCK_DENSITY_MULTIPLIER`].
pub fn rock_density_multiplier(slope: f32) -> f32 {
    let slope = slope.max(0.0);
    (1.0 + SLOPE_WEIGHT * slope).min(MAX_ROCK_DENSITY_MULTIPLIER)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::terrain::{ConstantHeight, TerrainHeightSource};

    fn unit_area() -> Area {
        Area {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(40.0, 40.0),
        }
    }

    #[test]
    fn min_distance_matches_inverse_sqrt_density() {
        let r = min_distance_from_density(0.08).unwrap();
        assert!((r - (1.0 / 0.08_f32.sqrt())).abs() < 1e-5);
        assert!(min_distance_from_density(0.0).is_none());
        assert!(min_distance_from_density(-1.0).is_none());
    }

    #[test]
    fn poisson_points_respect_minimum_distance() {
        let density = 0.05;
        let r = min_distance_from_density(density).unwrap();
        let points = poisson_disc_sample(42, &unit_area(), density);
        assert!(!points.is_empty());

        for i in 0..points.len() {
            for j in (i + 1)..points.len() {
                let d = points[i].distance(points[j]);
                assert!(
                    d + 1e-4 >= r,
                    "points {i} and {j} are {d} apart, need >= {r}"
                );
            }
        }
    }

    #[test]
    fn poisson_same_seed_is_deterministic() {
        let area = unit_area();
        let a = poisson_disc_sample(7, &area, 0.04);
        let b = poisson_disc_sample(7, &area, 0.04);
        assert_eq!(a, b);
    }

    #[test]
    fn higher_density_yields_more_points() {
        let area = unit_area();
        let sparse = poisson_disc_sample(11, &area, 0.02);
        let dense = poisson_disc_sample(11, &area, 0.08);
        assert!(
            dense.len() > sparse.len(),
            "dense={} sparse={}",
            dense.len(),
            sparse.len()
        );
    }

    #[test]
    fn poisson_non_positive_density_is_empty() {
        assert!(poisson_disc_sample(1, &unit_area(), 0.0).is_empty());
    }

    /// Plane `h = 2x` → analytical gradient magnitude 2.
    struct RampTerrain {
        gradient_x: f32,
    }

    impl TerrainHeightSource for RampTerrain {
        fn height_at(&self, x: f32, _z: f32) -> f32 {
            self.gradient_x * x
        }
    }

    #[test]
    fn slope_on_ramp_matches_hand_calculation() {
        let terrain = RampTerrain { gradient_x: 2.0 };
        let slope = slope_at(5.0, 3.0, &terrain);
        assert!(
            (slope - 2.0).abs() < 1e-4,
            "expected slope 2.0, got {slope}"
        );
    }

    #[test]
    fn flat_terrain_has_zero_slope_and_unit_multiplier() {
        let terrain = ConstantHeight(4.0);
        let slope = slope_at(1.0, 2.0, &terrain);
        assert_eq!(slope, 0.0);
        assert_eq!(rock_density_multiplier(slope), 1.0);
    }

    #[test]
    fn steep_slope_multiplier_is_capped() {
        let huge = rock_density_multiplier(100.0);
        assert_eq!(huge, MAX_ROCK_DENSITY_MULTIPLIER);
        assert!(rock_density_multiplier(2.0) > 1.0);
        assert!(rock_density_multiplier(2.0) < MAX_ROCK_DENSITY_MULTIPLIER);
    }
}
