use std::collections::HashSet;

use glam::IVec3;

use crate::types::{BlockType, Segment};

/// Convert line segments into unique voxel positions.
pub fn voxelize(segments: &[Segment]) -> Vec<(IVec3, BlockType)> {
    let mut voxels = HashSet::new();

    for segment in segments {
        let start = to_voxel(segment.start);
        let end = to_voxel(segment.end);
        let radius = voxel_radius(segment.thickness);

        for center in walk_line(start, end) {
            fill_cross_section(center, radius, &mut voxels);
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
/// spherical cross-section around each line voxel.
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

fn fill_cross_section(center: IVec3, radius: i32, voxels: &mut HashSet<IVec3>) {
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

    #[test]
    fn axis_aligned_segment_with_unit_thickness_produces_centerline_voxels() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 3.0, 0.0), 1.0)];
        let voxels = voxelize(&segments);

        assert_eq!(
            voxels,
            wood_at(&[(0, 0, 0), (0, 1, 0), (0, 2, 0), (0, 3, 0)])
        );
    }

    #[test]
    fn meeting_segments_do_not_duplicate_shared_voxel() {
        let segments = vec![
            segment((0.0, 0.0, 0.0), (0.0, 1.0, 0.0), 1.0),
            segment((0.0, 1.0, 0.0), (0.0, 2.0, 0.0), 1.0),
        ];
        let voxels = voxelize(&segments);

        assert_eq!(voxels.len(), 3);
        assert_eq!(
            voxels,
            wood_at(&[(0, 0, 0), (0, 1, 0), (0, 2, 0)])
        );
    }

    #[test]
    fn zero_length_segment_produces_single_voxel() {
        let segments = vec![segment((2.0, 4.0, 6.0), (2.0, 4.0, 6.0), 1.0)];
        let voxels = voxelize(&segments);

        assert_eq!(voxels, wood_at(&[(2, 4, 6)]));
    }

    #[test]
    fn zero_length_segment_with_thickness_fills_spherical_cross_section() {
        let segments = vec![segment((0.0, 0.0, 0.0), (0.0, 0.0, 0.0), 2.0)];
        let voxels = voxelize(&segments);

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
    fn empty_segment_list_produces_no_voxels() {
        let voxels = voxelize(&[]);
        assert!(voxels.is_empty());
    }

    #[test]
    fn diagonal_segment_walks_through_expected_voxels() {
        let segments = vec![segment((0.0, 0.0, 0.0), (2.0, 2.0, 0.0), 1.0)];
        let voxels = voxelize(&segments);

        assert_eq!(
            voxels,
            wood_at(&[(0, 0, 0), (1, 1, 0), (2, 2, 0)])
        );
    }
}
