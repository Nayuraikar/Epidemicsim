//! Metal timing breakdown: kernel / buffer setup / CPU lambda / readback, per storage mode and
//! threadgroup size. Each configuration is verified against the sequential run when affordable.
use super::{MetalBackend, MetalOpts, Timings};
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::{synth, Backend, Config, SequentialBackend, StepStats};
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize)]
pub struct Row {
    pub agents: usize,
    pub storage: &'static str,
    pub threadgroup: usize,
    pub steps: u32,
    /// Medians over the timed repetitions, seconds for the whole run.
    pub total_s: f64,
    pub buffer_setup_s: f64,
    pub kernel_s: f64,
    pub kernel_cmd1_place_locs_s: f64,
    pub kernel_cmd2_update_stats_s: f64,
    pub cpu_lambda_s: f64,
    pub readback_s: f64,
    /// step wall - kernel - cpu lambda - readback: encode, commit, wait and scheduling.
    pub dispatch_overhead_s: f64,
    pub matches_sequential: Option<bool>,
}

#[derive(Serialize)]
pub struct Breakdown {
    pub device: String,
    /// One-time cost in this process: device lookup + MSL compile + 6 pipeline states.
    pub init_once_s: f64,
    pub warmup: usize,
    pub reps: usize,
    pub days: u32,
    pub rows: Vec<Row>,
}

fn median(mut v: Vec<f64>) -> f64 {
    v.sort_by(f64::total_cmp);
    v[v.len() / 2]
}

fn run(
    world: &epidemicsim_core::World,
    o: &MetalOpts,
    steps: u32,
) -> Result<(f64, Timings, Vec<StepStats>), String> {
    let mut b = MetalBackend::new(o.clone())?;
    let mut w = world.clone();
    let iv = Default::default();
    let t0 = Instant::now();
    let stats: Vec<StepStats> = (0..steps).map(|_| b.step(&mut w, &iv)).collect();
    Ok((t0.elapsed().as_secs_f64(), b.timings().clone(), stats))
}

pub fn breakdown(
    agents: &[usize],
    tgs: &[usize],
    days: u32,
    warmup: usize,
    reps: usize,
    seed: u64,
) -> Result<Breakdown, String> {
    let t0 = Instant::now();
    if !super::available() {
        return Err("no Metal device".into());
    }
    let init = t0.elapsed().as_secs_f64();
    let steps = days * STEPS_PER_DAY;
    let mut rows = Vec::new();
    for &n in agents {
        let world = synth::generate(&Config {
            agents: n,
            seed,
            ..Default::default()
        });
        let reference = (n <= 1_000_000).then(|| {
            let mut w = world.clone();
            let mut b = SequentialBackend::default();
            let iv = Default::default();
            (0..steps).map(|_| b.step(&mut w, &iv)).collect::<Vec<_>>()
        });
        for private in [false, true] {
            for &tg in tgs {
                let o = MetalOpts {
                    threadgroup: tg,
                    private,
                };
                let mut samples: Vec<(f64, Timings)> = Vec::new();
                let mut last = Vec::new();
                for rep in 0..warmup + reps {
                    let (t, tm, st) = run(&world, &o, steps)?;
                    if rep >= warmup {
                        samples.push((t, tm));
                    }
                    last = st;
                }
                let med = |f: &dyn Fn(&Timings) -> f64| {
                    median(samples.iter().map(|(_, t)| f(t)).collect())
                };
                let (kernel, cpu, rb) = (
                    med(&|t| t.kernel_s),
                    med(&|t| t.cpu_lambda_s),
                    med(&|t| t.readback_s),
                );
                let wall = med(&|t| t.step_wall_s);
                rows.push(Row {
                    agents: n,
                    storage: if private { "private" } else { "shared" },
                    threadgroup: tg,
                    steps,
                    total_s: median(samples.iter().map(|s| s.0).collect()),
                    buffer_setup_s: med(&|t| t.setup_s),
                    kernel_s: kernel,
                    kernel_cmd1_place_locs_s: med(&|t| t.cmd1_s),
                    kernel_cmd2_update_stats_s: med(&|t| t.cmd2_s),
                    cpu_lambda_s: cpu,
                    readback_s: rb,
                    dispatch_overhead_s: (wall - kernel - cpu - rb).max(0.0),
                    matches_sequential: reference.as_ref().map(|r| *r == last),
                });
                let r = rows.last().expect("pushed");
                eprintln!(
                    "n={n} {} tg{tg}: total {:.3}s kernel {:.3} setup {:.3} cpu {:.3} readback {:.4} overhead {:.3} ok={:?}",
                    r.storage, r.total_s, r.kernel_s, r.buffer_setup_s, r.cpu_lambda_s, r.readback_s, r.dispatch_overhead_s, r.matches_sequential
                );
            }
        }
    }
    Ok(Breakdown {
        device: super::device_name(),
        init_once_s: init,
        warmup,
        reps,
        days,
        rows,
    })
}

impl Breakdown {
    pub fn write(&self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        if let Some(d) = path.parent() {
            std::fs::create_dir_all(d)?;
        }
        std::fs::write(path, serde_json::to_string_pretty(self)? + "\n")?;
        Ok(())
    }
}
