pub use glam::Vec3;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BlockType {
    Wood,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Segment {
    pub start: Vec3,
    pub end: Vec3,
    pub thickness: f32,
    pub depth: u32,
}

