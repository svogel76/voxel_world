//! Generate procedural textures into `crates/voxel_game/assets/textures/`.
//!
//! ```text
//! cargo run -p world_generator --example generate_terrain_textures
//! ```
//!
//! Writes:
//! - `terrain_array.png` — stacked Dirt/GrassTop/GrassSide/Stone for bvw
//! - `wood.png`, `moss.png`, `leaf.png`, `stone.png` — vegetation / props

use std::path::PathBuf;

use image::{ImageBuffer, Rgba};
use world_generator::voxel_textures::{
    generate_layer, generate_terrain_array, BlockLayer, LAYER_COUNT, LAYER_SIZE,
};

fn main() {
    let out_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../voxel_game/assets/textures");
    std::fs::create_dir_all(&out_dir).expect("create voxel_game/assets/textures");

    write_rgba(
        &out_dir.join("terrain_array.png"),
        LAYER_SIZE,
        LAYER_SIZE * LAYER_COUNT,
        generate_terrain_array(),
    );
    println!(
        "Wrote {} ({}x{}, {} terrain layers)",
        out_dir.join("terrain_array.png").display(),
        LAYER_SIZE,
        LAYER_SIZE * LAYER_COUNT,
        LAYER_COUNT
    );

    for layer in [
        BlockLayer::Stone,
        BlockLayer::Wood,
        BlockLayer::Moss,
        BlockLayer::Leaf,
    ] {
        let path = out_dir.join(layer.file_name());
        write_rgba(&path, LAYER_SIZE, LAYER_SIZE, generate_layer(layer));
        println!("Wrote {}", path.display());
    }
}

fn write_rgba(path: &PathBuf, width: u32, height: u32, pixels: Vec<u8>) {
    let img: ImageBuffer<Rgba<u8>, _> =
        ImageBuffer::from_raw(width, height, pixels).expect("RGBA buffer size mismatch");
    img.save(path)
        .unwrap_or_else(|e| panic!("failed to write {}: {e}", path.display()));
}
