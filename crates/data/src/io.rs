//! Reading/writing `cities/<name>/{locations,agents}.parquet` and `meta.json`.
use arrow_array::{ArrayRef, Float32Array, Float64Array, RecordBatch, UInt32Array, UInt8Array};
use arrow_schema::{DataType, Field, Schema};
use epidemicsim_core::{agents::Agents, world::Layout, Config, World};
use parquet::arrow::{arrow_reader::ParquetRecordBatchReaderBuilder, ArrowWriter};
use serde::{Deserialize, Serialize};
use std::{error::Error, fs::File, path::Path, sync::Arc};

pub const KIND_HOME: u8 = 0;
pub const KIND_WORK: u8 = 1;
pub const KIND_MARKET: u8 = 2;
pub const KIND_HOSPITAL: u8 = 3;
/// Subtype for KIND_WORK rows.
pub const SUB_SCHOOL: u8 = 1;
pub const SUB_COLLEGE: u8 = 2;
pub const SUB_OFFICE: u8 = 3;

#[derive(Clone, Debug, Default)]
pub struct Locations {
    pub kind: Vec<u8>,
    pub subtype: Vec<u8>,
    pub lon: Vec<f64>,
    pub lat: Vec<f64>,
    /// Homes: residents; work: assigned agents; hospital: beds; market: 0.
    pub size: Vec<f32>,
    /// 1 if not from OSM (topped up).
    pub synthetic: Vec<u8>,
}

#[derive(Clone, Debug, Default)]
pub struct AgentRows {
    pub age_group: Vec<u8>,
    pub home_loc: Vec<u32>,
    pub work_loc: Vec<u32>,
}

#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct Counts {
    pub homes: u32,
    pub schools: u32,
    pub colleges: u32,
    pub offices: u32,
    pub markets: u32,
    pub hospitals: u32,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Meta {
    pub name: String,
    pub source_url: String,
    pub bbox: [f64; 4],
    pub seed: u64,
    pub target_agents: usize,
    pub agents: usize,
    pub pop_weight: String,
    pub hospital_beds_per_1000: f64,
    pub total_beds: f64,
    pub counts: Counts,
    /// Locations created synthetically because OSM had too few.
    pub synthetic: Counts,
}

fn write(path: &Path, fields: Vec<(&str, DataType, ArrayRef)>) -> Result<(), Box<dyn Error>> {
    let schema = Arc::new(Schema::new(
        fields
            .iter()
            .map(|(n, t, _)| Field::new(*n, t.clone(), false))
            .collect::<Vec<_>>(),
    ));
    let batch = RecordBatch::try_new(schema.clone(), fields.into_iter().map(|f| f.2).collect())?;
    let mut w = ArrowWriter::try_new(File::create(path)?, schema, None)?;
    w.write(&batch)?;
    w.close()?;
    Ok(())
}

pub fn write_city(
    dir: &Path,
    loc: &Locations,
    ag: &AgentRows,
    meta: &Meta,
) -> Result<(), Box<dyn Error>> {
    std::fs::create_dir_all(dir)?;
    let n = loc.kind.len() as u32;
    write(
        &dir.join("locations.parquet"),
        vec![
            (
                "loc_id",
                DataType::UInt32,
                Arc::new(UInt32Array::from_iter_values(0..n)),
            ),
            (
                "kind",
                DataType::UInt8,
                Arc::new(UInt8Array::from(loc.kind.clone())),
            ),
            (
                "subtype",
                DataType::UInt8,
                Arc::new(UInt8Array::from(loc.subtype.clone())),
            ),
            (
                "lon",
                DataType::Float64,
                Arc::new(Float64Array::from(loc.lon.clone())),
            ),
            (
                "lat",
                DataType::Float64,
                Arc::new(Float64Array::from(loc.lat.clone())),
            ),
            (
                "size",
                DataType::Float32,
                Arc::new(Float32Array::from(loc.size.clone())),
            ),
            (
                "synthetic",
                DataType::UInt8,
                Arc::new(UInt8Array::from(loc.synthetic.clone())),
            ),
        ],
    )?;
    let m = ag.age_group.len() as u32;
    write(
        &dir.join("agents.parquet"),
        vec![
            (
                "id",
                DataType::UInt32,
                Arc::new(UInt32Array::from_iter_values(0..m)),
            ),
            (
                "age_group",
                DataType::UInt8,
                Arc::new(UInt8Array::from(ag.age_group.clone())),
            ),
            (
                "home_loc",
                DataType::UInt32,
                Arc::new(UInt32Array::from(ag.home_loc.clone())),
            ),
            (
                "work_loc",
                DataType::UInt32,
                Arc::new(UInt32Array::from(ag.work_loc.clone())),
            ),
        ],
    )?;
    std::fs::write(dir.join("meta.json"), serde_json::to_string_pretty(meta)?)?;
    Ok(())
}

