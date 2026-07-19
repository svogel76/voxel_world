//! Procedural block textures for `bevy_voxel_world` array textures.
//!
//! Output is raw RGBA8 pixels. Stack layers top-to-bottom into a single image of
//! size `LAYER_SIZE × (LAYER_SIZE * LAYER_COUNT)` — the format bvw expects.

use crate::noise::value_noise_2d;

/// Pixels per layer edge (blocky look; replaceable later with higher-res art).
pub const LAYER_SIZE: u32 = 32;
/// Number of stacked layers in the terrain array.
pub const LAYER_COUNT: u32 = 4;

pub const LAYER_DIRT: u32 = 0;
pub const LAYER_GRASS_TOP: u32 = 1;
pub const LAYER_GRASS_SIDE: u32 = 2;
pub const LAYER_STONE: u32 = 3;

const TEX_SEED: u64 = 0x7E22_A1E5;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum TerrainLayer {
    Dirt,
    GrassTop,
    GrassSide,
    Stone,
}

impl TerrainLayer {
    pub const ALL: [TerrainLayer; 4] = [
        TerrainLayer::Dirt,
        TerrainLayer::GrassTop,
        TerrainLayer::GrassSide,
        TerrainLayer::Stone,
    ];

    pub fn index(self) -> u32 {
        match self {
            TerrainLayer::Dirt => LAYER_DIRT,
            TerrainLayer::GrassTop => LAYER_GRASS_TOP,
            TerrainLayer::GrassSide => LAYER_GRASS_SIDE,
            TerrainLayer::Stone => LAYER_STONE,
        }
    }
}

/// One layer as tightly packed RGBA8 (`LAYER_SIZE * LAYER_SIZE * 4` bytes).
pub fn generate_layer(layer: TerrainLayer) -> Vec<u8> {
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

/// Stack all terrain layers into one vertical strip: width `LAYER_SIZE`,
/// height `LAYER_SIZE * LAYER_COUNT`.
pub fn generate_terrain_array() -> Vec<u8> {
    let w = LAYER_SIZE as usize;
    let h = (LAYER_SIZE * LAYER_COUNT) as usize;
    let mut out = vec![0u8; w * h * 4];
    for layer in TerrainLayer::ALL {
        let src = generate_layer(layer);
        let layer_y0 = (layer.index() as usize) * w;
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

fn sample_layer(layer: TerrainLayer, x: usize, y: usize) -> (u8, u8, u8) {
    let u = x as f32 / LAYER_SIZE as f32;
    let v = y as f32 / LAYER_SIZE as f32;
    match layer {
        TerrainLayer::Dirt => dirt(u, v),
        TerrainLayer::GrassTop => grass_top(u, v),
        TerrainLayer::GrassSide => grass_side(u, v),
        TerrainLayer::Stone => stone(u, v),
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
    // Classic block look: grass fringe on top of the face, dirt below.
    let fringe = 0.28;
    if v < fringe {
        let fade = v / fringe;
        let g = grass_top(u, v);
        let d = dirt(u, v);
        lerp_rgb(g, d, fade * fade)
    } else {
        dirt(u, v)
    }
}

fn stone(u: f32, v: f32) -> (u8, u8, u8) {
    let n = value_noise_2d(u * 5.0, v * 5.0, TEX_SEED.wrapping_add(4));
    let n2 = value_noise_2d(u * 22.0 + 0.5, v * 22.0, TEX_SEED.wrapping_add(5));
    // Occasional darker “cracks” from high-frequency dips.
    let crack = if n2 < 0.22 { 0.35 } else { 1.0 };
    let t = n * crack;
    lerp_rgb((70, 72, 78), (145, 148, 155), t)
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
        let layer = generate_layer(TerrainLayer::Dirt);
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
        let dirt = generate_layer(TerrainLayer::Dirt);
        let grass = generate_layer(TerrainLayer::GrassTop);
        let stone = generate_layer(TerrainLayer::Stone);
        let i = ((LAYER_SIZE * LAYER_SIZE / 2) * 4) as usize;
        assert_ne!(&dirt[i..i + 3], &grass[i..i + 3]);
        assert_ne!(&dirt[i..i + 3], &stone[i..i + 3]);
    }
}
