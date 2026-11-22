//! `cuda` backend (NVIDIA GPU, feature `cuda`). Same structure and exactness design as the
//! Metal backend: the agent arrays stay on the device; each step is
//!   [partition by health state every N steps] -> k_place_* + k_locs -> readback of the active
//!   location list -> host f64 lambda (exact integer thresholds) -> upload -> k_apply_lam ->
//!   k_update_* -> k_stats -> readback of 16 words.
//! Kernels are the `.cu` files in `crates/kernels-c/cuda`, compiled with NVRTC at run time
//! through `cudarc` (the CUDA driver and libnvrtc are loaded dynamically, so building this
//! feature needs no toolkit; running it needs an NVIDIA driver + CUDA runtime libraries).
//!
//! STATUS: written without access to an NVIDIA GPU. It compiles (`cargo check --features
//! cuda`) but has never been run; the golden tests in `tests/golden.rs` are the first check.
// chunks_exact(4) is deliberate (the vector types are passed as flat u32 arrays).
#![allow(clippy::chunks_exact_to_as_chunks)]
use crate::gpu_common::*;
use cudarc::driver::{
    sys, CudaContext, CudaEvent, CudaFunction, CudaSlice, CudaStream, DeviceRepr, LaunchConfig,
    PushKernelArg,
};
use cudarc::nvrtc::{compile_ptx_with_opts, CompileOptions, Ptx};
use epidemicsim_core::agents::state::{self, COUNT};
use epidemicsim_core::backend::{Backend, InterventionState, StepStats};
use epidemicsim_core::world::{World, STEPS_PER_DAY};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use std::time::Instant;

pub mod measure;

const COMMON: &str = include_str!("../../../kernels-c/cuda/common.cuh");
const FILES: [&str; 3] = [
    include_str!("../../../kernels-c/cuda/place.cu"),
    include_str!("../../../kernels-c/cuda/update.cu"),
    include_str!("../../../kernels-c/cuda/stats.cu"),
];
const STATS_WORDS: usize = 16;

// SAFETY: plain-old-data, repr(C), mirrors `struct P` / `struct AgentAoS` in common.cuh.
unsafe impl DeviceRepr for Params {}

#[repr(C, align(8))]
#[derive(Clone, Copy, Default)]
struct AgentAoS {
    id: u32,
    home: u32,
    work: u32,
    loc: u32,
    timer: u16,
    vdays: u16,
    age: u8,
    state: u8,
    vs: u8,
    pad: u8,
}
// SAFETY: see above (24 bytes, no padding bytes that matter).
unsafe impl DeviceRepr for AgentAoS {}

/// How per-location infectious counts are accumulated.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CountMode {
    /// One global atomic per agent.
    Atomic,
    /// Warp-level segmented reduction over runs of equal location, then one atomic per run.
    Segmented,
}

/// Update kernel variant.
#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum UpdateKind {
    /// Branchy: control flow follows the health state.
    Branch,
    /// Predicated: every outcome computed for every agent, selected without branches.
    Pred,
    /// Branchy + shared-memory tile of per-location lambda entries.
    Smem,
    /// Split: progress kernel + one thread per (susceptible agent, location) pair.
    Pairs,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Layout {
    Soa,
    Aos,
}

#[derive(Clone, Debug)]
pub struct CudaOpts {
    /// Threads per block (64, 128, 256, 512).
    pub block: u32,
    pub count: CountMode,
    pub update: UpdateKind,
    pub layout: Layout,
    /// Re-partition agents by health state every N steps (0 = never).
    pub partition_every: u32,
}

impl Default for CudaOpts {
    fn default() -> Self {
        Self {
            block: 256,
            count: CountMode::Atomic,
            update: UpdateKind::Branch,
            layout: Layout::Soa,
            partition_every: 0,
        }
    }
}

impl CudaOpts {
    pub fn validate(&self) -> Result<(), String> {
        if ![64, 128, 256, 512].contains(&self.block) {
            return Err("cuda block size must be 64, 128, 256 or 512".into());
        }
        if self.layout == Layout::Aos {
            if !matches!(self.update, UpdateKind::Branch | UpdateKind::Pred) {
                return Err("cuda aos layout supports update=branch|pred only".into());
            }
            if self.count == CountMode::Segmented || self.partition_every > 0 {
                return Err("cuda aos layout supports count=atomic and no partition only".into());
            }
        }
        Ok(())
    }

    /// Short tag for file names / table rows.
    pub fn tag(&self) -> String {
        let mut s = format!(
            "{:?}_{:?}_{:?}_b{}",
            self.layout, self.count, self.update, self.block
        )
        .to_lowercase();
        if self.partition_every > 0 {
            s += &format!("_part{}", self.partition_every);
        }
        s
    }
}

