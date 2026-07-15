//! Seed-driven size and axis-scale variation (Phase 2).

use glam::IVec3;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::types::RockParams;

/// Derive anisotropic half-extents from `params.half_extent` and the seed.
///
/// Each axis is multiplied by an independent scale drawn uniformly from
/// `[axis_scale_min, axis_scale_max]`, then rounded to at least 1.
pub fn axis_extents(seed: u64, params: &RockParams) -> IVec3 {
    let mut rng = StdRng::seed_from_u64(seed);
    let base = params.half_extent.max(1) as f32;
    let min = params.axis_scale_min.min(params.axis_scale_max);
    let max = params.axis_scale_min.max(params.axis_scale_max);

    IVec3::new(
        scale_axis(&mut rng, base, min, max),
        scale_axis(&mut rng, base, min, max),
        scale_axis(&mut rng, base, min, max),
    )
}

fn scale_axis(rng: &mut StdRng, base: f32, min: f32, max: f32) -> i32 {
    let scale = if (min - max).abs() < f32::EPSILON {
        min
    } else {
        rng.gen_range(min..=max)
    };
    (base * scale).round().max(1.0) as i32
}

#[cfg(test)]
mod tests {
    use super::*;

    fn varied_params() -> RockParams {
        RockParams {
            half_extent: 5,
            threshold: 0.5,
            noise_frequency: 0.35,
            radial_falloff: 0.45,
            axis_scale_min: 0.7,
            axis_scale_max: 1.3,
        }
    }

    #[test]
    fn same_seed_yields_same_extents() {
        let params = varied_params();
        assert_eq!(axis_extents(42, &params), axis_extents(42, &params));
    }

    #[test]
    fn fixed_scale_keeps_cubic_extent() {
        let params = RockParams {
            axis_scale_min: 1.0,
            axis_scale_max: 1.0,
            ..varied_params()
        };
        let e = axis_extents(99, &params);
        assert_eq!(e, IVec3::splat(5));
    }

    #[test]
    fn different_seeds_can_produce_different_extents() {
        let params = varied_params();
        let extents: Vec<IVec3> = [1u64, 2, 3, 4, 5, 6, 7, 8]
            .into_iter()
            .map(|s| axis_extents(s, &params))
            .collect();
        let unique: std::collections::HashSet<_> = extents.iter().copied().collect();
        assert!(
            unique.len() > 1,
            "expected seed-driven axis variation, got {extents:?}"
        );
    }

    #[test]
    fn extents_are_at_least_one() {
        let params = RockParams {
            half_extent: 1,
            axis_scale_min: 0.5,
            axis_scale_max: 0.5,
            ..varied_params()
        };
        assert_eq!(axis_extents(1, &params), IVec3::splat(1));
    }
}
