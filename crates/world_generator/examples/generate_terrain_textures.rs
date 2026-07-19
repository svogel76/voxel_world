//! Generate `crates/voxel_game/assets/textures/terrain_array.png`.
//!
//! ```text
//! cargo run -p world_generator --example generate_terrain_textures
//! ```
//!
//! Re-run after tweaking [`world_generator::voxel_textures`] to refresh the PNG.
//! Bevy loads assets from the `voxel_game` package `assets/` folder.

use std::path::PathBuf;

use image::{ImageBuffer, Rgba};
use world_generator::voxel_textures::{generate_terrain_array, LAYER_COUNT, LAYER_SIZE};

fn main() {
    let pixels = generate_terrain_array();
    let width = LAYER_SIZE;
    let height = LAYER_SIZE * LAYER_COUNT;

    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, pixels).expect("buffer size matches LAYER_SIZE stack");

    let out = PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .join("../voxel_game/assets/textures/terrain_array.png");
    if let Some(parent) = out.parent() {
        std::fs::create_dir_all(parent).expect("create voxel_game/assets/textures");
    }
    img.save(&out)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", out.display()));
    println!(
        "Wrote {} ({}x{}, {} layers)",
        out.display(),
        width,
        height,
        LAYER_COUNT
    );
}
