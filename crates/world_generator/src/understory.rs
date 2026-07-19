//! Forest understory rules lifted from `examples/reference_scene.rs`.
//!
//! Keeps generators Bevy-free: fern carpets via `grass_generator`, bushes and
//! fallen logs as small voxel helpers (no separate crates yet).

use std::f32::consts::TAU;

use glam::{IVec3, Vec2};
use grass_generator::{GrassParams, VariantWeights};
use rand::rngs::StdRng;
use rand::{Rng, SeedableRng};

/// Corridor / floor fern density used by Forest biome params.
pub const FOREST_FLOOR_DENSITY: f32 = 2.0;
/// Denser fern belt around tree trunks (reference_scene trunk strips).
pub const TRUNK_FERN_DENSITY: f32 = 3.2;
/// Half-extent of the trunk-foot fern AABB in world XZ meters.
pub const TRUNK_FERN_HALF_EXTENT: f32 = 2.5;
/// Chance that a forest tree gets a nearby fallen log.
pub const FALLEN_LOG_CHANCE: f32 = 0.5;
/// Bushes placed per forest tree (fixed count keeps density predictable).
pub const BUSHES_PER_TREE: u32 = 2;

/// Fern-heavy grass params matching the reference-scene undergrowth mix.
pub fn fern_carpet_params(density: f32) -> GrassParams {
    GrassParams {
        density,
        variant_weights: VariantWeights {
            grass: 0.25,
            fern: 1.0,
        },
        ..GrassParams::default()
    }
}

/// Irregular leaf cluster (~1–2 m) relative to bush origin on the ground.
pub fn bush_cluster_voxels(seed: u64) -> Vec<IVec3> {
    let mut rng = StdRng::seed_from_u64(seed);
    let mut voxels = Vec::new();
    let height = rng.gen_range(1..=2);
    let radius = rng.gen_range(1..=2);
    for y in 0..=height {
        let layer_r = radius - (y / 2);
        let r = layer_r.max(1);
        for dx in -r..=r {
            for dz in -r..=r {
                if dx * dx + dz * dz > r * r {
                    continue;
                }
                // Sparse holes so bushes don't look like solid cubes.
                if rng.gen_bool(0.72) {
                    voxels.push(IVec3::new(dx, y, dz));
                }
            }
        }
    }
    voxels
}

/// Short fallen trunk relative to origin `(0,0,0)` on the ground plane.
///
/// Oriented in a seeded horizontal direction; ~5–8 m long, 2×2 cross-section
/// (same spirit as the hand-placed log in `reference_scene`).
pub fn fallen_log_voxels(seed: u64) -> Vec<IVec3> {
    let mut rng = StdRng::seed_from_u64(seed);
    let angle = rng.gen_range(0.0..TAU);
    let dir = Vec2::new(angle.cos(), angle.sin());
    // Start a few meters off the tree so the log does not sit inside the trunk.
    let start_offset = rng.gen_range(3.0..5.0);
    let steps = rng.gen_range(5..=8);
    let start = Vec2::new(dir.x * start_offset, dir.y * start_offset);

    let mut voxels = Vec::new();
    for i in 0..=steps {
        let t = i as f32 / steps as f32;
        let p = start + dir * (t * steps as f32);
        let center = IVec3::new(p.x.round() as i32, 0, p.y.round() as i32);
        for dy in 0..2 {
            for d in -1i32..=0 {
                voxels.push(center + IVec3::new(d, dy, 0));
                voxels.push(center + IVec3::new(0, dy, d));
            }
            voxels.push(center + IVec3::new(0, dy, 0));
        }
    }
    voxels.sort_by_key(|p| (p.x, p.y, p.z));
    voxels.dedup();
    voxels
}

/// Chance that an upper log face gets a moss pad on top.
const FALLEN_LOG_MOSS_CHANCE: f64 = 0.55;

