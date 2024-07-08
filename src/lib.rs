pub mod sql_types;
pub mod types;

pub mod auto;
pub mod ewkb;
pub mod spatialite;
pub mod geo_json;

pub mod error;
pub mod functions;

const DEFAULT_SRID: u32 = 4326;