pub fn parse_count(s: &str) -> Result<CountMode, String> {
    match s {
        "atomic" => Ok(CountMode::Atomic),
        "segmented" => Ok(CountMode::Segmented),
        _ => Err(format!("unknown cuda count mode '{s}' (atomic|segmented)")),
    }
}

pub fn parse_update(s: &str) -> Result<UpdateKind, String> {
    match s {
        "branch" => Ok(UpdateKind::Branch),
        "pred" => Ok(UpdateKind::Pred),
        "smem" => Ok(UpdateKind::Smem),
        "pairs" => Ok(UpdateKind::Pairs),
        _ => Err(format!(
            "unknown cuda update '{s}' (branch|pred|smem|pairs)"
        )),
    }
}

pub fn parse_layout(s: &str) -> Result<Layout, String> {
    match s {
        "soa" => Ok(Layout::Soa),
        "aos" => Ok(Layout::Aos),
        _ => Err(format!("unknown cuda layout '{s}' (soa|aos)")),
    }
}

/// Wall-clock / event seconds per part of the run (summed over steps).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Timings {
    /// Context + module load + buffer allocation + initial host->device upload.
    pub setup_s: f64,
    /// Of which: the initial upload of the agent arrays (and bytes).
    pub upload_s: f64,
    pub upload_bytes: u64,
    /// CUDA-event time of all kernels (partition + cmd1 + cmd2).
    pub kernel_s: f64,
    pub partition_s: f64,
    /// k_place_* + k_locs.
    pub cmd1_s: f64,
    /// k_apply_lam + k_update_* + k_stats.
    pub cmd2_s: f64,
    /// Of cmd2: just the update kernel(s).
    pub update_s: f64,
    /// CPU lambda stage.
    pub cpu_lambda_s: f64,
    /// Per-step host->device transfers (lambda entries), wall clock.
    pub h2d_s: f64,
    pub h2d_bytes: u64,
    /// Device->host readback (active list, stats, sync_to_host), wall clock.
    pub readback_s: f64,
    pub d2h_bytes: u64,
    /// Everything inside `step`.
    pub step_wall_s: f64,
    pub steps: u64,
    /// Smem variant: blocks that cached a tile / all blocks (from device counters).
    pub smem_blocks_hit: u64,
    pub smem_blocks: u64,
}

fn arch_for(cc: (i32, i32)) -> Option<&'static str> {
    Some(match cc.0 * 10 + cc.1 {
        60 => "compute_60",
        61 => "compute_61",
        70 => "compute_70",
        75 => "compute_75",
        80 => "compute_80",
        86 => "compute_86",
        89 => "compute_89",
        90 => "compute_90",
        _ => return None, // default PTX target; the driver JIT-compiles it
    })
}

fn source() -> String {
    let mut s = String::from(COMMON);
    for f in FILES {
        for line in f.lines() {
            if !line.trim_start().starts_with("#include \"common.cuh\"") {
                s.push_str(line);
                s.push('\n');
            }
        }
    }
    s
}

/// PTX per target arch, compiled once per process.
fn ptx_for(arch: Option<&'static str>) -> Result<Ptx, String> {
    static CACHE: Mutex<Option<HashMap<&'static str, String>>> = Mutex::new(None);
    let key = arch.unwrap_or("default");
    let mut g = CACHE.lock().map_err(|e| e.to_string())?;
    let map = g.get_or_insert_with(HashMap::new);
    if let Some(src) = map.get(key) {
        return Ok(Ptx::from_src(src.clone()));
    }
    let opts = CompileOptions {
        arch,
        // Doubles never appear in the kernels; keep integer/Philox maths exact.
        fmad: Some(false),
        ..Default::default()
    };
    let ptx = compile_ptx_with_opts(source(), opts).map_err(|e| format!("NVRTC: {e}"))?;
    map.insert(key, ptx.to_src());
    Ok(ptx)
}

