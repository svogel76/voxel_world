use std::collections::{HashSet, VecDeque};

use glam::IVec3;

use crate::noise::value_noise_3d;
use crate::types::{BlockType, RockParams};
use crate::variation::axis_extents;

const NEIGHBOR_OFFSETS: [IVec3; 6] = [
    IVec3::new(1, 0, 0),
    IVec3::new(-1, 0, 0),
    IVec3::new(0, 1, 0),
    IVec3::new(0, -1, 0),
    IVec3::new(0, 0, 1),
    IVec3::new(0, 0, -1),
];

/// Generate a rock clump as stone voxels centered at the origin.
///
/// Pipeline:
/// 1. Seed-driven per-axis extents ([`crate::variation::axis_extents`])
/// 2. Sample the anisotropic box with noise − radial falloff
/// 3. Keep only the largest 6-connected component
pub fn generate(seed: u64, params: &RockParams) -> Vec<(IVec3, BlockType)> {
    let extents = axis_extents(seed, params);
    let raw = sample_threshold(seed, params, extents);
    keep_largest_component(raw)
}

fn sample_threshold(
    seed: u64,
    params: &RockParams,
    extents: IVec3,
) -> Vec<(IVec3, BlockType)> {
    let freq = params.noise_frequency;
    let mut voxels = Vec::new();

    for z in -extents.z..=extents.z {
        for y in -extents.y..=extents.y {
            for x in -extents.x..=extents.x {
                let noise = value_noise_3d(x as f32 * freq, y as f32 * freq, z as f32 * freq, seed);
                let score = noise - params.radial_falloff * ellipsoid_distance(x, y, z, extents);
                if score > params.threshold {
                    voxels.push((IVec3::new(x, y, z), BlockType::Stone));
                }
            }
        }
    }

    voxels
}

/// Normalized ellipsoid distance: 0 at the origin, 1 at the axis-aligned
/// extremes of `extents` (corners reach up to `sqrt(3)`).
fn ellipsoid_distance(x: i32, y: i32, z: i32, extents: IVec3) -> f32 {
    let nx = x as f32 / extents.x.max(1) as f32;
    let ny = y as f32 / extents.y.max(1) as f32;
    let nz = z as f32 / extents.z.max(1) as f32;
    (nx * nx + ny * ny + nz * nz).sqrt()
}

/// Mandatory post-process: discard every voxel outside the largest
/// 6-connected component so automatic multi-seed placement never emits islands.
fn keep_largest_component(voxels: Vec<(IVec3, BlockType)>) -> Vec<(IVec3, BlockType)> {
    if voxels.len() <= 1 {
        return voxels;
    }

    let positions: HashSet<IVec3> = voxels.iter().map(|(p, _)| *p).collect();
    let largest = largest_component(&positions);
    voxels
        .into_iter()
        .filter(|(p, _)| largest.contains(p))
        .collect()
}

fn largest_component(positions: &HashSet<IVec3>) -> HashSet<IVec3> {
    let mut remaining = positions.clone();
    let mut best: HashSet<IVec3> = HashSet::new();
    let mut best_min = IVec3::new(i32::MAX, i32::MAX, i32::MAX);

    while let Some(&start) = remaining.iter().next() {
        let component = flood_fill(start, &mut remaining);
        let size = component.len();
        let min_pos = component.iter().copied().min_by_key(|p| (p.x, p.y, p.z)).unwrap();

        let replaces = size > best.len()
            || (size == best.len()
                && (min_pos.x, min_pos.y, min_pos.z) < (best_min.x, best_min.y, best_min.z));
        if replaces {
            best_min = min_pos;
            best = component;
        }
    }

    best
}

fn flood_fill(start: IVec3, remaining: &mut HashSet<IVec3>) -> HashSet<IVec3> {
    let mut component = HashSet::new();
    let mut queue = VecDeque::from([start]);
    remaining.remove(&start);

    while let Some(p) = queue.pop_front() {
        component.insert(p);
        for offset in &NEIGHBOR_OFFSETS {
            let neighbor = p + *offset;
            if remaining.remove(&neighbor) {
                queue.push_back(neighbor);
            }
        }
    }

    component
}

