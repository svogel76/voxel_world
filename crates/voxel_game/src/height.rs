//! Shared height source used by voxel fill **and** `world_generator`.
//!
//! `bevy_voxel_world`'s `get_voxel` returns `Unset` for unloaded chunks, so vegetation
//! placement must not query the voxel world. Instead we reuse the same public
//! [`SimpleNoiseTerrain`](world_generator::SimpleNoiseTerrain) that the voxel lookup uses.

use world_generator::{SimpleNoiseTerrain, TerrainHeightSource};

/// World seed shared by terrain voxels and vegetation placement.
pub const WORLD_SEED: u64 = 42;

/// Keep these in sync with the values baked into [`VoxelNoiseHeight::default_world`].
pub const TERRAIN_FREQUENCY: f32 = 0.02;
pub const TERRAIN_AMPLITUDE: f32 = 12.0;
pub const TERRAIN_BASE_HEIGHT: f32 = 8.0;

/// Thin wrapper so voxel fill and `generate_chunk` share one height function.
#[derive(Clone, Copy)]
pub struct VoxelNoiseHeight {
    inner: SimpleNoiseTerrain,
}

impl VoxelNoiseHeight {
    pub fn new(seed: u64) -> Self {
        Self {
            inner: SimpleNoiseTerrain {
                seed,
                frequency: TERRAIN_FREQUENCY,
                amplitude: TERRAIN_AMPLITUDE,
                base: TERRAIN_BASE_HEIGHT,
            },
        }
    }

    pub fn default_world() -> Self {
        Self::new(WORLD_SEED)
    }
}

impl TerrainHeightSource for VoxelNoiseHeight {
    fn height_at(&self, x: f32, z: f32) -> f32 {
        self.inner.height_at(x, z)
    }
}

/// Integer Y of the topmost solid voxel column at `(x, z)` (same rule as the voxel lookup).
pub fn top_solid_y(height: &impl TerrainHeightSource, x: i32, z: i32) -> i32 {
    height.height_at(x as f32, z as f32).floor() as i32 - 1
}

#[cfg(test)]
mod tests {
    use super::*;
    use world_generator::TerrainHeightSource;

    #[test]
    fn height_is_deterministic() {
        let a = VoxelNoiseHeight::default_world();
        let b = VoxelNoiseHeight::default_world();
        assert_eq!(a.height_at(10.0, -3.0), b.height_at(10.0, -3.0));
    }

    #[test]
    fn height_is_plausible() {
        let h = VoxelNoiseHeight::default_world();
        let y = h.height_at(0.0, 0.0);
        assert!(
            y > TERRAIN_BASE_HEIGHT - TERRAIN_AMPLITUDE - 1.0
                && y < TERRAIN_BASE_HEIGHT + TERRAIN_AMPLITUDE + 1.0,
            "height {y} out of expected noise band"
        );
    }

    #[test]
    fn top_solid_matches_floor_rule() {
        let h = VoxelNoiseHeight::default_world();
        let x = 5;
        let z = -2;
        let surface = h.height_at(x as f32, z as f32);
        assert_eq!(top_solid_y(&h, x, z), surface.floor() as i32 - 1);
    }
}
