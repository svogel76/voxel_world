//! Procedural block textures for terrain (bvw array) and vegetation (single PNGs).
//!
//! Terrain array: stacked `LAYER_SIZE × (LAYER_SIZE * LAYER_COUNT)` RGBA8.
//! Vegetation: same generators, written as individual PNGs and loaded as
//! `StandardMaterial` base-color textures.

use crate::noise::value_noise_2d;

/// Pixels per layer edge (blocky look; replaceable later with higher-res art).
pub const LAYER_SIZE: u32 = 32;
/// Number of stacked layers in the **terrain** array only.
pub const LAYER_COUNT: u32 = 4;

pub const LAYER_DIRT: u32 = 0;
pub const LAYER_GRASS_TOP: u32 = 1;
pub const LAYER_GRASS_SIDE: u32 = 2;
pub const LAYER_STONE: u32 = 3;

const TEX_SEED: u64 = 0x7E22_A1E5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockLayer {
    Dirt,
    GrassTop,
    GrassSide,
    Stone,
    Wood,
    Moss,
    Leaf,
}

impl BlockLayer {
    /// Layers written into `terrain_array.png` (bvw).
    pub const TERRAIN: [BlockLayer; 4] = [
        BlockLayer::Dirt,
        BlockLayer::GrassTop,
        BlockLayer::GrassSide,
        BlockLayer::Stone,
    ];

    /// Extra stylized tiles for vegetation cubes / future moss blending.
    pub const VEGETATION: [BlockLayer; 3] = [
        BlockLayer::Wood,
        BlockLayer::Moss,
        BlockLayer::Leaf,
    ];

    pub fn terrain_index(self) -> Option<u32> {
        match self {
            BlockLayer::Dirt => Some(LAYER_DIRT),
            BlockLayer::GrassTop => Some(LAYER_GRASS_TOP),
            BlockLayer::GrassSide => Some(LAYER_GRASS_SIDE),
            BlockLayer::Stone => Some(LAYER_STONE),
            _ => None,
        }
    }

    /// Asset filename under `textures/` (without path).
    pub fn file_name(self) -> &'static str {
        match self {
            BlockLayer::Dirt => "dirt.png",
            BlockLayer::GrassTop => "grass_top.png",
            BlockLayer::GrassSide => "grass_side.png",
            BlockLayer::Stone => "stone.png",
            BlockLayer::Wood => "wood.png",
            BlockLayer::Moss => "moss.png",
            BlockLayer::Leaf => "leaf.png",
        }
    }
}

/// Back-compat alias used by older call sites / docs.
pub type TerrainLayer = BlockLayer;

/// One layer as tightly packed RGBA8 (`LAYER_SIZE * LAYER_SIZE * 4` bytes).
pub fn generate_layer(layer: BlockLayer) -> Vec<u8> {
    let w = LAYER_SIZE as usize;
    let mut pixels = vec![0u8; w * w * 4];
    for y in 0..w {
        for x in 0..w {
            let (r, g, b) = sample_layer(layer, x, y);
            let i = (y * w + x) * 4;
            pixels[i] = r;
            pixels[i + 1] = g;
            pixels[i + 2] = b;
            pixels[i + 3] = 255;
        }
    }
    pixels
}

/// Stack terrain layers into one vertical strip for `bevy_voxel_world`.
pub fn generate_terrain_array() -> Vec<u8> {
    let w = LAYER_SIZE as usize;
    let h = (LAYER_SIZE * LAYER_COUNT) as usize;
    let mut out = vec![0u8; w * h * 4];
    for layer in BlockLayer::TERRAIN {
        let src = generate_layer(layer);
        let layer_y0 = (layer.terrain_index().unwrap() as usize) * w;
        for y in 0..w {
            for x in 0..w {
                let si = (y * w + x) * 4;
                let di = ((layer_y0 + y) * w + x) * 4;
                out[di..di + 4].copy_from_slice(&src[si..si + 4]);
            }
        }
    }
    out
}

fn sample_layer(layer: BlockLayer, x: usize, y: usize) -> (u8, u8, u8) {
    let u = x as f32 / LAYER_SIZE as f32;
    let v = y as f32 / LAYER_SIZE as f32;
    match layer {
        BlockLayer::Dirt => dirt(u, v),
        BlockLayer::GrassTop => grass_top(u, v),
        BlockLayer::GrassSide => grass_side(u, v),
        BlockLayer::Stone => stone(u, v),
        BlockLayer::Wood => wood(u, v),
        BlockLayer::Moss => moss(u, v),
        BlockLayer::Leaf => leaf(u, v),
    }
}

