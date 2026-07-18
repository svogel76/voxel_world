//! `bevy_voxel_world` configuration: streaming chunks + noise voxel lookup.
//!
//! The plugin owns chunking, meshing, and camera-based streaming. We only supply
//! which voxel sits at each `(x, y, z)` via [`VoxelWorldConfig::voxel_lookup_delegate`].

use std::sync::Arc;

use bevy::prelude::*;
use bevy_voxel_world::prelude::*;

use crate::height::{top_solid_y, WORLD_SEED, VoxelNoiseHeight};

/// Material indices written into [`WorldVoxel::Solid`].
/// Mapped through [`VoxelWorldConfig::texture_index_mapper`] onto the default
/// 4-layer texture shipped by `bevy_voxel_world` (textures come in Phase 3).
pub const MAT_DIRT: u8 = 0;
pub const MAT_GRASS: u8 = 1;
pub const MAT_STONE: u8 = 2;

#[derive(Resource, Clone, Default)]
pub struct VoxelTerrain;

impl VoxelWorldConfig for VoxelTerrain {
    type MaterialIndex = u8;
    type ChunkUserBundle = ();

    fn spawning_distance(&self) -> u32 {
        8
    }

    fn voxel_lookup_delegate(&self) -> VoxelLookupDelegate<Self::MaterialIndex> {
        Box::new(move |_chunk_pos, _lod, _previous| {
            // Same seed/params as vegetation placement (`VoxelNoiseHeight`).
            let height = VoxelNoiseHeight::new(WORLD_SEED);
            Box::new(move |pos: IVec3, _previous| lookup_voxel(pos, &height))
        })
    }

    fn texture_index_mapper(
        &self,
    ) -> Arc<dyn Fn(Self::MaterialIndex) -> [u32; 3] + Send + Sync> {
        Arc::new(|mat| match mat {
            MAT_GRASS => [1, 1, 1],
            MAT_STONE => [2, 2, 2],
            _ => [0, 0, 0],
        })
    }
}

/// Solid below the noise surface; grass on the top solid cell, dirt beneath, deep stone.
fn lookup_voxel(pos: IVec3, height: &VoxelNoiseHeight) -> WorldVoxel<u8> {
    let top_solid = top_solid_y(height, pos.x, pos.z);

    if pos.y > top_solid {
        return WorldVoxel::Air;
    }
    if pos.y == top_solid {
        return WorldVoxel::Solid(MAT_GRASS);
    }
    if pos.y > top_solid - 4 {
        return WorldVoxel::Solid(MAT_DIRT);
    }
    WorldVoxel::Solid(MAT_STONE)
}

/// Marker for the camera that drives chunk streaming (alongside [`VoxelWorldCamera`]).
#[derive(Component)]
pub struct TerrainCamera;
