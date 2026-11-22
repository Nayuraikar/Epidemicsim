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

/// Empirical: k=10 on Mumbai gives attack 41.3% / peak day 84 vs 41.7% / day 80 for the full city
/// (exponent 0.5 gave 19%, 0.75 gave 61%). Not validated for other k or cities.
const SUBSAMPLE_BETA_EXP: f64 = 0.6;

/// Keep whole households whose home id hashes into 1 of `k` buckets (about `1/k` of agents) and
/// rescale non-home transmission by `k^0.6` so epidemic size roughly matches the full city.
/// Approximate; used for capped live runs.
pub fn subsample_world(full: &epidemicsim_core::World, k: u32) -> epidemicsim_core::World {
    use epidemicsim_core::agents::Agents;
    if k <= 1 {
        return full.clone();
    }
    let keep = |home: u32| (home.wrapping_mul(2_654_435_761) >> 7).is_multiple_of(k);
    let a = &full.agents;
    let mut agents = Agents::with_capacity(a.len() / k as usize + 16);
    for i in 0..a.len() {
        if keep(a.home_loc[i]) {
            agents.push(a.age_group[i], a.home_loc[i], a.work_loc[i], 0, 0);
        }
    }
    let mut disease = full.disease.clone();
    for kind in 1..disease.beta.len() {
        disease.beta[kind] *= f64::from(k).powf(SUBSAMPLE_BETA_EXP);
    }
    epidemicsim_core::World::new(
        full.seed,
        agents,
        full.layout.clone(),
        disease,
        full.vaccine.clone(),
    )
}
