//! City data pipeline: OSM .pbf -> locations/agents Parquet + meta.json.
pub mod city;
pub mod download;
pub mod geo;
pub mod io;
pub mod osm;
pub mod pop;
pub mod weight;

pub use city::CityConfig;
pub use io::{load_world, read_meta, Meta};

use epidemicsim_core::params::SynthParams;
use std::{
    error::Error,
    path::{Path, PathBuf},
};

pub struct PipelineOpts {
    /// Directory holding `<name>.toml` files and receiving `<name>/` output folders.
    pub cities_dir: PathBuf,
    pub cache_dir: PathBuf,
    /// Use this .pbf instead of downloading.
    pub pbf: Option<PathBuf>,
    /// Override target agents (clamped by validation to <= 1M).
    pub agents: Option<usize>,
}

/// Build `<cities_dir>/<name>/` from `<cities_dir>/<name>.toml`.
pub fn build_city(name: &str, o: &PipelineOpts) -> Result<PathBuf, Box<dyn Error>> {
    let mut cfg = CityConfig::from_file(&o.cities_dir.join(format!("{name}.toml")))?;
    if let Some(a) = o.agents {
        cfg.target_agents = a;
        cfg.validate()?;
    }
    let pbf = match &o.pbf {
        Some(p) => p.clone(),
        None => download::ensure_pbf(&cfg.geofabrik_url, &o.cache_dir)?,
    };
    build_from_pbf(&cfg, &pbf, &o.cache_dir, &o.cities_dir.join(&cfg.name))
}

pub fn build_from_pbf(
    cfg: &CityConfig,
    pbf: &Path,
    cache_dir: &Path,
    out_dir: &Path,
) -> Result<PathBuf, Box<dyn Error>> {
    let pois = osm::extract(pbf, &cfg.bbox())?;
    eprintln!(
        "{}: homes={} offices={} schools={} colleges={} markets={} hospitals={}",
        cfg.name,
        pois.homes.len(),
        pois.offices.len(),
        pois.schools.len(),
        pois.colleges.len(),
        pois.markets.len(),
        pois.hospitals.len()
    );
    let tif = match &cfg.worldpop_tif {
        Some(p) => PathBuf::from(p),
        None => cache_dir.join(format!("{}_worldpop.tif", cfg.name)),
    };
    let weighter = weight::choose(Some(&tif))?;
    let city = pop::build(cfg, pois, weighter.as_ref(), &SynthParams::default())?;
    io::write_city(out_dir, &city.locations, &city.agents, &city.meta)?;
    Ok(out_dir.to_path_buf())
}
