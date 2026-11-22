//! CUDA measurements, same JSON shape as the Metal breakdown (`results/gpu/metal_breakdown.json`):
//!   * `breakdown`: block-size sweep (64/128/256/512) of the default variant
//!   * `variants` : kernel-variant matrix (count mode, update kernel, AoS/SoA, smem, pairs)
//!   * `warp`     : branch-heavy vs predicated vs health-partitioned, at several prevalences
//!
//! Every row is checked against the sequential reference when affordable.
use super::{CountMode, CudaBackend, CudaOpts, Layout, Timings, UpdateKind};
use epidemicsim_core::world::STEPS_PER_DAY;
use epidemicsim_core::{synth, Backend, Config, SequentialBackend, StepStats, World};
use serde::Serialize;
use std::time::Instant;

#[derive(Serialize)]
pub struct Row {
    pub agents: usize,
    pub variant: String,
    pub layout: Layout,
    pub count: CountMode,
    pub update: UpdateKind,
    pub partition_every: u32,
    /// Initial infected fraction (warp study) or the default seeding.
    pub prevalence: f64,
    pub block: usize,
    pub steps: u32,
    /// Medians over the timed repetitions, seconds for the whole run.
    pub total_s: f64,
    pub buffer_setup_s: f64,
    /// Of which: initial host->device upload of the agent arrays.
    pub upload_s: f64,
    pub upload_bytes: u64,
    pub kernel_s: f64,
    pub kernel_partition_s: f64,
    pub kernel_cmd1_place_locs_s: f64,
    pub kernel_cmd2_apply_update_stats_s: f64,
    /// Just the update kernel(s): the number to compare for warp-divergence variants.
    pub kernel_update_s: f64,
    pub cpu_lambda_s: f64,
    /// Per-step host->device transfers (lambda entries), wall clock.
    pub h2d_s: f64,
    pub h2d_bytes: u64,
    /// Device->host readback (active list + stats), wall clock.
    pub readback_s: f64,
    pub d2h_bytes: u64,
    /// step wall - kernel - cpu lambda - h2d - readback: launch + sync overhead.
    pub dispatch_overhead_s: f64,
    pub smem_tile_hit_fraction: Option<f64>,
    pub matches_sequential: Option<bool>,
}

