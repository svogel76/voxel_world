use std::collections::HashSet;

use glam::IVec3;

use crate::types::{BlockType, Segment};

/// Cross-section used when thickening line segments into voxels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossSectionShape {
    Sphere,
    #[default]
    Cube,
}

/// Convert line segments into unique wood voxel positions using [`CrossSectionShape::Cube`].
pub fn voxelize(segments: &[Segment]) -> Vec<(IVec3, BlockType)> {
    voxelize_with_shape(segments, CrossSectionShape::default())
}

/// Convert line segments into unique wood voxel positions with an explicit cross-section.
pub fn voxelize_with_shape(
    segments: &[Segment],
    shape: CrossSectionShape,
) -> Vec<(IVec3, BlockType)> {
    let mut voxels = HashSet::new();

    for segment in segments {
        let start = to_voxel(segment.start);
        let end = to_voxel(segment.end);
        let delta = end - start;
        let radius = voxel_radius(segment.thickness);

        for center in walk_line(start, end) {
            match shape {
                CrossSectionShape::Sphere => {
                    fill_cross_section_sphere(center, radius, &mut voxels);
                }
                CrossSectionShape::Cube => {
                    fill_cross_section_cube(center, radius, delta, &mut voxels);
                }
            }
        }
    }

    let mut result: Vec<_> = voxels
        .into_iter()
        .map(|position| (position, BlockType::Wood))
        .collect();
    result.sort_by_key(|(position, _)| (position.x, position.y, position.z));
    result
}

/// Extra voxels beyond the branch cross-section for a voluminous leaf crown.
const LEAF_CROWN_EXTENSION: i32 = 3;

/// Controls which branch tips receive leaf clusters based on turtle branch depth.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LeafPlacement {
    /// How many outermost branch-depth levels receive leaves.
    ///
    /// `1` = only tips at the tree's maximum segment depth; `2` = max depth and
    /// one level below (typical crown); larger values widen the leafy region.
    pub crown_levels: u32,
}

impl Default for LeafPlacement {
    fn default() -> Self {
        Self { crown_levels: 2 }
    }
}

impl LeafPlacement {
    /// Place leaves on every branch tip regardless of depth.
    pub const ALL: Self = Self {
        crown_levels: u32::MAX,
    };
}

/// Extend a wood voxel list with leaf clusters at branch tips.
///
/// A branch tip is a segment end that is not the start of another segment.
/// Only tips whose ending segment depth satisfies [`LeafPlacement`] are leafy.
/// Leaf radius scales with the ending segment's thickness:
/// `leaf_radius = voxel_radius(thickness) + LEAF_CROWN_EXTENSION`.
/// Wood positions take precedence: leaves are never placed on existing wood.
pub fn add_leaves(
    wood_voxels: &[(IVec3, BlockType)],
    segments: &[Segment],
    placement: LeafPlacement,
) -> Vec<(IVec3, BlockType)> {
    let wood_positions: HashSet<IVec3> = wood_voxels.iter().map(|(position, _)| *position).collect();
    let mut leaf_positions = HashSet::new();
    let min_leaf_depth = min_leaf_depth(segments, placement);

    for (tip, thickness, depth) in branch_tips(segments) {
        if depth < min_leaf_depth {
            continue;
        }

        let leaf_radius = leaf_cluster_radius(thickness);
        collect_sphere_cluster(tip, leaf_radius, &wood_positions, &mut leaf_positions);
    }

    let mut result: Vec<_> = wood_voxels.to_vec();
    let mut leaves: Vec<_> = leaf_positions
        .into_iter()
        .map(|position| (position, BlockType::Leaf))
        .collect();
    leaves.sort_by_key(|(position, _)| (position.x, position.y, position.z));
    result.extend(leaves);
    result.sort_by_key(|(position, _)| (position.x, position.y, position.z));
    result
}

fn leaf_cluster_radius(thickness: f32) -> i32 {
    voxel_radius(thickness) + LEAF_CROWN_EXTENSION
}

fn max_segment_depth(segments: &[Segment]) -> u32 {
    segments.iter().map(|segment| segment.depth).max().unwrap_or(0)
}

fn min_leaf_depth(segments: &[Segment], placement: LeafPlacement) -> u32 {
    let max_depth = max_segment_depth(segments);
    max_depth.saturating_sub(placement.crown_levels.saturating_sub(1))
}