fn read_batches(path: &Path) -> Result<Vec<RecordBatch>, Box<dyn Error>> {
    let r = ParquetRecordBatchReaderBuilder::try_new(File::open(path)?)?.build()?;
    Ok(r.collect::<Result<_, _>>()?)
}

fn col<'a, T: 'static>(b: &'a RecordBatch, name: &str) -> Result<&'a T, Box<dyn Error>> {
    b.column_by_name(name)
        .and_then(|c| c.as_any().downcast_ref::<T>())
        .ok_or_else(|| format!("missing or mistyped column {name}").into())
}

pub fn read_meta(dir: &Path) -> Result<Meta, Box<dyn Error>> {
    Ok(serde_json::from_str(&std::fs::read_to_string(
        dir.join("meta.json"),
    )?)?)
}

/// Build a `World` from a city folder; disease/vaccine params and seed come from `cfg`.
pub fn load_world(dir: &Path, cfg: &Config) -> Result<World, Box<dyn Error>> {
    let meta = read_meta(dir)
        .map_err(|e| format!("{}: {e} (run `epidemicsim pipeline` first)", dir.display()))?;
    let c = &meta.counts;
    let layout = Layout::new(
        c.homes,
        c.schools + c.colleges + c.offices,
        c.markets,
        c.hospitals,
    )
    .with_colleges(c.schools, c.colleges);
    let total = layout.total() as u32;
    let mut agents = Agents::with_capacity(meta.agents);
    for b in read_batches(&dir.join("agents.parquet"))? {
        let (age, home, work) = (
            col::<UInt8Array>(&b, "age_group")?,
            col::<UInt32Array>(&b, "home_loc")?,
            col::<UInt32Array>(&b, "work_loc")?,
        );
        for i in 0..b.num_rows() {
            if home.value(i) >= total || work.value(i) >= total {
                return Err("agent references unknown location".into());
            }
            agents.push(age.value(i), home.value(i), work.value(i), 0, 0);
        }
    }
    if agents.len() != meta.agents {
        return Err(format!(
            "agents.parquet has {} rows, meta says {}",
            agents.len(),
            meta.agents
        )
        .into());
    }
    let mut world = World::new(
        cfg.seed,
        agents,
        layout,
        cfg.disease.clone(),
        cfg.vaccine.clone(),
    );
    world.seed_infections(cfg.disease.initial_infected_frac);
    Ok(world)
}

/// Read `locations.parquet` (coordinates and kinds) from a city folder.
pub fn read_locations(dir: &Path) -> Result<Locations, Box<dyn Error>> {
    let mut l = Locations::default();
    for b in read_batches(&dir.join("locations.parquet"))? {
        let (kind, sub) = (
            col::<UInt8Array>(&b, "kind")?,
            col::<UInt8Array>(&b, "subtype")?,
        );
        let (lon, lat) = (
            col::<Float64Array>(&b, "lon")?,
            col::<Float64Array>(&b, "lat")?,
        );
        let size = col::<Float32Array>(&b, "size")?;
        let syn = col::<UInt8Array>(&b, "synthetic")?;
        for i in 0..b.num_rows() {
            l.kind.push(kind.value(i));
            l.subtype.push(sub.value(i));
            l.lon.push(lon.value(i));
            l.lat.push(lat.value(i));
            l.size.push(size.value(i));
            l.synthetic.push(syn.value(i));
        }
    }
    Ok(l)
}

/// Read a Parquet file of numeric columns (any integer/float type) as `(name, values)` pairs.
pub type NumericColumns = Vec<(String, Vec<f64>)>;

pub fn read_numeric_columns(path: &Path) -> Result<NumericColumns, Box<dyn Error>> {
    use arrow_array::cast::AsArray;
    use arrow_array::types::{Float64Type, UInt32Type};
    let mut out: Vec<(String, Vec<f64>)> = Vec::new();
    for b in read_batches(path)? {
        if out.is_empty() {
            out = b
                .schema()
                .fields()
                .iter()
                .map(|f| (f.name().clone(), vec![]))
                .collect();
        }
        for (k, c) in b.columns().iter().enumerate() {
            match c.data_type() {
                DataType::Float64 => out[k]
                    .1
                    .extend(c.as_primitive::<Float64Type>().values().iter().copied()),
                DataType::UInt32 => out[k].1.extend(
                    c.as_primitive::<UInt32Type>()
                        .values()
                        .iter()
                        .map(|&v| f64::from(v)),
                ),
                t => return Err(format!("unsupported column type {t}").into()),
            }
        }
    }
    Ok(out)
}