#[derive(Serialize)]
pub struct Breakdown {
    pub device: String,
    /// One-time cost: context + NVRTC compile + module load.
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

fn run(world: &World, o: &CudaOpts, steps: u32) -> Result<(f64, Timings, Vec<StepStats>), String> {
    let mut b = CudaBackend::new(o.clone())?;
    let mut w = world.clone();
    let iv = Default::default();
    let t0 = Instant::now();
    let stats: Vec<StepStats> = (0..steps).map(|_| b.step(&mut w, &iv)).collect();
    Ok((t0.elapsed().as_secs_f64(), b.timings().clone(), stats))
}

fn make_world(n: usize, seed: u64, prevalence: Option<f64>) -> World {
    let mut cfg = Config {
        agents: n,
        seed,
        ..Default::default()
    };
    if let Some(p) = prevalence {
        cfg.disease.initial_infected_frac = p;
    }
    synth::generate(&cfg)
}

#[allow(clippy::too_many_arguments)]
fn measure(
    world: &World,
    reference: Option<&Vec<StepStats>>,
    o: &CudaOpts,
    n: usize,
    prevalence: f64,
    steps: u32,
    warmup: usize,
    reps: usize,
) -> Result<Row, String> {
    let mut samples: Vec<(f64, Timings)> = Vec::new();
    let mut last = Vec::new();
    for rep in 0..warmup + reps {
        let (t, tm, st) = run(world, o, steps)?;
        if rep >= warmup {
            samples.push((t, tm));
        }
        last = st;
    }
    let med = |f: &dyn Fn(&Timings) -> f64| median(samples.iter().map(|(_, t)| f(t)).collect());
    let (kernel, cpu, h2d, rb) = (
        med(&|t| t.kernel_s),
        med(&|t| t.cpu_lambda_s),
        med(&|t| t.h2d_s),
        med(&|t| t.readback_s),
    );
    let wall = med(&|t| t.step_wall_s);
    let (hit, all) = (
        med(&|t| t.smem_blocks_hit as f64),
        med(&|t| t.smem_blocks as f64),
    );
    let r = Row {
        agents: n,
        variant: o.tag(),
        layout: o.layout,
        count: o.count,
        update: o.update,
        partition_every: o.partition_every,
        prevalence,
        block: o.block as usize,
        steps,
        total_s: median(samples.iter().map(|s| s.0).collect()),
        buffer_setup_s: med(&|t| t.setup_s),
        upload_s: med(&|t| t.upload_s),
        upload_bytes: samples[0].1.upload_bytes,
        kernel_s: kernel,
        kernel_partition_s: med(&|t| t.partition_s),
        kernel_cmd1_place_locs_s: med(&|t| t.cmd1_s),
        kernel_cmd2_apply_update_stats_s: med(&|t| t.cmd2_s),
        kernel_update_s: med(&|t| t.update_s),
        cpu_lambda_s: cpu,
        h2d_s: h2d,
        h2d_bytes: samples[0].1.h2d_bytes,
        readback_s: rb,
        d2h_bytes: samples[0].1.d2h_bytes,
        dispatch_overhead_s: (wall - kernel - cpu - h2d - rb).max(0.0),
        smem_tile_hit_fraction: (o.update == UpdateKind::Smem && all > 0.0).then(|| hit / all),
        matches_sequential: reference.map(|r| *r == last),
    };
    eprintln!(
        "n={n} {} prev={prevalence}: total {:.3}s kernel {:.3} (update {:.3}) h2d {:.4} d2h {:.4} cpu {:.3} ok={:?}",
        r.variant, r.total_s, r.kernel_s, r.kernel_update_s, r.h2d_s, r.readback_s, r.cpu_lambda_s, r.matches_sequential
    );
    Ok(r)
}

fn sequential(world: &World, steps: u32, n: usize) -> Option<Vec<StepStats>> {
    (n <= 1_000_000).then(|| {
        let mut w = world.clone();
        let mut b = SequentialBackend::default();
        let iv = Default::default();
        (0..steps).map(|_| b.step(&mut w, &iv)).collect()
    })
}

fn finish(rows: Vec<Row>, init: f64, days: u32, warmup: usize, reps: usize) -> Breakdown {
    Breakdown {
        device: super::device_name(),
        init_once_s: init,
        warmup,
        reps,
        days,
        rows,
    }
}

fn init() -> Result<f64, String> {
    let t0 = Instant::now();
    if !super::available() {
        return Err("no CUDA device".into());
    }
    Ok(t0.elapsed().as_secs_f64())
}

pub const BLOCKS: [u32; 4] = [64, 128, 256, 512];

/// Block-size sweep of the default variant (SoA, atomic counts, branchy update).
pub fn breakdown(
    agents: &[usize],
    days: u32,
    warmup: usize,
    reps: usize,
    seed: u64,
) -> Result<Breakdown, String> {
    let init = init()?;
    let steps = days * STEPS_PER_DAY;
    let mut rows = Vec::new();
    for &n in agents {
        let world = make_world(n, seed, None);
        let reference = sequential(&world, steps, n);
        for block in BLOCKS {
            let o = CudaOpts {
                block,
                ..Default::default()
            };
            rows.push(measure(
                &world,
                reference.as_ref(),
                &o,
                n,
                0.0005,
                steps,
                warmup,
                reps,
            )?);
        }
    }
    Ok(finish(rows, init, days, warmup, reps))
}

/// Kernel-variant matrix at a fixed block size, plus a block sweep of the best-guess variants.
pub fn variants(
    agents: &[usize],
    block: u32,
    days: u32,
    warmup: usize,
    reps: usize,
    seed: u64,
) -> Result<Breakdown, String> {
    let init = init()?;
    let steps = days * STEPS_PER_DAY;
    let mut rows = Vec::new();
    let base = CudaOpts {
        block,
        ..Default::default()
    };
    let list: Vec<CudaOpts> = vec![
        base.clone(),
        CudaOpts {
            count: CountMode::Segmented,
            ..base.clone()
        },
        CudaOpts {
            update: UpdateKind::Pred,
            ..base.clone()
        },
        CudaOpts {
            update: UpdateKind::Smem,
            ..base.clone()
        },
        CudaOpts {
            update: UpdateKind::Pairs,
            ..base.clone()
        },
        CudaOpts {
            layout: Layout::Aos,
            ..base.clone()
        },
        CudaOpts {
            layout: Layout::Aos,
            update: UpdateKind::Pred,
            ..base.clone()
        },
    ];
    for &n in agents {
        let world = make_world(n, seed, None);
        let reference = sequential(&world, steps, n);
        for o in &list {
            rows.push(measure(
                &world,
                reference.as_ref(),
                o,
                n,
                0.0005,
                steps,
                warmup,
                reps,
            )?);
        }
    }
    Ok(finish(rows, init, days, warmup, reps))
}

/// Warp divergence: branchy vs predicated vs partitioned-by-health-state, at several
/// initial infection prevalences (fraction of agents seeded in E; the epidemic then evolves
/// for `days` days, so the prevalence drifts upward during the run).
pub fn warp(
    n: usize,
    prevalences: &[f64],
    block: u32,
    days: u32,
    warmup: usize,
    reps: usize,
    seed: u64,
) -> Result<Breakdown, String> {
    let init = init()?;
    let steps = days * STEPS_PER_DAY;
    let mut rows = Vec::new();
    let base = CudaOpts {
        block,
        ..Default::default()
    };
    let list = [
        base.clone(),
        CudaOpts {
            update: UpdateKind::Pred,
            ..base.clone()
        },
        CudaOpts {
            partition_every: 4,
            ..base.clone()
        },
        CudaOpts {
            partition_every: 16,
            ..base.clone()
        },
        CudaOpts {
            update: UpdateKind::Pred,
            partition_every: 16,
            ..base.clone()
        },
    ];
    for &p in prevalences {
        let world = make_world(n, seed, Some(p));
        let reference = sequential(&world, steps, n);
        for o in &list {
            rows.push(measure(
                &world,
                reference.as_ref(),
                o,
                n,
                p,
                steps,
                warmup,
                reps,
            )?);
        }
    }
    Ok(finish(rows, init, days, warmup, reps))
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