fn dirt(u: f32, v: f32) -> (u8, u8, u8) {
    let n = value_noise_2d(u * 6.0, v * 6.0, TEX_SEED);
    let n2 = value_noise_2d(u * 14.0 + 3.1, v * 14.0, TEX_SEED.wrapping_add(1));
    let t = 0.65 * n + 0.35 * n2;
    lerp_rgb((92, 58, 32), (140, 95, 55), t)
}

fn grass_top(u: f32, v: f32) -> (u8, u8, u8) {
    let n = value_noise_2d(u * 8.0, v * 8.0, TEX_SEED.wrapping_add(2));
    let n2 = value_noise_2d(u * 18.0, v * 18.0 + 1.7, TEX_SEED.wrapping_add(3));
    let t = 0.55 * n + 0.45 * n2;
    lerp_rgb((34, 92, 28), (72, 140, 48), t)
}

fn grass_side(u: f32, v: f32) -> (u8, u8, u8) {
    let fringe = 0.28;
    if v < fringe {
        let fade = v / fringe;
        lerp_rgb(grass_top(u, v), dirt(u, v), fade * fade)
    } else {
        dirt(u, v)
    }
}

fn stone(u: f32, v: f32) -> (u8, u8, u8) {
    let n = value_noise_2d(u * 5.0, v * 5.0, TEX_SEED.wrapping_add(4));
    let n2 = value_noise_2d(u * 22.0 + 0.5, v * 22.0, TEX_SEED.wrapping_add(5));
    let crack = if n2 < 0.22 { 0.35 } else { 1.0 };
    let t = n * crack;
    lerp_rgb((70, 72, 78), (145, 148, 155), t)
}

fn wood(u: f32, v: f32) -> (u8, u8, u8) {
    // Vertical grain + soft ring modulation.
    let grain = value_noise_2d(u * 3.0, v * 22.0, TEX_SEED.wrapping_add(6));
    let rings = ((u * 18.0 + grain * 2.0).sin() * 0.5 + 0.5).clamp(0.0, 1.0);
    let n = 0.55 * grain + 0.45 * rings;
    lerp_rgb((78, 48, 28), (148, 102, 58), n)
}

fn moss(u: f32, v: f32) -> (u8, u8, u8) {
    let n = value_noise_2d(u * 10.0, v * 10.0, TEX_SEED.wrapping_add(7));
    let clumps = value_noise_2d(u * 4.0 + 2.0, v * 4.0, TEX_SEED.wrapping_add(8));
    let t = (0.4 * n + 0.6 * clumps).clamp(0.0, 1.0);
    // Darker pockets read as denser moss.
    if clumps < 0.35 {
        lerp_rgb((18, 48, 22), (40, 88, 36), n)
    } else {
        lerp_rgb((40, 88, 36), (70, 130, 55), t)
    }
}

fn leaf(u: f32, v: f32) -> (u8, u8, u8) {
    let n = value_noise_2d(u * 9.0, v * 9.0, TEX_SEED.wrapping_add(9));
    let n2 = value_noise_2d(u * 20.0, v * 16.0 + 0.8, TEX_SEED.wrapping_add(10));
    let t = 0.5 * n + 0.5 * n2;
    lerp_rgb((28, 70, 30), (58, 120, 42), t)
}

fn lerp_rgb(a: (u8, u8, u8), b: (u8, u8, u8), t: f32) -> (u8, u8, u8) {
    let t = t.clamp(0.0, 1.0);
    (
        (a.0 as f32 + (b.0 as f32 - a.0 as f32) * t) as u8,
        (a.1 as f32 + (b.1 as f32 - a.1 as f32) * t) as u8,
        (a.2 as f32 + (b.2 as f32 - a.2 as f32) * t) as u8,
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn layer_byte_len_is_rgba() {
        let layer = generate_layer(BlockLayer::Dirt);
        assert_eq!(layer.len(), (LAYER_SIZE * LAYER_SIZE * 4) as usize);
        assert_eq!(layer[3], 255);
    }

    #[test]
    fn array_height_is_stacked_layers() {
        let arr = generate_terrain_array();
        assert_eq!(
            arr.len(),
            (LAYER_SIZE * LAYER_SIZE * LAYER_COUNT * 4) as usize
        );
    }

    #[test]
    fn layers_differ_at_same_pixel() {
        let dirt = generate_layer(BlockLayer::Dirt);
        let grass = generate_layer(BlockLayer::GrassTop);
        let stone = generate_layer(BlockLayer::Stone);
        let wood = generate_layer(BlockLayer::Wood);
        let i = ((LAYER_SIZE * LAYER_SIZE / 2) * 4) as usize;
        assert_ne!(&dirt[i..i + 3], &grass[i..i + 3]);
        assert_ne!(&dirt[i..i + 3], &stone[i..i + 3]);
        assert_ne!(&wood[i..i + 3], &stone[i..i + 3]);
    }
}