/// Moss pads on the upper face of a fallen log (same idea as `reference_scene`).
///
/// Positions are relative to the same origin as `log` voxels. Only cells with
/// `y >= 1` are candidates so moss sits on the top of the 2-high trunk section.
pub fn fallen_log_moss_voxels(seed: u64, log: &[IVec3]) -> Vec<IVec3> {
    let mut rng = StdRng::seed_from_u64(seed.wrapping_add(0x4D_05_5E_ED));
    let mut moss = Vec::new();
    for p in log {
        if p.y >= 1 && rng.gen_bool(FALLEN_LOG_MOSS_CHANCE) {
            moss.push(IVec3::new(p.x, p.y + 1, p.z));
        }
    }
    moss.sort_by_key(|p| (p.x, p.y, p.z));
    moss.dedup();
    moss
}

/// Axis-aligned fern patch around a tree, clipped to the chunk area.
pub fn trunk_fern_area(tree_xz: Vec2, chunk: &grass_generator::Area) -> Option<grass_generator::Area> {
    let h = TRUNK_FERN_HALF_EXTENT;
    let min_x = (tree_xz.x - h).max(chunk.min.x);
    let max_x = (tree_xz.x + h).min(chunk.max.x);
    let min_z = (tree_xz.y - h).max(chunk.min.y);
    let max_z = (tree_xz.y + h).min(chunk.max.y);
    if max_x <= min_x || max_z <= min_z {
        return None;
    }
    Some(grass_generator::Area {
        min: Vec2::new(min_x, min_z),
        max: Vec2::new(max_x, max_z),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use grass_generator::Area;

    #[test]
    fn fern_carpet_is_fern_biased() {
        let params = fern_carpet_params(FOREST_FLOOR_DENSITY);
        assert!(params.variant_weights.fern > params.variant_weights.grass);
        assert!((params.density - FOREST_FLOOR_DENSITY).abs() < f32::EPSILON);
    }

    #[test]
    fn bush_cluster_is_non_empty_and_deterministic() {
        let a = bush_cluster_voxels(77);
        let b = bush_cluster_voxels(77);
        assert_eq!(a, b);
        assert!(!a.is_empty());
        assert!(a.iter().all(|p| p.y >= 0 && p.y <= 2));
    }

    #[test]
    fn fallen_log_is_non_empty_and_deterministic() {
        let a = fallen_log_voxels(99);
        let b = fallen_log_voxels(99);
        assert_eq!(a, b);
        assert!(a.len() >= 10);
        assert!(a.iter().all(|p| p.y == 0 || p.y == 1));
    }

    #[test]
    fn fallen_log_moss_is_deterministic_and_above_log() {
        let log = fallen_log_voxels(99);
        let a = fallen_log_moss_voxels(99, &log);
        let b = fallen_log_moss_voxels(99, &log);
        assert_eq!(a, b);
        assert!(!a.is_empty());
        assert!(a.iter().all(|p| p.y >= 2));
        for m in &a {
            assert!(
                log.iter().any(|l| l.x == m.x && l.z == m.z && l.y == m.y - 1),
                "moss {m:?} must sit on a log cell"
            );
        }
    }

    #[test]
    fn trunk_fern_area_clips_to_chunk() {
        let chunk = Area {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(10.0, 10.0),
        };
        let area = trunk_fern_area(Vec2::new(1.0, 1.0), &chunk).unwrap();
        assert_eq!(area.min, Vec2::new(0.0, 0.0));
        assert_eq!(area.max, Vec2::new(1.0 + TRUNK_FERN_HALF_EXTENT, 1.0 + TRUNK_FERN_HALF_EXTENT));
    }

    #[test]
    fn trunk_fern_area_none_when_outside() {
        let chunk = Area {
            min: Vec2::new(0.0, 0.0),
            max: Vec2::new(10.0, 10.0),
        };
        assert!(trunk_fern_area(Vec2::new(-20.0, -20.0), &chunk).is_none());
    }
}
