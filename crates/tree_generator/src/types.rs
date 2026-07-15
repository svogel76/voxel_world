pub use glam::Vec3;

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub start: Vec3,
    pub end: Vec3,
    pub thickness: f32,
    pub depth: u32,
}