fn branch_tips(segments: &[Segment]) -> Vec<(IVec3, f32, u32)> {
    let mut starts = HashSet::new();

    for segment in segments {
        starts.insert(to_voxel(segment.start));
    }

    let mut tips: Vec<_> = segments
        .iter()
        .filter(|segment| !starts.contains(&to_voxel(segment.end)))
        .map(|segment| (to_voxel(segment.end), segment.thickness, segment.depth))
        .collect();
    tips.sort_by_key(|(position, _, _)| (position.x, position.y, position.z));
    tips
}

fn collect_sphere_cluster(
    center: IVec3,
    radius: i32,
    wood_positions: &HashSet<IVec3>,
    leaf_positions: &mut HashSet<IVec3>,
) {
    if radius == 0 {
        if !wood_positions.contains(&center) {
            leaf_positions.insert(center);
        }
        return;
    }

    let radius_sq = radius * radius;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dy * dy + dz * dz <= radius_sq {
                    let position = center + IVec3::new(dx, dy, dz);
                    if !wood_positions.contains(&position) {
                        leaf_positions.insert(position);
                    }
                }
            }
        }
    }
}

fn to_voxel(position: glam::Vec3) -> IVec3 {
    IVec3::new(
        position.x.round() as i32,
        position.y.round() as i32,
        position.z.round() as i32,
    )
}

/// `thickness <= 1.0` maps to a single-voxel centerline; larger values grow a
/// cross-section around each line voxel.
fn voxel_radius(thickness: f32) -> i32 {
    if thickness <= 1.0 {
        0
    } else {
        (thickness / 2.0).ceil() as i32
    }
}

