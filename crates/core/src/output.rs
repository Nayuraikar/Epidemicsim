//! Output: per-step Parquet time series and JSON summary (see docs/SCHEMA.md).
use crate::agents::state;
use crate::backend::StepStats;
use crate::params::Config;
use crate::world::STEPS_PER_DAY;
use arrow_array::{ArrayRef, Float64Array, RecordBatch, UInt32Array};
use arrow_schema::{DataType, Field, Schema};
use parquet::arrow::ArrowWriter;
use serde::Serialize;
use std::{error::Error, fs::File, path::Path, sync::Arc};

/// FNV-1a 64 over integer outputs only (platform-independent).
pub fn hash_stats(stats: &[StepStats]) -> String {
    let mut h: u64 = 0xcbf2_9ce4_8422_2325;
    let mut feed = |x: u32| {
        for b in x.to_le_bytes() {
            h ^= u64::from(b);
            h = h.wrapping_mul(0x0000_0100_0000_01b3);
        }
    };
    for s in stats {
        feed(s.step);
        s.counts.iter().for_each(|&c| feed(c));
        feed(s.new_infections);
    }
    format!("{h:016x}")
}

pub fn write_parquet(path: &Path, stats: &[StepStats]) -> Result<(), Box<dyn Error>> {
    let u32col = |f: &dyn Fn(&StepStats) -> u32| -> ArrayRef {
        Arc::new(UInt32Array::from_iter_values(stats.iter().map(f)))
    };
    let mut fields = vec![Field::new("step", DataType::UInt32, false)];
    let mut cols: Vec<ArrayRef> = vec![u32col(&|s| s.step)];
    fields.push(Field::new("day", DataType::UInt32, false));
    cols.push(u32col(&|s| s.day));
    for (k, name) in state::NAMES.iter().enumerate() {
        fields.push(Field::new(*name, DataType::UInt32, false));
        cols.push(u32col(&|s| s.counts[k]));
    }
    fields.push(Field::new("new_infections", DataType::UInt32, false));
    cols.push(u32col(&|s| s.new_infections));
    fields.push(Field::new("hospital_load", DataType::UInt32, false));
    cols.push(u32col(&|s| s.hospital_load));
    fields.push(Field::new("r_t", DataType::Float64, false));
    cols.push(Arc::new(Float64Array::from_iter_values(
        stats.iter().map(|s| s.r_t),
    )));

    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema.clone(), cols)?;
    let mut w = ArrowWriter::try_new(File::create(path)?, schema, None)?;
    w.write(&batch)?;
    w.close()?;
    Ok(())
}

/// Write `step`, `day` (u32) followed by named f64 columns, one row per step.
pub fn write_f64_columns(
    path: &Path,
    steps_per_day: u32,
    cols: &[(String, Vec<f64>)],
) -> Result<(), Box<dyn Error>> {
    let n = cols.first().map_or(0, |c| c.1.len());
    let steps: Vec<u32> = (0..n as u32).collect();
    let mut fields = vec![
        Field::new("step", DataType::UInt32, false),
        Field::new("day", DataType::UInt32, false),
    ];
    let mut arrays: Vec<ArrayRef> = vec![
        Arc::new(UInt32Array::from(steps.clone())),
        Arc::new(UInt32Array::from_iter_values(
            steps.iter().map(|s| s / steps_per_day),
        )),
    ];
    for (name, v) in cols {
        fields.push(Field::new(name, DataType::Float64, false));
        arrays.push(Arc::new(Float64Array::from(v.clone())));
    }
    let schema = Arc::new(Schema::new(fields));
    let batch = RecordBatch::try_new(schema.clone(), arrays)?;
    let mut w = ArrowWriter::try_new(File::create(path)?, schema, None)?;
    w.write(&batch)?;
    w.close()?;
    Ok(())
}

#[derive(Debug, Serialize)]
pub struct FinalCounts {
    #[serde(rename = "S")]
    pub s: u32,
    #[serde(rename = "E")]
    pub e: u32,
    #[serde(rename = "A")]
    pub a: u32,
    #[serde(rename = "I")]
    pub i: u32,
    #[serde(rename = "H")]
    pub h: u32,
    #[serde(rename = "R")]
    pub r: u32,
    #[serde(rename = "D")]
    pub d: u32,
}

#[derive(Debug, Serialize)]
pub struct Summary {
    pub backend: String,
    pub agents: usize,
    pub days: u32,
    pub seed: u64,
    pub steps: usize,
    /// Peak of A + I.
    pub peak_infectious: u32,
    pub peak_infectious_day: u32,
    pub peak_hospital_load: u32,
    pub peak_hospital_day: u32,
    /// Fraction of agents no longer susceptible at the end.
    pub attack_rate: f64,
    pub total_deaths: u32,
    pub max_r_t: f64,
    pub final_counts: FinalCounts,
    /// FNV-1a 64 of per-step state counts and new infections.
    pub hash: String,
}

impl Summary {
    pub fn from_stats(cfg: &Config, backend: &str, stats: &[StepStats]) -> Self {
        let (mut pi, mut pi_step, mut ph, mut ph_step) = (0, 0, 0, 0);
        for s in stats {
            let inf = s.counts[state::A as usize] + s.counts[state::I as usize];
            if inf > pi {
                (pi, pi_step) = (inf, s.step);
            }
            if s.hospital_load > ph {
                (ph, ph_step) = (s.hospital_load, s.step);
            }
        }
        let c = stats.last().map(|s| s.counts).unwrap_or([0; state::COUNT]);
        let total: u32 = c.iter().sum();
        Summary {
            backend: backend.to_string(),
            agents: cfg.agents,
            days: cfg.days,
            seed: cfg.seed,
            steps: stats.len(),
            peak_infectious: pi,
            peak_infectious_day: pi_step / STEPS_PER_DAY,
            peak_hospital_load: ph,
            peak_hospital_day: ph_step / STEPS_PER_DAY,
            attack_rate: if total > 0 {
                1.0 - f64::from(c[0]) / f64::from(total)
            } else {
                0.0
            },
            total_deaths: c[state::D as usize],
            max_r_t: stats.iter().map(|s| s.r_t).fold(0.0, f64::max),
            final_counts: FinalCounts {
                s: c[0],
                e: c[1],
                a: c[2],
                i: c[3],
                h: c[4],
                r: c[5],
                d: c[6],
            },
            hash: hash_stats(stats),
        }
    }

    pub fn write_json(&self, path: &Path) -> Result<(), Box<dyn Error>> {
        std::fs::write(path, serde_json::to_string_pretty(self)? + "\n")?;
        Ok(())
    }
}
