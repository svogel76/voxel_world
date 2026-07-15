use crate::types::{Segment, Vec3};

/// Parameters for turtle interpretation. All angles are in degrees.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct TurtleParams {
    pub step_length: f32,
    pub angle_degrees: f32,
    pub base_thickness: f32,
    /// Per-branch taper factor. Final thickness = `base_thickness * taper_ratio^depth`.
    pub taper_ratio: f32,
}

impl Default for TurtleParams {
    fn default() -> Self {
        Self {
            step_length: 1.0,
            angle_degrees: 90.0,
            base_thickness: 1.0,
            taper_ratio: 1.0,
        }
    }
}

/// Interpret an expanded L-system string as a list of 3D line segments.
///
/// Coordinate convention: the turtle starts at the origin facing **+Y** (up).
/// - `F` draws a segment along the current heading and advances.
/// - `+` / `-` yaw left / right around the turtle's local up axis.
/// - `&` / `^` pitch down / up around the turtle's local right axis.
/// - `[` pushes position, heading, and depth; `]` restores the saved state.
/// - Any other symbol is ignored.
pub fn interpret(input: &str, params: &TurtleParams) -> Vec<Segment> {
    let mut segments = Vec::new();
    let mut state = TurtleState::initial();
    let mut stack = Vec::new();
    let angle_rad = params.angle_degrees.to_radians();

    for ch in input.chars() {
        match ch {
            'F' => {
                let end = state.position + state.direction * params.step_length;
                segments.push(Segment {
                    start: state.position,
                    end,
                    thickness: segment_thickness(params, state.depth),
                    depth: state.depth,
                });
                state.position = end;
            }
            '+' => state.direction = rotate_yaw(state.direction, angle_rad),
            '-' => state.direction = rotate_yaw(state.direction, -angle_rad),
            '&' => state.direction = rotate_pitch(state.direction, angle_rad),
            '^' => state.direction = rotate_pitch(state.direction, -angle_rad),
            '[' => {
                stack.push(state);
                state.depth += 1;
            }
            ']' => {
                if let Some(saved) = stack.pop() {
                    state = saved;
                }
            }
            _ => {}
        }
    }

    segments
}

#[derive(Debug, Clone, Copy, PartialEq)]
struct TurtleState {
    position: Vec3,
    direction: Vec3,
    depth: u32,
}

impl TurtleState {
    fn initial() -> Self {
        Self {
            position: Vec3::ZERO,
            direction: Vec3::Y,
            depth: 0,
        }
    }
}

fn segment_thickness(params: &TurtleParams, depth: u32) -> f32 {
    params.base_thickness * params.taper_ratio.powi(depth as i32)
}

fn rotate_yaw(direction: Vec3, angle_rad: f32) -> Vec3 {
    let axis = local_up(direction);
    normalize_direction(direction.rotate_axis(axis, angle_rad))
}

fn rotate_pitch(direction: Vec3, angle_rad: f32) -> Vec3 {
    let axis = local_right(direction);
    normalize_direction(direction.rotate_axis(axis, angle_rad))
}

fn normalize_direction(direction: Vec3) -> Vec3 {
    let normalized = direction.normalize_or_zero();
    if normalized.length_squared() <= f32::EPSILON {
        Vec3::Y
    } else {
        normalized
    }
}

fn local_up(direction: Vec3) -> Vec3 {
    normalize_or_fallback(Vec3::Y.cross(direction), Vec3::new(0.0, 0.0, -1.0))
}

fn local_right(direction: Vec3) -> Vec3 {
    normalize_or_fallback(local_up(direction).cross(direction), Vec3::X)
}

