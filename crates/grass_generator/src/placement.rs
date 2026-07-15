use glam::{Vec2, Vec3};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Axis-aligned placement region on the XZ plane.
///
/// **Vec2 convention (differs from `tree_generator`):** horizontal bounds use
/// `glam::Vec2` with `x` = world X and `y` = world Z. There is no `Vec2::z`;
/// the second component is world Z, not height. Callers such as `world_generator`
/// must map chunk or biome bounds into this `(x, z)` pair explicitly.
///
/// `min` and `max` are inclusive lower bounds and exclusive upper bounds for
/// sampling (see [`place_positions`]).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Area {
    /// Lower corner: `(world_x, world_z)`.
    pub min: Vec2,
    /// Upper corner: `(world_x, world_z)` — exclusive for random sampling.
    pub max: Vec2,
}

impl Area {
    pub fn width(&self) -> f32 {
        (self.max.x - self.min.x).max(0.0)
    }

    pub fn depth(&self) -> f32 {
        (self.max.y - self.min.y).max(0.0)
    }

    /// Horizontal footprint: width × depth.
    pub fn size(&self) -> f32 {
        self.width() * self.depth()
    }
}

/// Number of grass positions to sample for the given area and density.
///
/// `density` is instances per square unit of area. Non-positive density or
/// zero-sized area yields zero instances.
pub fn instance_count(area: &Area, density: f32) -> usize {
    if density <= 0.0 || area.size() <= 0.0 {
        return 0;
    }

    (area.size() * density).round().max(0.0) as usize
}

/// Phase 1: density-based random placement on the XZ plane.
///
/// # Returns
///
/// A `Vec<Vec3>` of placement positions. Each entry uses the usual world-space
/// layout: `x` and `z` are sampled inside `area` (see [`Area`] for the `Vec2`
/// as `(world_x, world_z)` convention); `y` is always `0.0` for now. Terrain
/// height is applied later by the integration layer, not here.
pub fn place_positions(seed: u64, area: &Area, density: f32) -> Vec<Vec3> {
    let count = instance_count(area, density);
    if count == 0 {
        return Vec::new();
    }

    let mut rng = StdRng::seed_from_u64(seed);
    let mut positions = Vec::with_capacity(count);

    for _ in 0..count {
        let x = rng.gen_range(area.min.x..area.max.x);
        let z = rng.gen_range(area.min.y..area.max.y);
        positions.push(Vec3::new(x, 0.0, z));
    }

    positions
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_area() -> Area {
        Area {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(10.0, 10.0),
        }
    }

    #[test]
    fn same_seed_produces_identical_positions() {
        let area = sample_area();
        let first = place_positions(42, &area, 2.0);
        let second = place_positions(42, &area, 2.0);
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_produce_different_positions() {
        let area = sample_area();
        let first = place_positions(1, &area, 2.0);
        let second = place_positions(2, &area, 2.0);
        assert_ne!(first, second);
    }

    #[test]
    fn zero_density_yields_empty_output() {
        let area = sample_area();
        assert!(place_positions(42, &area, 0.0).is_empty());
        assert!(place_positions(42, &area, -1.0).is_empty());
    }

    #[test]
    fn all_positions_lie_within_area_bounds() {
        let area = Area {
            min: Vec2::new(-5.0, 2.5),
            max: Vec2::new(15.0, 12.5),
        };
        let positions = place_positions(99, &area, 5.0);

        assert!(!positions.is_empty());
        for position in positions {
            assert!(position.x >= area.min.x && position.x < area.max.x);
            assert_eq!(position.y, 0.0);
            assert!(position.z >= area.min.y && position.z < area.max.y);
        }
    }

    #[test]
    fn instance_count_matches_area_times_density() {
        let area = Area {
            min: Vec2::ZERO,
            max: Vec2::new(10.0, 10.0),
        };
        // Hand calculation: 10 × 10 = 100 area units, × density 2.0 = 200.0 → round → 200
        let expected = 200;
        assert_eq!(instance_count(&area, 2.0), expected);
        assert_eq!(place_positions(0, &area, 2.0).len(), expected);
    }

    #[test]
    fn instance_count_is_non_negative_for_small_area() {
        let tiny = Area {
            min: Vec2::ZERO,
            max: Vec2::new(0.1, 0.1),
        };
        assert_eq!(instance_count(&tiny, 0.5), 0);
        assert_eq!(instance_count(&tiny, 100.0), 1);
    }
}
