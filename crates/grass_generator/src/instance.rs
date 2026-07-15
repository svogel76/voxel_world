use std::f32::consts::TAU;

use glam::Vec3;
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

use crate::generate::{GrassParams, VariantWeights};

/// Default minimum uniform scale factor (see [`GrassParams::default`]).
pub const SCALE_MIN: f32 = 0.8;
/// Default maximum uniform scale factor (see [`GrassParams::default`]).
pub const SCALE_MAX: f32 = 1.2;

/// Visual kind of vegetation clump. Extend this enum when adding new species;
/// wire new weights into [`VariantWeights`] and [`pick_variant`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GrassVariant {
    Grass,
    Fern,
}

/// Placement data for one grass/fern cross-quad clump (geometry is built later in Bevy).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GrassInstance {
    pub position: Vec3,
    /// Y-axis rotation in **radians**, uniform in `[0, 2π)`.
    ///
    /// `tree_generator` exposes turtle angles in degrees for authoring; stored
    /// instance rotation uses radians to match `glam` / Bevy (`Quat::from_rotation_y`).
    pub rotation_y: f32,
    /// Uniform scale factor within `params.scale_range`.
    pub scale: f32,
    pub variant: GrassVariant,
}

/// Phase 2: attach random rotation, scale, and variant to each placement position.
///
/// Uses one [`StdRng::seed_from_u64`] for the entire call. Per-instance draws happen
/// in fixed order (rotation → scale → variant) so the output is reproducible.
pub fn build_instances(
    seed: u64,
    positions: &[Vec3],
    params: &GrassParams,
) -> Vec<GrassInstance> {
    if positions.is_empty() {
        return Vec::new();
    }

    let (scale_min, scale_max) = normalized_scale_range(params.scale_range);
    let mut rng = StdRng::seed_from_u64(seed);
    let mut instances = Vec::with_capacity(positions.len());

    for &position in positions {
        let rotation_y = rng.gen_range(0.0..TAU);
        let scale = rng.gen_range(scale_min..=scale_max);
        let variant = pick_variant(&params.variant_weights, &mut rng);

        instances.push(GrassInstance {
            position,
            rotation_y,
            scale,
            variant,
        });
    }

    instances
}

fn normalized_scale_range(range: (f32, f32)) -> (f32, f32) {
    (range.0.min(range.1), range.0.max(range.1))
}

fn pick_variant(weights: &VariantWeights, rng: &mut impl Rng) -> GrassVariant {
    let choices = [
        (GrassVariant::Grass, weights.grass),
        (GrassVariant::Fern, weights.fern),
    ];
    let positive: Vec<_> = choices.into_iter().filter(|(_, w)| *w > 0.0).collect();

    if positive.is_empty() {
        return GrassVariant::Grass;
    }

    let total: f32 = positive.iter().map(|(_, w)| w).sum();
    let mut roll = rng.gen_range(0.0..total);

    for (variant, weight) in &positive {
        roll -= weight;
        if roll <= 0.0 {
            return *variant;
        }
    }

    positive.last().expect("positive is non-empty").0
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::generate::GrassParams;

    fn sample_positions(count: usize) -> Vec<Vec3> {
        (0..count)
            .map(|i| Vec3::new(i as f32, 0.0, i as f32 * 0.5))
            .collect()
    }

    fn default_params() -> GrassParams {
        GrassParams::default()
    }

    #[test]
    fn same_seed_produces_identical_instances() {
        let positions = sample_positions(50);
        let params = default_params();
        let first = build_instances(42, &positions, &params);
        let second = build_instances(42, &positions, &params);
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_produce_different_instances() {
        let positions = sample_positions(50);
        let params = default_params();
        let first = build_instances(1, &positions, &params);
        let second = build_instances(2, &positions, &params);
        assert_ne!(first, second);
    }

    #[test]
    fn all_scale_values_lie_within_expected_range() {
        let positions = sample_positions(200);
        let params = default_params();
        let instances = build_instances(7, &positions, &params);

        for instance in instances {
            assert!(instance.scale >= SCALE_MIN);
            assert!(instance.scale <= SCALE_MAX);
        }
    }

    #[test]
    fn custom_scale_range_is_respected() {
        let positions = sample_positions(100);
        let params = GrassParams {
            scale_range: (0.5, 0.6),
            ..default_params()
        };
        let instances = build_instances(3, &positions, &params);

        for instance in instances {
            assert!(instance.scale >= 0.5);
            assert!(instance.scale <= 0.6);
        }
    }

    #[test]
    fn both_variants_appear_with_enough_instances() {
        let positions = sample_positions(200);
        let params = default_params();
        let instances = build_instances(99, &positions, &params);

        let has_grass = instances
            .iter()
            .any(|i| i.variant == GrassVariant::Grass);
        let has_fern = instances.iter().any(|i| i.variant == GrassVariant::Fern);

        assert!(has_grass, "expected at least one Grass variant");
        assert!(has_fern, "expected at least one Fern variant");
    }

    #[test]
    fn empty_positions_yields_empty_instances() {
        assert!(build_instances(42, &[], &default_params()).is_empty());
    }
}
