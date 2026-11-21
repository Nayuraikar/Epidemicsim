//! City config (`cities/<name>.toml`).
use crate::geo::Bbox;
use serde::Deserialize;
use std::{error::Error, path::Path};

pub const MAX_AGENTS: usize = 1_000_000;

#[derive(Clone, Debug, Deserialize)]
pub struct CityConfig {
    pub name: String,
    pub geofabrik_url: String,
    /// [min_lon, min_lat, max_lon, max_lat]
    pub bbox: [f64; 4],
    pub target_agents: usize,
    pub hospital_beds_per_1000: f64,
    /// Optional WorldPop GeoTIFF (clipped to the city). Default: data_cache/<name>_worldpop.tif if present.
    pub worldpop_tif: Option<String>,
    pub seed: Option<u64>,
}

impl CityConfig {
    pub fn from_file(path: &Path) -> Result<Self, Box<dyn Error>> {
        let c: Self = toml::from_str(&std::fs::read_to_string(path)?)
            .map_err(|e| format!("{}: {e}", path.display()))?;
        c.validate()?;
        Ok(c)
    }

    pub fn validate(&self) -> Result<(), Box<dyn Error>> {
        let [a, b, c, d] = self.bbox;
        if !(a < c && b < d) {
            return Err("bbox must be [min_lon, min_lat, max_lon, max_lat]".into());
        }
        if self.target_agents == 0 || self.target_agents > MAX_AGENTS {
            return Err(format!("target_agents must be in 1..={MAX_AGENTS}").into());
        }
        Ok(())
    }

    pub fn bbox(&self) -> Bbox {
        let [min_lon, min_lat, max_lon, max_lat] = self.bbox;
        Bbox {
            min_lon,
            min_lat,
            max_lon,
            max_lat,
        }
    }
}