struct Fns(HashMap<&'static str, CudaFunction>);

impl Fns {
    fn get(&self, n: &str) -> CudaFunction {
        self.0
            .get(n)
            .unwrap_or_else(|| panic!("missing CUDA kernel {n}"))
            .clone()
    }
}

const KERNELS: &[&str] = &[
    "k_place_atomic",
    "k_place_aos",
    "k_place_segmented",
    "k_locs",
    "k_apply_lam",
    "k_update_soa_branch",
    "k_update_soa_pred",
    "k_update_soa_smem",
    "k_update_aos_branch",
    "k_update_aos_pred",
    "k_pairs_build",
    "k_infect_pairs",
    "k_progress",
    "k_vclock",
    "k_stats",
    "k_stats_aos",
    "k_part_offsets",
    "k_part_scatter",
    "k_test_lt_prod",
    "k_test_philox",
];

struct Ctx {
    ctx: Arc<CudaContext>,
    stream: Arc<CudaStream>,
    fns: Fns,
    device: String,
}

fn e<T: std::fmt::Display>(x: T) -> String {
    x.to_string()
}

fn open() -> Result<Ctx, String> {
    let ctx = CudaContext::new(0).map_err(|x| format!("no CUDA device / driver: {x}"))?;
    let cc = ctx.compute_capability().map_err(e)?;
    let ptx = ptx_for(arch_for(cc))?;
    let module = ctx.load_module(ptx).map_err(e)?;
    let mut m = HashMap::new();
    for &k in KERNELS {
        m.insert(k, module.load_function(k).map_err(e)?);
    }
    let device = ctx.name().map_err(e)?;
    let stream = ctx.new_stream().map_err(e)?;
    Ok(Ctx {
        ctx,
        stream,
        fns: Fns(m),
        device,
    })
}

pub fn available() -> bool {
    open().is_ok()
}

pub fn device_name() -> String {
    open().map(|c| c.device).unwrap_or_default()
}

/// Device-resident state, rebuilt whenever the world is (re)uploaded.
struct Dev {
    n: usize,
    nloc: usize,
    // SoA agent arrays (state/timer/vdays/loc live here only for the SoA layout).
    id: CudaSlice<u32>,
    age: CudaSlice<u8>,
    state: CudaSlice<u8>,
    timer: CudaSlice<u16>,
    home: CudaSlice<u32>,
    work: CudaSlice<u32>,
    vstat: CudaSlice<u8>,
    vdays: CudaSlice<u16>,
    loc: CudaSlice<u32>,
    // AoS agent records (AoS layout only; otherwise a 1-element dummy).
    aos: CudaSlice<AgentAoS>,
    occ: CudaSlice<u32>,
    inf_a: CudaSlice<u32>,
    inf_i: CudaSlice<u32>,
    lam_thr: CudaSlice<u64>,
    lam_bits: CudaSlice<u64>,
    active: CudaSlice<u32>,
    nactive: CudaSlice<u32>,
    ent: CudaSlice<u64>,
    stats: CudaSlice<u32>,
    dtab: CudaSlice<u32>,
    vcap: usize,
    hosp: CudaSlice<u64>,
    binf: CudaSlice<u64>,
    order: CudaSlice<u32>,
    pcnt: CudaSlice<u32>,
    cursor: CudaSlice<u32>,
    pairs: CudaSlice<u32>,
    npairs: CudaSlice<u32>,
}

struct Events {
    p0: CudaEvent,
    c0: CudaEvent,
    c1: CudaEvent,
    c2: CudaEvent,
    u0: CudaEvent,
    u1: CudaEvent,
    c3: CudaEvent,
}

pub struct CudaBackend {
    opts: CudaOpts,
    g: Ctx,
    ev: Events,
    dev: Option<Dev>,
    tables: Option<Tables>,
    expect_step: u32,
    dirty: bool,
    days_hi: u32,
    steps_done: u64,
    have_order: bool,
    t: Timings,
}

macro_rules! launch {
    ($s:expr, $f:expr, $cfg:expr, $($a:expr),+ $(,)?) => {{
        let f = $s.g.fns.get($f);
        let mut b = $s.g.stream.launch_builder(&f);
        $( b.arg($a); )+
        // SAFETY: argument lists mirror the kernel signatures in crates/kernels-c/cuda/*.cu.
        unsafe { b.launch($cfg) }.map_err(e)?;
    }};
}

fn cfg1(n: usize, block: u32) -> LaunchConfig {
    LaunchConfig {
        grid_dim: ((n as u32).div_ceil(block).max(1), 1, 1),
        block_dim: (block, 1, 1),
        shared_mem_bytes: 0,
    }
}

impl CudaBackend {
    pub fn new(opts: CudaOpts) -> Result<Self, String> {
        let t0 = Instant::now();
        opts.validate()?;
        let g = open()?;
        let mk = || {
            g.ctx
                .new_event(Some(sys::CUevent_flags::CU_EVENT_DEFAULT))
                .map_err(e)
        };
        let ev = Events {
            p0: mk()?,
            c0: mk()?,
            c1: mk()?,
            c2: mk()?,
            u0: mk()?,
            u1: mk()?,
            c3: mk()?,
        };
        let t = Timings {
            setup_s: t0.elapsed().as_secs_f64(),
            ..Default::default()
        };
        Ok(Self {
            opts,
            g,
            ev,
            dev: None,
            tables: None,
            expect_step: u32::MAX,
            dirty: true,
            days_hi: 0,
            steps_done: 0,
            have_order: false,
            t,
        })
    }

    pub fn timings(&self) -> &Timings {
        &self.t
    }

    pub fn options(&self) -> &CudaOpts {
        &self.opts
    }

    fn vtables_dev(
        &self,
        w: &World,
        cap: usize,
    ) -> Result<(CudaSlice<u64>, CudaSlice<u64>), String> {
        let (hosp, binf) = vtables(w, cap);
        let s = &self.g.stream;
        Ok((
            s.clone_htod(&hosp).map_err(e)?,
            s.clone_htod(&binf).map_err(e)?,
        ))
    }

    fn upload_world(&mut self, w: &World) -> Result<(), String> {
        let t0 = Instant::now();
        let s = self.g.stream.clone();
        let (n, nloc) = (w.agents.len(), w.layout.total());
        let a = &w.agents;
        let aos_layout = self.opts.layout == Layout::Aos;
        let up0 = Instant::now();
        let mut bytes = 0u64;
        let sz = |len: usize, w: usize| (len * w) as u64;
        // SoA arrays always exist (the AoS layout leaves them as 1-element placeholders except
        // the read-only ones nothing needs).
        let (id, age, state, timer, home, work, vstat, vdays, aos);
        if aos_layout {
            let recs: Vec<AgentAoS> = (0..n)
                .map(|i| AgentAoS {
                    id: a.id[i],
                    home: a.home_loc[i],
                    work: a.work_loc[i],
                    loc: 0,
                    timer: a.timer[i],
                    vdays: a.vax_days[i],
                    age: a.age_group[i],
                    state: a.state[i],
                    vs: a.vax_status[i],
                    pad: 0,
                })
                .collect();
            bytes += sz(n, 24);
            aos = s.clone_htod(&recs).map_err(e)?;
            let z8 = [0u8; 1];
            let z16 = [0u16; 1];
            let z32 = [0u32; 1];
            id = s.clone_htod(&z32).map_err(e)?;
            home = s.clone_htod(&z32).map_err(e)?;
            work = s.clone_htod(&z32).map_err(e)?;
            age = s.clone_htod(&z8).map_err(e)?;
            state = s.clone_htod(&z8).map_err(e)?;
            vstat = s.clone_htod(&z8).map_err(e)?;
            timer = s.clone_htod(&z16).map_err(e)?;
            vdays = s.clone_htod(&z16).map_err(e)?;
        } else {
            aos = s.clone_htod(&[AgentAoS::default()]).map_err(e)?;
            id = s.clone_htod(&a.id).map_err(e)?;
            age = s.clone_htod(&a.age_group).map_err(e)?;
            state = s.clone_htod(&a.state).map_err(e)?;
            timer = s.clone_htod(&a.timer).map_err(e)?;
            home = s.clone_htod(&a.home_loc).map_err(e)?;
            work = s.clone_htod(&a.work_loc).map_err(e)?;
            vstat = s.clone_htod(&a.vax_status).map_err(e)?;
            vdays = s.clone_htod(&a.vax_days).map_err(e)?;
            bytes += sz(n, 4 + 1 + 1 + 2 + 4 + 4 + 1 + 2);
        }
        s.synchronize().map_err(e)?;
        self.t.upload_s += up0.elapsed().as_secs_f64();
        self.t.upload_bytes += bytes;
        self.days_hi = (0..n)
            .filter(|&i| a.vax_status[i] != 0)
            .map(|i| u32::from(a.vax_days[i]))
            .max()
            .unwrap_or(0);
        let vcap = (self.days_hi as usize + 1 + 64)
            .next_power_of_two()
            .min(DAY_CAP);
        let tables = Tables::new(&w.disease, &w.vaccine)?;
        let (hosp, binf) = self.vtables_dev(w, vcap)?;
        let dtab = s.clone_htod(&tables.dtab).map_err(e)?;
        self.tables = Some(tables);
        let z = |len: usize| s.alloc_zeros::<u32>(len.max(1)).map_err(e);
        let want_order = self.opts.partition_every > 0;
        self.dev = Some(Dev {
            n,
            nloc,
            id,
            age,
            state,
            timer,
            home,
            work,
            vstat,
            vdays,
            loc: z(if aos_layout { 1 } else { n })?,
            aos,
            occ: z(nloc)?,
            inf_a: z(nloc)?,
            inf_i: z(nloc)?,
            lam_thr: s.alloc_zeros::<u64>(nloc.max(1)).map_err(e)?,
            lam_bits: s.alloc_zeros::<u64>(nloc.max(1)).map_err(e)?,
            active: z(nloc * 4)?,
            nactive: z(1)?,
            ent: s.alloc_zeros::<u64>((nloc * 3).max(3)).map_err(e)?,
            stats: z(STATS_WORDS)?,
            dtab,
            vcap,
            hosp,
            binf,
            order: z(if want_order { n } else { 1 })?,
            pcnt: z(8)?,
            cursor: z(8)?,
            pairs: z(if self.opts.update == UpdateKind::Pairs {
                n
            } else {
                1
            })?,
            npairs: z(1)?,
        });
        s.synchronize().map_err(e)?;
        self.dirty = false;
        self.have_order = false;
        self.expect_step = w.step;
        self.t.setup_s += t0.elapsed().as_secs_f64();
        Ok(())
    }

    fn refresh_tables(&mut self, w: &World) -> Result<(), String> {
        if self
            .tables
            .as_ref()
            .is_none_or(|t| t.sig != signature(&w.disease, &w.vaccine))
        {
            let tb = Tables::new(&w.disease, &w.vaccine)?;
            let d = self.g.stream.clone_htod(&tb.dtab).map_err(e)?;
            self.tables = Some(tb);
            let vcap = self.dev.as_ref().expect("uploaded").vcap;
            let (h, b) = self.vtables_dev(w, vcap)?;
            let dev = self.dev.as_mut().expect("uploaded");
            dev.dtab = d;
            dev.hosp = h;
            dev.binf = b;
        }
        let vcap = self.dev.as_ref().expect("uploaded").vcap;
        if self.days_hi as usize + 1 >= vcap && vcap < DAY_CAP {
            let cap = (vcap * 2).min(DAY_CAP);
            let (h, b) = self.vtables_dev(w, cap)?;
            let dev = self.dev.as_mut().expect("uploaded");
            dev.vcap = cap;
            dev.hosp = h;
            dev.binf = b;
        }
        Ok(())
    }

    fn ms(&self, a: &CudaEvent, b: &CudaEvent) -> f64 {
        f64::from(a.elapsed_ms(b).unwrap_or(0.0)) / 1000.0
    }

    fn try_step(&mut self, w: &mut World, iv: &InterventionState) -> Result<StepStats, String> {
        let t_step = Instant::now();
        let n = w.agents.len();
        if self.dirty || self.dev.as_ref().is_none_or(|d| d.n != n) || self.expect_step != w.step {
            self.upload_world(w)?;
        }
        self.refresh_tables(w)?;
        let block = self.opts.block;
        let aos = self.opts.layout == Layout::Aos;
        let mut p = {
            let (tb, dev) = (
                self.tables.as_ref().expect("t"),
                self.dev.as_ref().expect("d"),
            );
            build_params(w, iv, tb, dev.n, dev.nloc, dev.vcap)
        };
        let step = w.step;
        let stream = self.g.stream.clone();
        let (nn, nloc) = (p.n as usize, p.nloc as usize);

        // ---- optional partition of agents by health state ----
        self.ev.p0.record(&stream).map_err(e)?;
        let every = u64::from(self.opts.partition_every);
        if every > 0 && (!self.have_order || self.steps_done.is_multiple_of(every)) {
            let dev = self.dev.as_mut().expect("d");
            stream.memset_zeros(&mut dev.pcnt).map_err(e)?;
            let gs = nn.div_ceil(block as usize * 8).clamp(1, 1024);
            let cfgs = LaunchConfig {
                grid_dim: (gs as u32, 1, 1),
                block_dim: (block, 1, 1),
                shared_mem_bytes: 0,
            };
            launch!(self, "k_stats", cfgs, &p, &dev.state, &mut dev.pcnt);
            launch!(
                self,
                "k_part_offsets",
                cfg1(1, 32),
                &dev.pcnt,
                &mut dev.cursor
            );
            launch!(
                self,
                "k_part_scatter",
                cfg1(nn, block),
                &p,
                &dev.state,
                &mut dev.cursor,
                &mut dev.order
            );
            self.have_order = true;
        }
        p.flags = u32::from(every > 0 && self.have_order);
        self.ev.c0.record(&stream).map_err(e)?;

        // ---- cmd 1: placement + per-location counts, active-list compaction ----
        {
            let dev = self.dev.as_mut().expect("d");
            stream.memset_zeros(&mut dev.nactive).map_err(e)?;
            if aos {
                launch!(
                    self,
                    "k_place_aos",
                    cfg1(nn, block),
                    &p,
                    &mut dev.aos,
                    &mut dev.occ,
                    &mut dev.inf_a,
                    &mut dev.inf_i
                );
            } else {
                let name = match self.opts.count {
                    CountMode::Atomic => "k_place_atomic",
                    CountMode::Segmented => "k_place_segmented",
                };
                launch!(
                    self,
                    name,
                    cfg1(nn, block),
                    &p,
                    &dev.id,
                    &dev.state,
                    &dev.home,
                    &dev.work,
                    &mut dev.loc,
                    &mut dev.occ,
                    &mut dev.inf_a,
                    &mut dev.inf_i
                );
            }
            launch!(
                self,
                "k_locs",
                cfg1(nloc, block),
                &p,
                &mut dev.occ,
                &mut dev.inf_a,
                &mut dev.inf_i,
                &mut dev.lam_thr,
                &mut dev.lam_bits,
                &mut dev.active,
                &mut dev.nactive
            );
        }
        self.ev.c1.record(&stream).map_err(e)?;

        // ---- readback of the active-location list ----
        let t_rb = Instant::now();
        let (na, list) = {
            let dev = self.dev.as_ref().expect("d");
            let na = stream.clone_dtoh(&dev.nactive).map_err(e)?[0] as usize;
            let raw = if na > 0 {
                stream.clone_dtoh(&dev.active.slice(0..4 * na)).map_err(e)?
            } else {
                Vec::new()
            };
            stream.synchronize().map_err(e)?;
            let list: Vec<[u32; 4]> = raw
                .chunks_exact(4)
                .map(|c| [c[0], c[1], c[2], c[3]])
                .collect();
            (na, list)
        };
        let readback1 = t_rb.elapsed().as_secs_f64();
        self.t.d2h_bytes += 4 + 16 * na as u64;

        // ---- CPU stage: lambda (f64) for the active locations only ----
        let t_cpu = Instant::now();
        let mut ent: Vec<u64> = Vec::with_capacity(3 * na);
        let infectious_total = lambda_stage(w, iv, &list, |l, t, b| {
            ent.extend([u64::from(l), t, b]);
        });
        let cpu_lambda = t_cpu.elapsed().as_secs_f64();

        // ---- upload the lambda entries ----
        let t_up = Instant::now();
        let cnt = (ent.len() / 3) as u32;
        {
            let dev = self.dev.as_mut().expect("d");
            if cnt > 0 {
                stream
                    .memcpy_htod(&ent, &mut dev.ent.slice_mut(0..ent.len()))
                    .map_err(e)?;
            }
            stream.synchronize().map_err(e)?;
        }
        let h2d = t_up.elapsed().as_secs_f64();
        self.t.h2d_bytes += 8 * ent.len() as u64;

        // ---- cmd 2: apply lambda, agent update, statistics ----
        self.ev.c2.record(&stream).map_err(e)?;
        {
            let dev = self.dev.as_mut().expect("d");
            stream.memset_zeros(&mut dev.stats).map_err(e)?;
            if cnt > 0 {
                launch!(
                    self,
                    "k_apply_lam",
                    cfg1(cnt as usize, block),
                    &dev.ent,
                    &cnt,
                    &mut dev.lam_thr,
                    &mut dev.lam_bits
                );
            }
        }
        self.ev.u0.record(&stream).map_err(e)?;
        {
            let dev = self.dev.as_mut().expect("d");
            let upd = cfg1(nn, block);
            match (self.opts.layout, self.opts.update) {
                (Layout::Soa, UpdateKind::Branch | UpdateKind::Pred | UpdateKind::Smem) => {
                    let name = match self.opts.update {
                        UpdateKind::Branch => "k_update_soa_branch",
                        UpdateKind::Pred => "k_update_soa_pred",
                        _ => "k_update_soa_smem",
                    };
                    launch!(
                        self,
                        name,
                        upd,
                        &p,
                        &dev.order,
                        &dev.id,
                        &dev.age,
                        &mut dev.state,
                        &mut dev.timer,
                        &dev.vstat,
                        &mut dev.vdays,
                        &dev.loc,
                        &dev.lam_thr,
                        &dev.lam_bits,
                        &dev.hosp,
                        &dev.binf,
                        &dev.dtab,
                        &mut dev.stats
                    );
                }
                (Layout::Soa, UpdateKind::Pairs) => {
                    // progress first: it must not see agents infected in this step.
                    launch!(
                        self,
                        "k_progress",
                        upd,
                        &p,
                        &dev.order,
                        &dev.id,
                        &dev.age,
                        &mut dev.state,
                        &mut dev.timer,
                        &dev.vstat,
                        &dev.vdays,
                        &dev.hosp,
                        &dev.dtab
                    );
                    stream.memset_zeros(&mut dev.npairs).map_err(e)?;
                    launch!(
                        self,
                        "k_pairs_build",
                        upd,
                        &p,
                        &dev.state,
                        &dev.loc,
                        &dev.lam_bits,
                        &mut dev.pairs,
                        &mut dev.npairs
                    );
                    launch!(
                        self,
                        "k_infect_pairs",
                        upd,
                        &p,
                        &dev.pairs,
                        &dev.npairs,
                        &dev.id,
                        &mut dev.state,
                        &mut dev.timer,
                        &dev.vstat,
                        &dev.vdays,
                        &dev.loc,
                        &dev.lam_thr,
                        &dev.lam_bits,
                        &dev.binf,
                        &dev.dtab,
                        &mut dev.stats
                    );
                    if p.day_end != 0 {
                        launch!(self, "k_vclock", upd, &p, &dev.vstat, &mut dev.vdays);
                    }
                }
                (Layout::Aos, k) => {
                    let name = if k == UpdateKind::Pred {
                        "k_update_aos_pred"
                    } else {
                        "k_update_aos_branch"
                    };
                    launch!(
                        self,
                        name,
                        upd,
                        &p,
                        &mut dev.aos,
                        &dev.order,
                        &dev.lam_thr,
                        &dev.lam_bits,
                        &dev.hosp,
                        &dev.binf,
                        &dev.dtab,
                        &mut dev.stats
                    );
                }
            }
        }
        self.ev.u1.record(&stream).map_err(e)?;
        {
            let dev = self.dev.as_mut().expect("d");
            let gs = nn.div_ceil(block as usize * 8).clamp(1, 1024);
            let cfgs = LaunchConfig {
                grid_dim: (gs as u32, 1, 1),
                block_dim: (block, 1, 1),
                shared_mem_bytes: 0,
            };
            if aos {
                launch!(self, "k_stats_aos", cfgs, &p, &dev.aos, &mut dev.stats);
            } else {
                launch!(self, "k_stats", cfgs, &p, &dev.state, &mut dev.stats);
            }
        }
        self.ev.c3.record(&stream).map_err(e)?;

        let t_rb2 = Instant::now();
        let words = stream
            .clone_dtoh(&self.dev.as_ref().expect("d").stats)
            .map_err(e)?;
        stream.synchronize().map_err(e)?;
        let readback2 = t_rb2.elapsed().as_secs_f64();
        self.t.d2h_bytes += 4 * STATS_WORDS as u64;

        w.step += 1;
        self.expect_step = w.step;
        self.steps_done += 1;
        if step % STEPS_PER_DAY == STEPS_PER_DAY - 1 {
            self.days_hi = (self.days_hi + 1).min(65_535);
        }
        let mut counts = [0u32; COUNT];
        counts.copy_from_slice(&words[..COUNT]);
        let new_infections = words[7];
        let r_t = if infectious_total > 0.0 {
            f64::from(new_infections) / infectious_total * w.infectious_steps_per_case
        } else {
            0.0
        };
        let part = self.ms(&self.ev.p0, &self.ev.c0);
        let k1 = self.ms(&self.ev.c0, &self.ev.c1);
        let k2 = self.ms(&self.ev.c2, &self.ev.c3);
        self.t.partition_s += part;
        self.t.kernel_s += part + k1 + k2;
        self.t.cmd1_s += k1;
        self.t.cmd2_s += k2;
        self.t.update_s += self.ms(&self.ev.u0, &self.ev.u1);
        self.t.cpu_lambda_s += cpu_lambda;
        self.t.h2d_s += h2d;
        self.t.readback_s += readback1 + readback2;
        self.t.smem_blocks_hit += u64::from(words[8]);
        self.t.smem_blocks += u64::from(words[9]);
        self.t.step_wall_s += t_step.elapsed().as_secs_f64();
        self.t.steps += 1;
        Ok(StepStats {
            step,
            day: step / STEPS_PER_DAY,
            counts,
            new_infections,
            hospital_load: counts[state::H as usize],
            r_t,
        })
    }

    fn try_sync(&mut self, w: &mut World) -> Result<(), String> {
        let t0 = Instant::now();
        let Some(dev) = self.dev.as_ref() else {
            return Ok(());
        };
        if self.dirty || dev.n != w.agents.len() {
            return Ok(());
        }
        let s = &self.g.stream;
        let n = dev.n;
        if self.opts.layout == Layout::Aos {
            let recs = s.clone_dtoh(&dev.aos).map_err(e)?;
            for (i, r) in recs.iter().enumerate() {
                w.agents.state[i] = r.state;
                w.agents.timer[i] = r.timer;
                w.agents.vax_days[i] = r.vdays;
            }
            self.t.d2h_bytes += 24 * n as u64;
        } else {
            w.agents.state = s.clone_dtoh(&dev.state).map_err(e)?;
            w.agents.timer = s.clone_dtoh(&dev.timer).map_err(e)?;
            w.agents.vax_days = s.clone_dtoh(&dev.vdays).map_err(e)?;
            self.t.d2h_bytes += 5 * n as u64;
        }
        s.synchronize().map_err(e)?;
        self.t.readback_s += t0.elapsed().as_secs_f64();
        Ok(())
    }
}

impl Backend for CudaBackend {
    fn name(&self) -> &'static str {
        "cuda"
    }

    fn step(&mut self, w: &mut World, iv: &InterventionState) -> StepStats {
        self.try_step(w, iv).expect("cuda step")
    }

    fn sync_to_host(&mut self, w: &mut World) {
        self.try_sync(w).expect("cuda sync");
    }

    fn host_changed(&mut self) {
        self.dirty = true;
    }
}

