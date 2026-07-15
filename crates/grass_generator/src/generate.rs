use crate::instance::{build_instances, GrassInstance};
use crate::placement::{place_positions, Area};

pub use crate::instance::{SCALE_MAX, SCALE_MIN};

/// Relative weights for [`GrassVariant`] selection. Weights need not sum to 1.0;
/// only values `> 0.0` participate, chosen proportionally (like
/// `tree_generator::ProductionRule`).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct VariantWeights {
    pub grass: f32,
    pub fern: f32,
}

impl Default for VariantWeights {
    fn default() -> Self {
        Self {
            grass: 1.0,
            fern: 1.0,
        }
    }
}

/// Full parameter set for [`generate`].
#[derive(Debug, Clone, PartialEq)]
pub struct GrassParams {
    /// Instances per square unit of horizontal area.
    pub density: f32,
    /// Uniform scale range `(min, max)` applied per instance (inclusive bounds).
    pub scale_range: (f32, f32),
    pub variant_weights: VariantWeights,
}

impl Default for GrassParams {
    fn default() -> Self {
        Self {
            density: 1.5,
            scale_range: (SCALE_MIN, SCALE_MAX),
            variant_weights: VariantWeights::default(),
        }
    }
}

/// Generate grass/fern placement instances for an axis-aligned area.
///
/// Pipeline: `place_positions(seed, …)` → `build_instances(seed, …)`.
///
/// **RNG:** The same `seed` is passed to both phases. Each phase still creates
/// its own `StdRng::seed_from_u64(seed)` (see README) so their random streams
/// stay independent; `generate` does not derive separate sub-seeds. That keeps
/// `generate(seed, …)` bit-identical to calling the two phase functions manually
/// with the same seed, and preserves “same seed + same params → same output”.
pub fn generate(seed: u64, area: Area, params: &GrassParams) -> Vec<GrassInstance> {
    let positions = place_positions(seed, &area, params.density);
    build_instances(seed, &positions, params)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::instance::build_instances;
    use crate::placement::place_positions;

    fn sample_area() -> Area {
        Area {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(10.0, 10.0),
        }
    }

    fn preview_area() -> Area {
        Area {
            min: Vec2::new(-12.0, -12.0),
            max: Vec2::new(12.0, 12.0),
        }
    }

    #[test]
    fn same_seed_and_params_produce_identical_output() {
        let area = sample_area();
        let params = GrassParams::default();
        let first = generate(42, area, &params);
        let second = generate(42, area, &params);
        assert_eq!(first, second);
    }

    #[test]
    fn different_seeds_produce_different_output() {
        let area = sample_area();
        let params = GrassParams::default();
        let first = generate(1, area, &params);
        let second = generate(2, area, &params);
        assert_ne!(first, second);
    }

    #[test]
    fn default_params_produce_plausible_non_empty_list() {
        let area = sample_area();
        let instances = generate(7, area, &GrassParams::default());

        assert!(!instances.is_empty());
        assert_eq!(instances.len(), 150); // 10×10 × density 1.5 = 150
    }

    #[test]
    fn generate_matches_manual_phase_wiring() {
        let area = preview_area();
        let params = GrassParams::default();
        let seed = 42;

        let manual = build_instances(
            seed,
            &place_positions(seed, &area, params.density),
            &params,
        );
        let unified = generate(seed, area, &params);

        assert_eq!(manual, unified);
    }
}