/// 3D line walk using the dominant axis: `max(|dx|, |dy|, |dz|)` steps with
/// linear interpolation and rounding on the other axes (3D Bresenham-style).
fn walk_line(start: IVec3, end: IVec3) -> Vec<IVec3> {
    let delta = end - start;
    let steps = delta.x.abs().max(delta.y.abs()).max(delta.z.abs());

    if steps == 0 {
        return vec![start];
    }

    let mut points = Vec::with_capacity((steps + 1) as usize);
    for step in 0..=steps {
        let t = step as f32 / steps as f32;
        let point = IVec3::new(
            (start.x as f32 + delta.x as f32 * t).round() as i32,
            (start.y as f32 + delta.y as f32 * t).round() as i32,
            (start.z as f32 + delta.z as f32 * t).round() as i32,
        );
        if points.last() != Some(&point) {
            points.push(point);
        }
    }
    points
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum SegmentAxis {
    X,
    Y,
    Z,
}

fn dominant_segment_axis(delta: IVec3) -> SegmentAxis {
    let abs_x = delta.x.abs();
    let abs_y = delta.y.abs();
    let abs_z = delta.z.abs();

    if abs_y >= abs_x && abs_y >= abs_z {
        SegmentAxis::Y
    } else if abs_x >= abs_z {
        SegmentAxis::X
    } else {
        SegmentAxis::Z
    }
}

fn fill_cross_section_sphere(center: IVec3, radius: i32, voxels: &mut HashSet<IVec3>) {
    if radius == 0 {
        voxels.insert(center);
        return;
    }

    let radius_sq = radius * radius;
    for dx in -radius..=radius {
        for dy in -radius..=radius {
            for dz in -radius..=radius {
                if dx * dx + dy * dy + dz * dz <= radius_sq {
                    voxels.insert(center + IVec3::new(dx, dy, dz));
                }
            }
        }
    }
}

/// Fills a square cross-section perpendicular to the segment axis. The axis
/// direction itself is not thickened at each waypoint, avoiding gaps along the
/// segment.
fn fill_cross_section_cube(
    center: IVec3,
    radius: i32,
    segment_delta: IVec3,
    voxels: &mut HashSet<IVec3>,
) {
    if radius == 0 {
        voxels.insert(center);
        return;
    }

    match dominant_segment_axis(segment_delta) {
        SegmentAxis::Y => {
            for dx in -radius..=radius {
                for dz in -radius..=radius {
                    voxels.insert(center + IVec3::new(dx, 0, dz));
                }
            }
        }
        SegmentAxis::X => {
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    voxels.insert(center + IVec3::new(0, dy, dz));
                }
            }
        }
        SegmentAxis::Z => {
            for dx in -radius..=radius {
                for dy in -radius..=radius {
                    voxels.insert(center + IVec3::new(dx, dy, 0));
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::Segment;
    use glam::Vec3;

    fn wood_at(positions: &[(i32, i32, i32)]) -> Vec<(IVec3, BlockType)> {
        let mut voxels: Vec<_> = positions
            .iter()
            .map(|&(x, y, z)| (IVec3::new(x, y, z), BlockType::Wood))
            .collect();
        voxels.sort_by_key(|(position, _)| (position.x, position.y, position.z));
        voxels
    }

    fn segment(start: (f32, f32, f32), end: (f32, f32, f32), thickness: f32) -> Segment {
        segment_with_depth(start, end, thickness, 0)
    }

    fn segment_with_depth(
        start: (f32, f32, f32),
        end: (f32, f32, f32),
        thickness: f32,
        depth: u32,
    ) -> Segment {
        Segment {
            start: Vec3::new(start.0, start.1, start.2),
            end: Vec3::new(end.0, end.1, end.2),
            thickness,
            depth,
        }
    }

    fn expected_voxels_for_segment(
        start: IVec3,
        end: IVec3,
        thickness: f32,
        shape: CrossSectionShape,
    ) -> HashSet<IVec3> {
        let delta = end - start;
        let radius = voxel_radius(thickness);
        let mut voxels = HashSet::new();

        for center in walk_line(start, end) {
            match shape {
                CrossSectionShape::Sphere => {
                    fill_cross_section_sphere(center, radius, &mut voxels);
                }
                CrossSectionShape::Cube => {
                    fill_cross_section_cube(center, radius, delta, &mut voxels);
                }
            }
        }

        voxels
    }

    fn voxel_positions(voxels: &[(IVec3, BlockType)]) -> HashSet<IVec3> {
        voxels.iter().map(|(position, _)| *position).collect()
    }

    fn leaf_count(voxels: &[(IVec3, BlockType)]) -> usize {
        voxels
            .iter()
            .filter(|(_, block_type)| *block_type == BlockType::Leaf)
            .count()
    }

    fn sphere_voxel_count(radius: i32) -> usize {
        if radius == 0 {
            return 1;
        }

        let radius_sq = radius * radius;
        let mut count = 0;
        for dx in -radius..=radius {
            for dy in -radius..=radius {
                for dz in -radius..=radius {
                    if dx * dx + dy * dy + dz * dz <= radius_sq {
                        count += 1;
                    }
                }
            }
        }
        count
    }

    #[test]
    fn leaf_cluster_radius_scales_with_branch_thickness() {
        assert_eq!(leaf_cluster_radius(1.0), 3, "thin tip: branch r=0 + extension 3");
        assert_eq!(leaf_cluster_radius(2.0), 4, "thickness 2 -> branch r=1 + extension 3");
        assert_eq!(leaf_cluster_radius(4.0), 5, "thickness 4 -> branch r=2 + extension 3");
    }

    #[test]
    fn voxelize_default_shape_is_cube() {
        assert_eq!(CrossSectionShape::default(), CrossSectionShape::Cube);

        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 4.0, 0.0), 4.0)];
        assert_eq!(
            voxelize(&segments),
            voxelize_with_shape(&segments, CrossSectionShape::Cube)
        );
    }

    #[test]
    fn leaves_only_on_outermost_branch_depth_levels() {
        let segments = vec![
            segment_with_depth((0.0, 0.0, 0.0), (0.0, 2.0, 0.0), 2.0, 0),
            segment_with_depth((0.0, 2.0, 0.0), (0.0, 10.0, 0.0), 1.0, 1),
            segment_with_depth((0.0, 2.0, 0.0), (6.0, 5.0, 0.0), 1.0, 2),
            segment_with_depth((0.0, 2.0, 0.0), (-6.0, 5.0, 0.0), 1.0, 3),
        ];
        let wood = voxelize(&segments);
        let voxels = add_leaves(&wood, &segments, LeafPlacement { crown_levels: 2 });

        let leaves_near_tip = |tip: IVec3, thickness: f32| {
            let radius = leaf_cluster_radius(thickness);
            let radius_sq = radius * radius;
            voxels.iter().any(|(position, block_type)| {
                if *block_type != BlockType::Leaf {
                    return false;
                }

                let offset = *position - tip;
                offset.x * offset.x + offset.y * offset.y + offset.z * offset.z <= radius_sq
            })
        };

        assert_eq!(max_segment_depth(&segments), 3);
        assert_eq!(min_leaf_depth(&segments, LeafPlacement { crown_levels: 2 }), 2);

        assert!(
            leaves_near_tip(IVec3::new(6, 5, 0), 1.0),
            "depth-2 tip should receive a leaf cluster"
        );
        assert!(
            leaves_near_tip(IVec3::new(-6, 5, 0), 1.0),
            "depth-3 tip should receive a leaf cluster"
        );
        assert!(
            !leaves_near_tip(IVec3::new(0, 10, 0), 1.0),
            "depth-1 tip must stay leafless when crown_levels=2 on this tree"
        );
    }

    #[test]
    fn min_leaf_depth_is_relative_to_each_tree() {
        let shallow = vec![segment_with_depth((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0, 0)];
        let deep = vec![
            segment_with_depth((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0, 0),
            segment_with_depth((0.0, 1.0, 0.0), (0.0, 2.0, 0.0), 1.0, 4),
        ];

        assert_eq!(min_leaf_depth(&shallow, LeafPlacement { crown_levels: 2 }), 0);
        assert_eq!(min_leaf_depth(&deep, LeafPlacement { crown_levels: 2 }), 3);
    }

    #[test]
    fn thick_branch_tip_produces_mostly_visible_leaf_shell() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 2.0, 0.0), 4.0)];
        let wood = voxelize_with_shape(&segments, CrossSectionShape::Cube);
        let wood_positions = voxel_positions(&wood);
        let voxels = add_leaves(&wood, &segments, LeafPlacement::ALL);

        let leaf_positions: Vec<_> = voxels
            .iter()
            .filter(|(_, block_type)| *block_type == BlockType::Leaf)
            .map(|(position, _)| *position)
            .collect();

        assert!(
            !leaf_positions.is_empty(),
            "thick branch tip should produce leaf voxels"
        );

        let visible_leaves = leaf_positions
            .iter()
            .filter(|position| !wood_positions.contains(position))
            .count();

        assert!(
            visible_leaves * 2 > leaf_positions.len(),
            "majority of leaf voxels must sit outside wood (got {visible_leaves}/{} visible)",
            leaf_positions.len()
        );
        assert!(
            visible_leaves >= 80,
            "thick branch should form a substantial leaf shell, not 1-2 voxels"
        );
        assert_eq!(leaf_cluster_radius(4.0), 5, "thickness 4 -> branch r=2, leaf r=5");
    }

    #[test]
    fn neighboring_tip_clusters_overlap_and_deduplicate() {
        let segments = vec![
            segment_with_depth((0.0, 0.0, 0.0), (0.0, 2.0, 0.0), 1.0, 0),
            segment_with_depth((0.0, 2.0, 0.0), (2.0, 4.0, 0.0), 1.0, 2),
            segment_with_depth((0.0, 2.0, 0.0), (-2.0, 4.0, 0.0), 1.0, 2),
        ];
        let wood = voxelize(&segments);
        let merged = add_leaves(&wood, &segments, LeafPlacement::ALL);
        let single_left = add_leaves(
            &wood,
            &[segment_with_depth((0.0, 2.0, 0.0), (-2.0, 4.0, 0.0), 1.0, 2)],
            LeafPlacement::ALL,
        );

        let radius = leaf_cluster_radius(1.0);
        assert_eq!(radius, 3);

        assert!(
            merged.iter().any(|(position, block_type)| {
                *position == IVec3::new(0, 4, 0) && *block_type == BlockType::Leaf
            }),
            "overlapping clusters should fill the crown gap between neighboring tips"
        );
        assert!(
            leaf_count(&merged) < leaf_count(&single_left) * 2,
            "shared overlap region must deduplicate leaf voxels"
        );
        assert!(
            leaf_count(&merged) > leaf_count(&single_left),
            "merged crown should be larger than a single tip cluster"
        );
    }

    #[test]
    fn single_branch_tip_gets_leaf_cluster_around_end() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 2.0, 0.0), 1.0)];
        let wood = voxelize(&segments);
        let voxels = add_leaves(&wood, &segments, LeafPlacement::ALL);

        assert!(
            voxels.iter().any(|(position, block_type)| {
                *position == IVec3::new(0, 3, 0) && *block_type == BlockType::Leaf
            })
        );
        assert!(
            voxels.iter().any(|(position, block_type)| {
                *position == IVec3::new(1, 2, 0) && *block_type == BlockType::Leaf
            })
        );
        assert_eq!(
            voxels
                .iter()
                .filter(|(_, block_type)| *block_type == BlockType::Wood)
                .count(),
            3
        );
    }

    #[test]
    fn junction_is_not_treated_as_branch_tip() {
        let segments = vec![
            segment((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0),
            segment((0.0, 1.0, 0.0), (0.0, 2.0, 0.0), 1.0),
        ];
        let wood = voxelize(&segments);
        let voxels = add_leaves(&wood, &segments, LeafPlacement::ALL);

        assert!(
            !voxels
                .iter()
                .any(|(position, block_type)| {
                    *position == IVec3::new(0, 1, 0) && *block_type == BlockType::Leaf
                }),
            "junction at y=1 must not receive leaves"
        );
        assert!(
            voxels.iter().any(|(position, block_type)| {
                *position == IVec3::new(0, 3, 0) && *block_type == BlockType::Leaf
            })
        );
    }

    #[test]
    fn wood_takes_priority_when_leaf_cluster_overlaps_existing_wood() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0)];
        let wood = voxelize(&segments);
        let voxels = add_leaves(&wood, &segments, LeafPlacement::ALL);

        let tip_entry = voxels
            .iter()
            .find(|(position, _)| *position == IVec3::new(0, 1, 0))
            .expect("tip voxel should exist");

        assert_eq!(tip_entry.1, BlockType::Wood);
        assert_eq!(
            voxels
                .iter()
                .filter(|(position, _)| *position == IVec3::new(0, 1, 0))
                .count(),
            1,
            "tip position must not be duplicated as leaf"
        );
    }

    #[test]
    fn add_leaves_without_wood_places_cluster_at_tip() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0)];
        let voxels = add_leaves(&[], &segments, LeafPlacement::ALL);
        let radius = leaf_cluster_radius(1.0);

        assert_eq!(radius, 3);
        assert_eq!(leaf_count(&voxels), sphere_voxel_count(radius));
        assert!(
            voxels.iter().any(|(position, block_type)| {
                *position == IVec3::new(0, 4, 0) && *block_type == BlockType::Leaf
            })
        );
        assert!(
            voxels.iter().any(|(position, block_type)| {
                *position == IVec3::new(3, 1, 0) && *block_type == BlockType::Leaf
            })
        );
    }

    #[test]
    fn thick_axis_aligned_segment_applies_sphere_at_every_waypoint() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 4.0, 0.0), 4.0)];
        let voxels = voxelize_with_shape(&segments, CrossSectionShape::Sphere);
        let actual = voxel_positions(&voxels);

        let start = IVec3::ZERO;
        let end = IVec3::new(0, 4, 0);
        let expected = expected_voxels_for_segment(start, end, 4.0, CrossSectionShape::Sphere);

        assert_eq!(voxel_radius(4.0), 2, "thickness 4.0 -> radius 2");
        assert_eq!(
            actual, expected,
            "voxelize must match per-waypoint sphere union"
        );
        assert!(
            actual.len() > walk_line(start, end).len(),
            "thick segment must be more than a bare centerline"
        );

        let radius_sq = 2 * 2;
        for &center_y in &[0, 2, 4] {
            for x in -2..=2 {
                for z in -2..=2 {
                    if x * x + z * z <= radius_sq {
                        assert!(
                            actual.contains(&IVec3::new(x, center_y, z)),
                            "missing cross-section voxel at y={center_y} for offset ({x}, {z})"
                        );
                    }
                }
            }
        }
    }

    #[test]
    fn thick_axis_aligned_segment_applies_cube_at_every_waypoint() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 4.0, 0.0), 4.0)];
        let voxels = voxelize_with_shape(&segments, CrossSectionShape::Cube);
        let actual = voxel_positions(&voxels);

        let start = IVec3::ZERO;
        let end = IVec3::new(0, 4, 0);
        let expected = expected_voxels_for_segment(start, end, 4.0, CrossSectionShape::Cube);

        assert_eq!(
            actual, expected,
            "voxelize must match per-waypoint cube union"
        );
        assert_eq!(actual.len(), 125, "five 5x5 cross-section slices");

        for &center_y in &[0, 2, 4] {
            for x in -2..=2 {
                for z in -2..=2 {
                    assert!(
                        actual.contains(&IVec3::new(x, center_y, z)),
                        "missing cube voxel at y={center_y} for offset ({x}, {z})"
                    );
                }
            }
        }
    }

    #[test]
    fn axis_aligned_segment_with_unit_thickness_produces_centerline_voxels() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 3.0, 0.0), 1.0)];

        assert_eq!(
            voxelize_with_shape(&segments, CrossSectionShape::Sphere),
            wood_at(&[(0, 0, 0), (0, 1, 0), (0, 2, 0), (0, 3, 0)])
        );
        assert_eq!(
            voxelize_with_shape(&segments, CrossSectionShape::Cube),
            wood_at(&[(0, 0, 0), (0, 1, 0), (0, 2, 0), (0, 3, 0)])
        );
    }

    #[test]
    fn meeting_segments_do_not_duplicate_shared_voxel() {
        let segments = vec![
            segment((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0),
            segment((0.0, 1.0, 0.0), (0.0, 2.0, 0.0), 1.0),
        ];

        for shape in [CrossSectionShape::Sphere, CrossSectionShape::Cube] {
            let voxels = voxelize_with_shape(&segments, shape);
            assert_eq!(voxels.len(), 3, "{shape:?}");
            assert_eq!(
                voxels,
                wood_at(&[(0, 0, 0), (0, 1, 0), (0, 2, 0)]),
                "{shape:?}"
            );
        }
    }

    #[test]
    fn zero_length_segment_produces_single_voxel() {
        let segments = vec![segment((2.0, 4.0, 6.0), (2.0, 4.0, 6.0), 1.0)];

        for shape in [CrossSectionShape::Sphere, CrossSectionShape::Cube] {
            let voxels = voxelize_with_shape(&segments, shape);
            assert_eq!(voxels, wood_at(&[(2, 4, 6)]), "{shape:?}");
        }
    }

    #[test]
    fn zero_length_segment_with_thickness_fills_spherical_cross_section() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 2.0)];
        let voxels = voxelize_with_shape(&segments, CrossSectionShape::Sphere);

        assert_eq!(
            voxels,
            wood_at(&[
                (0, 0, 0),
                (1, 0, 0),
                (-1, 0, 0),
                (0, 1, 0),
                (0, -1, 0),
                (0, 0, 1),
                (0, 0, -1),
            ])
        );
    }

    #[test]
    fn zero_length_segment_with_thickness_fills_cube_cross_section() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 2.0)];
        let voxels = voxelize_with_shape(&segments, CrossSectionShape::Cube);

        let mut expected = Vec::new();
        for x in -1..=1 {
            for z in -1..=1 {
                expected.push((x, 0, z));
            }
        }

        assert_eq!(voxels, wood_at(&expected));
    }

    #[test]
    fn empty_segment_list_produces_no_voxels() {
        assert!(voxelize_with_shape(&[], CrossSectionShape::Sphere).is_empty());
        assert!(voxelize_with_shape(&[], CrossSectionShape::Cube).is_empty());
    }

    #[test]
    fn preview_tree_pipeline_matches_visualize_example() {
        use std::collections::HashMap;

        use crate::{interpret, LSystemGrammar, TurtleParams};

        let grammar = LSystemGrammar::new(
            "F",
            HashMap::from([('F', "F[+F]F[-F]F".to_string())]),
        );
        let segments = interpret(
            &grammar.expand(4),
            &TurtleParams {
                step_length: 1.0,
                angle_degrees: 25.0,
                base_thickness: 2.0,
                taper_ratio: 0.72,
            },
        );

        for shape in [CrossSectionShape::Sphere, CrossSectionShape::Cube] {
            let voxels = voxelize_with_shape(&segments, shape);
            assert!(voxels.len() > 100, "{shape:?}");
            assert!(voxels.iter().all(|(_, block_type)| *block_type == BlockType::Wood));
        }
    }

    #[test]
    fn diagonal_segment_walks_through_expected_voxels() {
        let segments = vec![segment((0.0, 0.0, 0.0), (2.0, 2.0, 0.0), 1.0)];

        for shape in [CrossSectionShape::Sphere, CrossSectionShape::Cube] {
            let voxels = voxelize_with_shape(&segments, shape);
            assert_eq!(
                voxels,
                wood_at(&[(0, 0, 0), (1, 1, 0), (2, 2, 0)]),
                "{shape:?}"
            );
        }
    }
}
