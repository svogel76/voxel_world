pub mod generate;
pub mod instance;
pub mod placement;

pub use generate::{generate, GrassParams, VariantWeights};
pub use instance::{build_instances, GrassInstance, GrassVariant, SCALE_MAX, SCALE_MIN};
pub use placement::{instance_count, place_positions, Area};