fn normalize_or_fallback(vector: Vec3, fallback: Vec3) -> Vec3 {
    if vector.length_squared() <= f32::EPSILON {
        fallback
    } else {
        vector.normalize()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn default_params() -> TurtleParams {
        TurtleParams::default()
    }

    #[test]
    fn single_forward_draws_segment_along_positive_y() {
        let segments = interpret("F", &default_params());

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start, Vec3::ZERO);
        assert_eq!(segments[0].end, Vec3::new(0.0, 1.0, 0.0));
        assert_eq!(segments[0].depth, 0);
        assert_eq!(segments[0].thickness, 1.0);
    }

    fn assert_vec3_approx(actual: Vec3, expected: Vec3) {
        const EPSILON: f32 = 1e-5;
        assert!((actual.x - expected.x).abs() < EPSILON, "x: {actual:?} != {expected:?}");
        assert!((actual.y - expected.y).abs() < EPSILON, "y: {actual:?} != {expected:?}");
        assert!((actual.z - expected.z).abs() < EPSILON, "z: {actual:?} != {expected:?}");
    }

    fn assert_segment_approx(actual: &Segment, expected: &Segment) {
        assert_vec3_approx(actual.start, expected.start);
        assert_vec3_approx(actual.end, expected.end);
        assert_eq!(actual.thickness, expected.thickness);
        assert_eq!(actual.depth, expected.depth);
    }

    #[test]
    fn branch_pattern_produces_trunk_branch_and_continuation() {
        let segments = interpret("F[+F]F", &default_params());

        assert_eq!(segments.len(), 3);
        assert_segment_approx(
            &segments[0],
            &Segment {
                start: Vec3::ZERO,
                end: Vec3::new(0.0, 1.0, 0.0),
                thickness: 1.0,
                depth: 0,
            },
        );
        assert_segment_approx(
            &segments[1],
            &Segment {
                start: Vec3::new(0.0, 1.0, 0.0),
                end: Vec3::new(1.0, 1.0, 0.0),
                thickness: 1.0,
                depth: 1,
            },
        );
        assert_segment_approx(
            &segments[2],
            &Segment {
                start: Vec3::new(0.0, 1.0, 0.0),
                end: Vec3::new(0.0, 2.0, 0.0),
                thickness: 1.0,
                depth: 0,
            },
        );
    }

    #[test]
    fn empty_string_produces_no_segments() {
        let segments = interpret("", &default_params());
        assert!(segments.is_empty());
    }

    #[test]
    fn ignored_symbols_do_not_create_segments() {
        let segments = interpret("+-&^XYZ", &default_params());
        assert!(segments.is_empty());
    }

    #[test]
    fn pop_on_empty_stack_is_ignored_without_panic() {
        let segments = interpret("]F", &default_params());

        assert_eq!(segments.len(), 1);
        assert_eq!(segments[0].start, Vec3::ZERO);
        assert_eq!(segments[0].end, Vec3::new(0.0, 1.0, 0.0));
    }

    #[test]
    fn pitch_symbols_tilt_heading_for_3d_growth() {
        let segments = interpret("F&F", &default_params());

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].end, Vec3::new(0.0, 1.0, 0.0));
        assert_vec3_approx(segments[1].start, Vec3::new(0.0, 1.0, 0.0));
        assert_vec3_approx(segments[1].end, Vec3::new(0.0, 1.0, 1.0));
    }

    #[test]
    fn nested_branches_accumulate_depth() {
        let segments = interpret("F[[F]]", &default_params());

        assert_eq!(segments.len(), 2);
        assert_eq!(segments[0].depth, 0);
        assert_eq!(segments[1].depth, 2);
    }

    #[test]
    fn thickness_tapers_with_branch_depth() {
        let params = TurtleParams {
            base_thickness: 1.0,
            taper_ratio: 0.5,
            ..Default::default()
        };
        let segments = interpret("F[+F]F", &params);

        assert_eq!(segments[0].thickness, 1.0);
        assert_eq!(segments[0].depth, 0);
        assert_eq!(segments[1].thickness, 0.5);
        assert_eq!(segments[1].depth, 1);
        assert_eq!(segments[2].thickness, 1.0);
        assert_eq!(segments[2].depth, 0);
    }

    #[test]
    fn thickness_tapers_compound_with_nested_branches() {
        let params = TurtleParams {
            base_thickness: 1.0,
            taper_ratio: 0.5,
            ..Default::default()
        };
        let segments = interpret("F[[F]]", &params);

        assert_eq!(segments[0].thickness, 1.0);
        assert_eq!(segments[0].depth, 0);
        assert_eq!(segments[1].thickness, 0.25);
        assert_eq!(segments[1].depth, 2);
    }

    #[test]
    fn custom_step_length_and_thickness_are_applied() {
        let params = TurtleParams {
            step_length: 2.0,
            angle_degrees: 90.0,
            base_thickness: 0.5,
            taper_ratio: 1.0,
        };
        let segments = interpret("F", &params);

        assert_eq!(segments[0].end, Vec3::new(0.0, 2.0, 0.0));
        assert_eq!(segments[0].thickness, 0.5);
    }
}