/// Count 6-connected components (test / analysis helper).
#[cfg(test)]
fn component_count(positions: &HashSet<IVec3>) -> usize {
    let mut remaining = positions.clone();
    let mut count = 0;
    while let Some(&start) = remaining.iter().next() {
        flood_fill(start, &mut remaining);
        count += 1;
    }
    count
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Phase-1-style cubic sampling: no radial bias, no axis stretch.
    fn phase1_params(half_extent: i32, threshold: f32) -> RockParams {
        RockParams {
            half_extent,
            threshold,
            noise_frequency: 0.35,
            radial_falloff: 0.0,
            axis_scale_min: 1.0,
            axis_scale_max: 1.0,
        }
    }

    fn default_params() -> RockParams {
        phase1_params(4, 0.5)
    }

    #[test]
    fn same_seed_produces_identical_voxels() {
        let params = default_params();
        let a = generate(42, &params);
        let b = generate(42, &params);
        assert_eq!(a, b);
        assert!(!a.is_empty(), "seeded rock should contain some stone");
    }

    #[test]
    fn different_seeds_produce_different_shapes() {
        let params = default_params();
        let a = generate(1, &params);
        let b = generate(2, &params);
        assert_ne!(a, b);
    }

    #[test]
    fn high_threshold_yields_empty_or_nearly_empty() {
        let params = phase1_params(5, 0.99);
        let voxels = generate(99, &params);
        let volume = (2 * params.half_extent + 1).pow(3);
        assert!(
            voxels.len() * 20 < volume as usize,
            "expected sparse fill, got {} / {volume}",
            voxels.len()
        );
    }

    #[test]
    fn low_threshold_fills_nearly_the_whole_box() {
        let params = phase1_params(5, 0.01);
        let voxels = generate(99, &params);
        let volume = (2 * params.half_extent + 1).pow(3) as usize;
        assert!(
            voxels.len() * 100 > volume * 80,
            "expected dense fill, got {} / {volume}",
            voxels.len()
        );
    }

    #[test]
    fn all_positions_lie_inside_bounding_box() {
        let params = default_params();
        let h = params.half_extent;
        for (pos, block) in generate(42, &params) {
            assert_eq!(block, BlockType::Stone);
            assert!(pos.x >= -h && pos.x <= h);
            assert!(pos.y >= -h && pos.y <= h);
            assert!(pos.z >= -h && pos.z <= h);
        }
    }

    #[test]
    fn every_voxel_is_stone_and_unique() {
        let params = default_params();
        let voxels = generate(7, &params);
        let positions: HashSet<IVec3> = voxels.iter().map(|(p, _)| *p).collect();
        assert_eq!(positions.len(), voxels.len());
        assert!(voxels.iter().all(|(_, b)| *b == BlockType::Stone));
    }

    #[test]
    fn high_threshold_result_is_exactly_one_connected_component() {
        // Analysis seed/params known to produce many islands before the filter.
        let params = phase1_params(5, 0.75);
        let voxels = generate(42, &params);
        assert!(!voxels.is_empty());
        let positions: HashSet<IVec3> = voxels.iter().map(|(p, _)| *p).collect();
        assert_eq!(component_count(&positions), 1);
    }

    #[test]
    fn generate_always_returns_at_most_one_connected_component() {
        for seed in [1u64, 7, 42, 99, 12345] {
            for threshold in [0.25, 0.5, 0.75] {
                let params = phase1_params(5, threshold);
                let voxels = generate(seed, &params);
                let positions: HashSet<IVec3> = voxels.iter().map(|(p, _)| *p).collect();
                assert!(
                    component_count(&positions) <= 1,
                    "seed={seed} threshold={threshold} has multiple components"
                );
            }
        }
    }

    #[test]
    fn connectivity_filter_barely_changes_dense_mid_results() {
        // Raw sample (no component filter) vs generate() for thresholds
        // known from analysis to be already mostly connected.
        let seed = 42u64;
        let extents = IVec3::splat(5);

        for (threshold, max_removed_ratio) in [(0.25, 0.01), (0.50, 0.05)] {
            let params = phase1_params(5, threshold);
            let raw = sample_threshold(seed, &params, extents);
            let filtered = generate(seed, &params);
            let removed = raw.len() - filtered.len();
            assert!(
                (removed as f32) <= raw.len() as f32 * max_removed_ratio,
                "threshold={threshold}: removed {removed} of {}, filter should barely apply",
                raw.len()
            );
            assert_eq!(filtered.len(), keep_largest_component(raw).len());
        }
    }

    // --- Phase 2 ---

    fn phase2_params() -> RockParams {
        RockParams::default()
    }

    #[test]
    fn phase2_same_seed_is_deterministic() {
        let params = phase2_params();
        assert_eq!(generate(11, &params), generate(11, &params));
    }

    #[test]
    fn phase2_different_seeds_produce_different_shapes() {
        let params = phase2_params();
        assert_ne!(generate(1, &params), generate(2, &params));
    }

    #[test]
    fn phase2_result_is_one_connected_component() {
        let params = phase2_params();
        for seed in [1u64, 7, 42, 99] {
            let positions: HashSet<IVec3> = generate(seed, &params).iter().map(|(p, _)| *p).collect();
            assert_eq!(component_count(&positions), 1, "seed={seed}");
        }
    }

    #[test]
    fn phase2_voxels_lie_inside_seed_extents() {
        let params = phase2_params();
        let seed = 42u64;
        let extents = axis_extents(seed, &params);
        for (pos, _) in generate(seed, &params) {
            assert!(pos.x.abs() <= extents.x);
            assert!(pos.y.abs() <= extents.y);
            assert!(pos.z.abs() <= extents.z);
        }
    }

    #[test]
    fn radial_falloff_makes_core_denser_than_rim() {
        let params = RockParams {
            half_extent: 6,
            threshold: 0.35,
            noise_frequency: 0.3,
            radial_falloff: 0.55,
            axis_scale_min: 1.0,
            axis_scale_max: 1.0,
        };
        let voxels = generate(42, &params);
        assert!(!voxels.is_empty());

        let mut core = 0usize;
        let mut core_total = 0usize;
        let mut rim = 0usize;
        let mut rim_total = 0usize;

        let set: HashSet<IVec3> = voxels.iter().map(|(p, _)| *p).collect();
        for z in -params.half_extent..=params.half_extent {
            for y in -params.half_extent..=params.half_extent {
                for x in -params.half_extent..=params.half_extent {
                    let dist = ellipsoid_distance(x, y, z, IVec3::splat(params.half_extent));
                    let filled = set.contains(&IVec3::new(x, y, z));
                    if dist <= 0.35 {
                        core_total += 1;
                        if filled {
                            core += 1;
                        }
                    } else if dist >= 0.85 {
                        rim_total += 1;
                        if filled {
                            rim += 1;
                        }
                    }
                }
            }
        }

        let core_fill = core as f32 / core_total.max(1) as f32;
        let rim_fill = rim as f32 / rim_total.max(1) as f32;
        assert!(
            core_fill > rim_fill + 0.2,
            "expected denser core than rim, core={core_fill:.2} rim={rim_fill:.2}"
        );
    }
}
