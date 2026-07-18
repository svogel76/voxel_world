//! Terrain height access without depending on a concrete voxel engine.

/// Provides world-space terrain height at an `(x, z)` surface position.
///
/// Real games will later implement this against `bevy_voxel_world` inside
/// the game crate. `world_generator` only needs this trait so it stays Bevy-free.
pub trait TerrainHeightSource {
    fn height_at(&self, x: f32, z: f32) -> f32;
}

/// Fixed height everywhere — useful for targeted biome rule tests.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct ConstantHeight(pub f32);

impl TerrainHeightSource for ConstantHeight {
    fn height_at(&self, _x: f32, _z: f32) -> f32 {
        self.0
    }
}

/// Placeholder height field from this crate's own 2D value noise.
///
/// Independent of `rock_generator`; intended for local tests and later
/// Phase 5 visualization until a real terrain backend exists.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct SimpleNoiseTerrain {
    pub seed: u64,
    pub frequency: f32,
    pub amplitude: f32,
    pub base: f32,
}

impl Default for SimpleNoiseTerrain {
    fn default() -> Self {
        Self {
            seed: 0,
            frequency: 0.04,
            amplitude: 8.0,
            base: 4.0,
        }
    }
}

impl TerrainHeightSource for SimpleNoiseTerrain {
    fn height_at(&self, x: f32, z: f32) -> f32 {
        let n = crate::noise::value_noise_2d(x * self.frequency, z * self.frequency, self.seed);
        self.base + n * self.amplitude
    }
}
