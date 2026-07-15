use std::collections::HashSet;

use glam::IVec3;

use crate::types::{BlockType, Segment};

/// Cross-section used when thickening line segments into voxels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
pub enum CrossSectionShape {
    #[default]
    Sphere,
    Cube,
}

/// Convert line segments into unique voxel positions.
pub fn voxelize(segments: &[Segment], shape: CrossSectionShape) -> Vec<(IVec3, BlockType)> {
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
        Segment {
            start: Vec3::new(start.0, start.1, start.2),
            end: Vec3::new(end.0, end.1, end.2),
            thickness,
            depth: 0,
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

    #[test]
    fn thick_axis_aligned_segment_applies_sphere_at_every_waypoint() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 4.0, 0.0), 4.0)];
        let voxels = voxelize(&segments, CrossSectionShape::Sphere);
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
        let voxels = voxelize(&segments, CrossSectionShape::Cube);
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
            voxelize(&segments, CrossSectionShape::Sphere),
            wood_at(&[(0, 0, 0), (0, 1, 0), (0, 2, 0), (0, 3, 0)])
        );
        assert_eq!(
            voxelize(&segments, CrossSectionShape::Cube),
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
            let voxels = voxelize(&segments, shape);
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
            let voxels = voxelize(&segments, shape);
            assert_eq!(voxels, wood_at(&[(2, 4, 6)]), "{shape:?}");
        }
    }

    #[test]
    fn zero_length_segment_with_thickness_fills_spherical_cross_section() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 2.0)];
        let voxels = voxelize(&segments, CrossSectionShape::Sphere);

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
        let voxels = voxelize(&segments, CrossSectionShape::Cube);

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
        assert!(voxelize(&[], CrossSectionShape::Sphere).is_empty());
        assert!(voxelize(&[], CrossSectionShape::Cube).is_empty());
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
            let voxels = voxelize(&segments, shape);
            assert!(voxels.len() > 100, "{shape:?}");
            assert!(voxels.iter().all(|(_, block_type)| *block_type == BlockType::Wood));
        }
    }

    #[test]
    fn diagonal_segment_walks_through_expected_voxels() {
        let segments = vec![segment((0.0, 0.0, 0.0), (2.0, 2.0, 0.0), 1.0)];

        for shape in [CrossSectionShape::Sphere, CrossSectionShape::Cube] {
            let voxels = voxelize(&segments, shape);
            assert_eq!(
                voxels,
                wood_at(&[(0, 0, 0), (1, 1, 0), (2, 2, 0)]),
                "{shape:?}"
            );
        }
    }
}
