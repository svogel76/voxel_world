pub mod grammar;
pub mod turtle;
pub mod types;
pub mod voxelize;

pub use grammar::LSystemGrammar;
pub use turtle::{interpret, TurtleParams};
pub use types::{BlockType, Segment, Vec3};
pub use voxelize::{
    add_leaves, voxelize, voxelize_with_shape, CrossSectionShape, LeafPlacement,
};

pub use glam::IVec3;