/// Philox blocks computed on the GPU for the given counters (test hook).
pub fn test_philox(ctr: &[[u32; 4]], key: [u32; 2]) -> Result<Vec<[u32; 4]>, String> {
    let g = open()?;
    let flat: Vec<u32> = ctr.iter().flatten().copied().collect();
    let s = &g.stream;
    let (din, dkey) = (
        s.clone_htod(&flat).map_err(e)?,
        s.clone_htod(&key).map_err(e)?,
    );
    let mut out = s.alloc_zeros::<u32>(flat.len().max(1)).map_err(e)?;
    let n = ctr.len() as u32;
    let f = g.fns.get("k_test_philox");
    let mut b = s.launch_builder(&f);
    b.arg(&din).arg(&dkey).arg(&mut out).arg(&n);
    // SAFETY: matches `k_test_philox(const uint4*, const uint2*, uint4*, u32)`.
    unsafe { b.launch(cfg1(ctr.len(), 128)) }.map_err(e)?;
    let v = s.clone_dtoh(&out).map_err(e)?;
    Ok(v.chunks_exact(4)
        .map(|c| [c[0], c[1], c[2], c[3]])
        .collect())
}

/// Exact-product test kernel (test hook).
pub fn test_lt_prod(xs: &[u32], a: &[u64], b: &[u64]) -> Result<Vec<bool>, String> {
    let g = open()?;
    let s = &g.stream;
    let (dx, da, db) = (
        s.clone_htod(xs).map_err(e)?,
        s.clone_htod(a).map_err(e)?,
        s.clone_htod(b).map_err(e)?,
    );
    let mut out = s.alloc_zeros::<u8>(xs.len().max(1)).map_err(e)?;
    let n = xs.len() as u32;
    let f = g.fns.get("k_test_lt_prod");
    let mut l = s.launch_builder(&f);
    l.arg(&dx).arg(&da).arg(&db).arg(&mut out).arg(&n);
    // SAFETY: matches `k_test_lt_prod(const u32*, const u64*, const u64*, u8*, u32)`.
    unsafe { l.launch(cfg1(xs.len(), 128)) }.map_err(e)?;
    Ok(s.clone_dtoh(&out)
        .map_err(e)?
        .iter()
        .map(|&v| v != 0)
        .collect())
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::rng::{philox4x32_10, to_unit};

    fn lcg(s: &mut u64) -> u64 {
        *s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *s >> 11
    }

    #[test]
    fn philox_matches_cpu() {
        if !available() {
            eprintln!("no CUDA device: skipping");
            return;
        }
        let mut s = 7;
        let ctr: Vec<[u32; 4]> = (0..100_000)
            .map(|_| std::array::from_fn(|_| lcg(&mut s) as u32))
            .collect();
        let key = [0xa409_3822, 0x299f_31d0];
        let got = test_philox(&ctr, key).unwrap();
        for (c, g) in ctr.iter().zip(got) {
            assert_eq!(philox4x32_10(*c, key), g);
        }
    }

    #[test]
    fn exact_product_compare_matches_f64() {
        if !available() {
            return;
        }
        let mut s = 99;
        let (mut xs, mut a, mut b) = (Vec::new(), Vec::new(), Vec::new());
        for k in 0..400_000u32 {
            let lam = (lcg(&mut s) as f64 / (1u64 << 53) as f64).powi(3).max(1e-9);
            let ve = (lcg(&mut s) as f64 / (1u64 << 53) as f64) * 0.95;
            let bb = 1.0 - ve;
            let x = if k % 2 == 0 {
                lcg(&mut s) as u32
            } else {
                let c = count_below(lam * bb) as i64;
                (c + (lcg(&mut s) % 5) as i64 - 2).clamp(0, u32::MAX as i64) as u32
            };
            xs.push(x);
            a.push(lam.to_bits());
            b.push(bb.to_bits());
        }
        let got = test_lt_prod(&xs, &a, &b).unwrap();
        for k in 0..xs.len() {
            let want = to_unit(xs[k]) < f64::from_bits(a[k]) * f64::from_bits(b[k]);
            assert_eq!(want, got[k], "k={k}");
        }
    }
}
