pub mod biome;
pub mod noise;
pub mod placement;
pub mod terrain;

pub use biome::{
    classify, moisture_at, params_for, Biome, BiomeParams, CLEARING_MAX_MOISTURE,
    MOISTURE_FREQUENCY, ROCKY_MIN_HEIGHT,
};
pub use grass_generator::Area;
pub use placement::{
    min_distance_from_density, poisson_disc_sample, rock_density_multiplier, slope_at,
    MAX_ROCK_DENSITY_MULTIPLIER, SLOPE_EPSILON, SLOPE_WEIGHT,
};
pub use terrain::{ConstantHeight, SimpleNoiseTerrain, TerrainHeightSource};
