#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Stone,
}

/// Parameters for noise-threshold rock generation (Phases 1–2).
#[derive(Debug, Clone, PartialEq)]
pub struct RockParams {
    /// Base inclusive half-size. Phase 2 scales this per axis via the seed.
    /// Sample range before variation is `-half_extent..=half_extent`.
    pub half_extent: i32,
    /// Keep a voxel when `noise - radial_falloff * distance > threshold`.
    /// Noise is in `[0, 1]`.
    pub threshold: f32,
    /// Scales positions before sampling noise. Higher = finer detail / more holes.
    pub noise_frequency: f32,
    /// Strength of the radial density bias. `0.0` disables it (pure threshold).
    /// Typical boulder values: roughly `0.3..=0.6`.
    pub radial_falloff: f32,
    /// Inclusive lower bound for seed-driven per-axis scale of `half_extent`.
    pub axis_scale_min: f32,
    /// Inclusive upper bound for seed-driven per-axis scale of `half_extent`.
    pub axis_scale_max: f32,
}

impl Default for RockParams {
    fn default() -> Self {
        Self {
            half_extent: 5,
            threshold: 0.5,
            noise_frequency: 0.35,
            radial_falloff: 0.45,
            axis_scale_min: 0.7,
            axis_scale_max: 1.3,
        }
    }
}
