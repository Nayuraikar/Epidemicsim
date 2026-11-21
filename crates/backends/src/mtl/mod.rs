//! `metal` backend (Apple GPU). The agent arrays live in GPU buffers for the whole run; a step
//! is two command buffers with one small CPU stage in between:
//!   cmd 1: k_place (placement + atomic per-location counts), k_locs (compact active locations)
//!   CPU  : lambda = 1 - exp(...) in f64 for the active locations only (Metal has no f64 and
//!          its `exp` would not be bit-identical), written as exact integer thresholds
//!   cmd 2: k_update (progression + infection + vaccine clock), k_stats (reduction)
//! Only the per-step stats (8 words) and the active-location list cross back to the host.
//!
//! Exactness: the CPU compares `to_unit(x) < p` in f64. Here every such `p` becomes the integer
//! `count_below(p)` (so the test is `x < threshold`), computed on the host with the CPU's own
//! predicate. The vaccinated infection test `u < lambda * (1 - ve)` multiplies two runtime
//! values, so the GPU emulates that f64 multiply exactly (`lt_prod`, 64-bit integer maths).
//! Duration draws use a host-built table of the exact switch points of `dur()`.
//! No remaining float differences are known; the golden tests check this.
use crate::kernels::{dur, LamPlan};
use ::metal::*;
use epidemicsim_core::agents::state::{self, COUNT};
use epidemicsim_core::backend::{Backend, InterventionState, StepStats};
use epidemicsim_core::params::{DiseaseParams, VaccineParams};
use epidemicsim_core::rng::to_unit;
use epidemicsim_core::world::{World, STEPS_PER_DAY};
use serde::Serialize;
use std::sync::OnceLock;
use std::time::Instant;

const SRC: &str = include_str!("kernels.metal");
const MAX_DUR: u32 = 4096;
const DAY_CAP: usize = 65_536;

#[derive(Clone, Debug)]
pub struct MetalOpts {
    /// Threads per threadgroup (32, 64, 128, 256).
    pub threadgroup: usize,
    /// Agent / location buffers in private (GPU-only) memory instead of shared.
    pub private: bool,
}

impl Default for MetalOpts {
    fn default() -> Self {
        Self {
            threadgroup: 64,
            private: false,
        }
    }
}

/// Wall-clock seconds spent per part of the run (summed over steps).
#[derive(Clone, Debug, Default, Serialize)]
pub struct Timings {
    /// Pipeline/device initialisation + buffer allocation + upload of the agent arrays.
    pub setup_s: f64,
    /// GPU execution time of all kernels (GPUEndTime - GPUStartTime of each command buffer).
    pub kernel_s: f64,
    /// Of which: command buffer 1 (placement + location compaction) and 2 (update + stats).
    pub cmd1_s: f64,
    pub cmd2_s: f64,
    /// CPU lambda stage (f64 exp for active locations + threshold writes).
    pub cpu_lambda_s: f64,
    /// Reading stats and the active list back from shared memory (+ host sync copies).
    pub readback_s: f64,
    /// Everything inside `step` (includes encode/commit/wait overhead on top of the above).
    pub step_wall_s: f64,
    pub steps: u64,
}

struct Gpu {
    device: Device,
    place: ComputePipelineState,
    locs: ComputePipelineState,
    update: ComputePipelineState,
    stats: ComputePipelineState,
    test_lt: ComputePipelineState,
    test_philox: ComputePipelineState,
}

fn gpu() -> Result<&'static Gpu, String> {
    static G: OnceLock<Result<Gpu, String>> = OnceLock::new();
    G.get_or_init(|| {
        let device = Device::system_default().ok_or("no Metal device")?;
        let lib = device
            .new_library_with_source(SRC, &CompileOptions::new())
            .map_err(|e| format!("MSL compile error: {e}"))?;
        let pipe = |name: &str| -> Result<ComputePipelineState, String> {
            let f = lib.get_function(name, None).map_err(|e| e.to_string())?;
            device
                .new_compute_pipeline_state_with_function(&f)
                .map_err(|e| e.to_string())
        };
        Ok(Gpu {
            place: pipe("k_place")?,
            locs: pipe("k_locs")?,
            update: pipe("k_update")?,
            stats: pipe("k_stats")?,
            test_lt: pipe("k_test_lt_prod")?,
            test_philox: pipe("k_test_philox")?,
            device,
        })
    })
    .as_ref()
    .map_err(Clone::clone)
}

pub fn available() -> bool {
    gpu().is_ok()
}

pub fn device_name() -> String {
    gpu()
        .map(|g| g.device.name().to_string())
        .unwrap_or_default()
}

/// Number of x in [0, 2^32) with `to_unit(x) < t` (the CPU predicate), exactly.
pub fn count_below(t: f64) -> u64 {
    if t.is_nan() || t <= 0.0 {
        return 0;
    }
    if t >= 1.0 {
        return 1 << 32;
    }
    let pred = |x: u64| to_unit(x as u32) < t;
    let mut c = ((t * 4_294_967_296.0).floor() as u64).min(1 << 32);
    while c > 0 && !pred(c - 1) {
        c -= 1;
    }
    while c < (1 << 32) && pred(c) {
        c += 1;
    }
    c
}

/// Switch points of `dur(mean4, u)`: entry k-2 is the smallest x with duration >= k.
fn dur_table(mean4: f64) -> Result<Vec<u32>, String> {
    if mean4 <= 0.0 {
        return Ok(Vec::new());
    }
    let f = |x: u32| u32::from(dur(mean4, to_unit(x)));
    let kmax = f(u32::MAX);
    if kmax > MAX_DUR {
        return Err(format!(
            "duration of {kmax} steps exceeds the metal table limit"
        ));
    }
    Ok((2..=kmax)
        .map(|k| {
            let (mut lo, mut hi) = (0u32, u32::MAX);
            while lo < hi {
                let mid = lo + (hi - lo) / 2;
                if f(mid) >= k {
                    hi = mid;
                } else {
                    lo = mid + 1;
                }
            }
            lo
        })
        .collect())
}

#[repr(C)]
#[derive(Clone, Copy, Default)]
struct Params {
    thr_asym: [u64; 5],
    thr_death: [u64; 5],
    mk_n: u64,
    hp_n: u64,
    nh: u64,
    mp_stay_thr: u64,
    mp_go_thr: u64,
    iso_thr: u64,
    n: u32,
    nloc: u32,
    step: u32,
    k0: u32,
    k1: u32,
    homes_end: u32,
    works_end: u32,
    markets_end: u32,
    mk_start: u32,
    hp_start: u32,
    col_lo: u32,
    col_hi: u32,
    at_work: u32,
    mp_stay_pos: u32,
    mp_go_pos: u32,
    iso_on: u32,
    closed_mask: u32,
    col_closed: u32,
    day_end: u32,
    vstride: u32,
    dcap: u32,
    dcnt: [u32; 4],
    don: [u32; 4],
    pad: u32,
}

/// Host-built tables that depend only on the disease / vaccine parameters.
struct Tables {
    sig: Vec<f64>,
    thr_asym: [u64; 5],
    thr_death: [u64; 5],
    dtab: Vec<u32>,
    dcnt: [u32; 4],
    don: [u32; 4],
    dcap: u32,
}

fn signature(d: &DiseaseParams, v: &VaccineParams) -> Vec<f64> {
    let mut s = Vec::new();
    s.extend(d.p_asymptomatic);
    s.extend(d.p_hospitalised);
    s.extend(d.p_death_hospitalised);
    s.extend([
        d.latent_days,
        d.asymptomatic_days,
        d.symptomatic_days,
        d.hospital_days,
    ]);
    s.extend([
        v.ve_infection,
        v.ve_severe,
        v.ramp_days,
        v.waning_half_life_days,
    ]);
    s
}

impl Tables {
    fn new(d: &DiseaseParams, v: &VaccineParams) -> Result<Self, String> {
        let s4 = f64::from(STEPS_PER_DAY);
        let means = [
            d.latent_days * s4,
            d.asymptomatic_days * s4,
            d.symptomatic_days * s4,
            d.hospital_days * s4,
        ];
        let tabs: Vec<Vec<u32>> = means
            .iter()
            .map(|&m| dur_table(m))
            .collect::<Result<_, _>>()?;
        let dcap = tabs.iter().map(Vec::len).max().unwrap_or(0).max(1);
        let mut dtab = vec![0u32; 4 * dcap];
        let mut dcnt = [0u32; 4];
        let mut don = [0u32; 4];
        for (k, t) in tabs.iter().enumerate() {
            dtab[k * dcap..k * dcap + t.len()].copy_from_slice(t);
            dcnt[k] = t.len() as u32;
            don[k] = u32::from(means[k] > 0.0);
        }
        Ok(Self {
            sig: signature(d, v),
            thr_asym: std::array::from_fn(|a| count_below(d.p_asymptomatic[a])),
            thr_death: std::array::from_fn(|a| count_below(d.p_death_hospitalised[a])),
            dtab,
            dcnt,
            don,
            dcap: dcap as u32,
        })
    }
}

struct Buf {
    shared: Buffer,
}

enum Mem {
    Shared,
    Private,
}

/// Device-resident state, rebuilt whenever the world is (re)uploaded.
struct Dev {
    n: usize,
    nloc: usize,
    id: Buffer,
    age: Buffer,
    state: Buffer,
    timer: Buffer,
    home: Buffer,
    work: Buffer,
    vstat: Buffer,
    vdays: Buffer,
    loc: Buffer,
    occ: Buffer,
    inf_a: Buffer,
    inf_i: Buffer,
    // Always shared: touched by the CPU every step.
    lam_thr: Buf,
    lam_bits: Buf,
    active: Buf,
    nactive: Buf,
    stats: Buf,
    dtab: Buffer,
    vcap: usize,
    hosp: Buffer,
    binf: Buffer,
}

pub mod measure;

pub struct MetalBackend {
    opts: MetalOpts,
    queue: CommandQueue,
    dev: Option<Dev>,
    tables: Option<Tables>,
    /// Step the device state corresponds to (a mismatch with the world forces a re-upload).
    expect_step: u32,
    dirty: bool,
    days_hi: u32,
    t: Timings,
}

fn slice_bytes<T>(v: &[T]) -> *const std::ffi::c_void {
    v.as_ptr().cast()
}

impl MetalBackend {
    pub fn new(opts: MetalOpts) -> Result<Self, String> {
        let t0 = Instant::now();
        let g = gpu()?;
        if ![32, 64, 128, 256].contains(&opts.threadgroup) {
            return Err("metal threadgroup size must be 32, 64, 128 or 256".into());
        }
        let queue = g.device.new_command_queue();
        let t = Timings {
            setup_s: t0.elapsed().as_secs_f64(),
            ..Default::default()
        };
        Ok(Self {
            opts,
            queue,
            dev: None,
            tables: None,
            expect_step: u32::MAX,
            dirty: true,
            days_hi: 0,
            t,
        })
    }

    pub fn timings(&self) -> &Timings {
        &self.t
    }

    fn mem(&self) -> Mem {
        if self.opts.private {
            Mem::Private
        } else {
            Mem::Shared
        }
    }

    /// Shared staging buffer filled from host memory, optionally blitted into a private one.
    fn upload<T>(&self, g: &Gpu, data: &[T], blit: Option<&BlitCommandEncoderRef>) -> Buffer {
        let bytes = (data.len().max(1) * std::mem::size_of::<T>()) as u64;
        let stage = g.device.new_buffer_with_data(
            slice_bytes(data),
            bytes.min(std::mem::size_of_val(data).max(1) as u64),
            MTLResourceOptions::StorageModeShared,
        );
        match (self.mem(), blit) {
            (Mem::Private, Some(b)) => {
                let dst = g
                    .device
                    .new_buffer(bytes, MTLResourceOptions::StorageModePrivate);
                b.copy_from_buffer(&stage, 0, &dst, 0, bytes);
                // The staging buffer must outlive the blit; the caller waits for completion
                // and the encoder retains it.
                dst
            }
            _ => stage,
        }
    }

    fn zeros(&self, g: &Gpu, bytes: usize, shared: bool) -> Buffer {
        let o = if shared || matches!(self.mem(), Mem::Shared) {
            MTLResourceOptions::StorageModeShared
        } else {
            MTLResourceOptions::StorageModePrivate
        };
        let b = g.device.new_buffer(bytes.max(4) as u64, o);
        if matches!(o, MTLResourceOptions::StorageModeShared) {
            // SAFETY: freshly allocated shared buffer of `bytes.max(4)` bytes.
            unsafe { std::ptr::write_bytes(b.contents().cast::<u8>(), 0, bytes.max(4)) };
        }
        b
    }

    fn vtables(&self, g: &Gpu, w: &World, cap: usize) -> (Buffer, Buffer) {
        // hosp[age][idx]: idx 0 = unvaccinated, idx d+1 = vaccinated d days ago.
        let stride = cap + 1;
        let mut hosp = vec![0u64; 5 * stride];
        let mut binf = vec![0u64; cap];
        for a in 0..5 {
            let p = w.disease.p_hospitalised[a];
            hosp[a * stride] = count_below(p * 1.0);
            for d in 0..cap {
                let b = 1.0 - w.vaccine.eff_severe(1, d as u16);
                hosp[a * stride + d + 1] = count_below(p * b);
            }
        }
        for (d, b) in binf.iter_mut().enumerate() {
            *b = (1.0 - w.vaccine.eff_infection(1, d as u16)).to_bits();
        }
        (
            g.device.new_buffer_with_data(
                slice_bytes(&hosp),
                (hosp.len() * 8) as u64,
                MTLResourceOptions::StorageModeShared,
            ),
            g.device.new_buffer_with_data(
                slice_bytes(&binf),
                (binf.len() * 8) as u64,
                MTLResourceOptions::StorageModeShared,
            ),
        )
    }

    fn upload_world(&mut self, w: &World) -> Result<(), String> {
        let t0 = Instant::now();
        let g = gpu()?;
        let n = w.agents.len();
        let nloc = w.layout.total();
        let a = &w.agents;
        let cmd = self.queue.new_command_buffer();
        let blit = cmd.new_blit_command_encoder();
        let id = self.upload(g, &a.id, Some(blit));
        let age = self.upload(g, &a.age_group, Some(blit));
        let st = self.upload(g, &a.state, Some(blit));
        let timer = self.upload(g, &a.timer, Some(blit));
        let home = self.upload(g, &a.home_loc, Some(blit));
        let work = self.upload(g, &a.work_loc, Some(blit));
        let vstat = self.upload(g, &a.vax_status, Some(blit));
        let vdays = self.upload(g, &a.vax_days, Some(blit));
        blit.end_encoding();
        cmd.commit();
        cmd.wait_until_completed();
        self.days_hi = (0..n)
            .filter(|&i| a.vax_status[i] != 0)
            .map(|i| u32::from(a.vax_days[i]))
            .max()
            .unwrap_or(0);
        let vcap = (self.days_hi as usize + 1 + 64)
            .next_power_of_two()
            .min(DAY_CAP);
        let (hosp, binf) = self.vtables(g, w, vcap);
        let tables = Tables::new(&w.disease, &w.vaccine)?;
        let dtab = g.device.new_buffer_with_data(
            slice_bytes(&tables.dtab),
            (tables.dtab.len() * 4) as u64,
            MTLResourceOptions::StorageModeShared,
        );
        self.tables = Some(tables);
        self.dev = Some(Dev {
            n,
            nloc,
            id,
            age,
            state: st,
            timer,
            home,
            work,
            vstat,
            vdays,
            loc: self.zeros(g, n * 4, false),
            occ: self.zeros(g, nloc * 4, false),
            inf_a: self.zeros(g, nloc * 4, false),
            inf_i: self.zeros(g, nloc * 4, false),
            lam_thr: Buf {
                shared: self.zeros(g, nloc * 8, true),
            },
            lam_bits: Buf {
                shared: self.zeros(g, nloc * 8, true),
            },
            active: Buf {
                shared: self.zeros(g, nloc * 16, true),
            },
            nactive: Buf {
                shared: self.zeros(g, 4, true),
            },
            stats: Buf {
                shared: self.zeros(g, 32, true),
            },
            dtab,
            vcap,
            hosp,
            binf,
        });
        self.dirty = false;
        self.expect_step = w.step;
        self.t.setup_s += t0.elapsed().as_secs_f64();
        Ok(())
    }

    fn dispatch(
        &self,
        enc: &ComputeCommandEncoderRef,
        pipe: &ComputePipelineStateRef,
        threads: usize,
    ) {
        enc.set_compute_pipeline_state(pipe);
        enc.dispatch_threads(
            MTLSize::new(threads.max(1) as u64, 1, 1),
            MTLSize::new(self.opts.threadgroup as u64, 1, 1),
        );
    }

    fn params(&self, w: &World, iv: &InterventionState) -> Params {
        let tb = self.tables.as_ref().expect("uploaded");
        let dev = self.dev.as_ref().expect("uploaded");
        let step = w.step;
        let day = step / STEPS_PER_DAY;
        let slot = step % STEPS_PER_DAY;
        let (mp_stay, at_work) = slot_plan(day, slot, true);
        let (mp_go, _) = slot_plan(day, slot, false);
        let l = &w.layout;
        let mut closed_mask = 0;
        for (k, &c) in iv.closed.iter().enumerate() {
            closed_mask |= u32::from(c) << k;
        }
        Params {
            thr_asym: tb.thr_asym,
            thr_death: tb.thr_death,
            mk_n: u64::from(l.markets.end - l.markets.start),
            hp_n: u64::from(l.hospitals.end - l.hospitals.start),
            nh: u64::from(l.homes.end.max(1)),
            mp_stay_thr: count_below(mp_stay),
            mp_go_thr: count_below(mp_go),
            iso_thr: count_below(iv.isolation_compliance),
            n: dev.n as u32,
            nloc: dev.nloc as u32,
            step,
            k0: w.seed as u32,
            k1: (w.seed >> 32) as u32,
            homes_end: l.homes.end,
            works_end: l.works.end,
            markets_end: l.markets.end,
            mk_start: l.markets.start,
            hp_start: l.hospitals.start,
            col_lo: l.colleges.start,
            col_hi: l.colleges.end,
            at_work: u32::from(at_work),
            mp_stay_pos: u32::from(mp_stay > 0.0),
            mp_go_pos: u32::from(mp_go > 0.0),
            iso_on: u32::from(iv.isolation_compliance > 0.0),
            closed_mask,
            col_closed: u32::from(iv.colleges_closed),
            day_end: u32::from(step % STEPS_PER_DAY == STEPS_PER_DAY - 1),
            vstride: dev.vcap as u32 + 1,
            dcap: tb.dcap,
            dcnt: tb.dcnt,
            don: tb.don,
            pad: 0,
        }
    }
}

/// Same table as `schedule::slot_plan` (market probability, at-work flag).
fn slot_plan(day: u32, slot: u32, stays_home: bool) -> (f64, bool) {
    use epidemicsim_core::schedule::{is_weekend, DAY, MORNING, NIGHT};
    let weekend = is_weekend(day);
    match (slot, weekend) {
        (NIGHT, _) => (0.0, false),
        (MORNING, false) => (if stays_home { 0.10 } else { 0.0 }, true),
        (MORNING, true) => (0.05, false),
        (DAY, false) => (if stays_home { 0.20 } else { 0.0 }, true),
        (DAY, true) => (0.30, false),
        (_, false) => (0.20, false),
        (_, true) => (0.30, false),
    }
}

fn set_params(enc: &ComputeCommandEncoderRef, p: &Params) {
    enc.set_bytes(
        0,
        std::mem::size_of::<Params>() as u64,
        std::ptr::from_ref(p).cast(),
    );
}

fn bufs(enc: &ComputeCommandEncoderRef, first: u64, list: &[&Buffer]) {
    for (k, b) in list.iter().enumerate() {
        enc.set_buffer(first + k as u64, Some(b), 0);
    }
}

#[allow(unexpected_cfgs)]
fn gpu_time(c: &CommandBufferRef) -> f64 {
    use ::metal::objc::{msg_send, sel, sel_impl};
    // SAFETY: GPUStartTime / GPUEndTime are `CFTimeInterval` (f64) properties of MTLCommandBuffer.
    let (a, b): (f64, f64) = unsafe { (msg_send![c, GPUStartTime], msg_send![c, GPUEndTime]) };
    (b - a).max(0.0)
}

impl Backend for MetalBackend {
    fn name(&self) -> &'static str {
        "metal"
    }

    fn step(&mut self, w: &mut World, iv: &InterventionState) -> StepStats {
        let t_step = Instant::now();
        let g = gpu().expect("metal available");
        let n = w.agents.len();
        if self.dirty || self.dev.as_ref().is_none_or(|d| d.n != n) || self.expect_step != w.step {
            self.upload_world(w).expect("metal upload");
        }
        // Rebuild the parameter tables if the model parameters changed.
        if self
            .tables
            .as_ref()
            .is_none_or(|t| t.sig != signature(&w.disease, &w.vaccine))
        {
            self.tables = Some(Tables::new(&w.disease, &w.vaccine).expect("metal tables"));
            let dev = self.dev.as_mut().expect("uploaded");
            dev.dtab = g.device.new_buffer_with_data(
                slice_bytes(&self.tables.as_ref().expect("set").dtab),
                (self.tables.as_ref().expect("set").dtab.len() * 4) as u64,
                MTLResourceOptions::StorageModeShared,
            );
            let (h, b) = self.vtables(g, w, self.dev.as_ref().expect("uploaded").vcap);
            let dev = self.dev.as_mut().expect("uploaded");
            dev.hosp = h;
            dev.binf = b;
        }
        // Vaccine-efficacy tables must cover every possible `vax_days` value.
        if self.days_hi as usize + 1 >= self.dev.as_ref().expect("uploaded").vcap
            && self.dev.as_ref().expect("uploaded").vcap < DAY_CAP
        {
            let cap = (self.dev.as_ref().expect("uploaded").vcap * 2).min(DAY_CAP);
            let (h, b) = self.vtables(g, w, cap);
            let dev = self.dev.as_mut().expect("uploaded");
            dev.vcap = cap;
            dev.hosp = h;
            dev.binf = b;
        }
        let p = self.params(w, iv);
        let dev = self.dev.as_ref().expect("uploaded");
        let step = w.step;

        // ---- command buffer 1: placement + per-location counts, active-list compaction ----
        // SAFETY (all `contents()` uses): shared buffers; the GPU is idle between command buffers.
        unsafe { *dev.nactive.shared.contents().cast::<u32>() = 0 };
        let c1 = self.queue.new_command_buffer();
        {
            let e = c1.new_compute_command_encoder();
            set_params(e, &p);
            bufs(
                e,
                1,
                &[
                    &dev.id, &dev.state, &dev.home, &dev.work, &dev.loc, &dev.occ, &dev.inf_a,
                    &dev.inf_i,
                ],
            );
            self.dispatch(e, &g.place, dev.n);
            e.end_encoding();
        }
        {
            let e = c1.new_compute_command_encoder();
            set_params(e, &p);
            bufs(
                e,
                1,
                &[
                    &dev.occ,
                    &dev.inf_a,
                    &dev.inf_i,
                    &dev.lam_thr.shared,
                    &dev.lam_bits.shared,
                    &dev.active.shared,
                    &dev.nactive.shared,
                ],
            );
            self.dispatch(e, &g.locs, dev.nloc);
            e.end_encoding();
        }
        c1.commit();
        c1.wait_until_completed();
        let k1 = gpu_time(c1);

        // ---- CPU stage: lambda (f64) for the active locations only ----
        let t_rb = Instant::now();
        // SAFETY: shared buffers, GPU idle; `na` entries were written by k_locs.
        let (_na, list) = unsafe {
            let na = *dev.nactive.shared.contents().cast::<u32>() as usize;
            (
                na,
                std::slice::from_raw_parts(dev.active.shared.contents().cast::<[u32; 4]>(), na),
            )
        };
        let readback1 = t_rb.elapsed().as_secs_f64();
        let t_cpu = Instant::now();
        let mut beta = w.disease.beta;
        for b in &mut beta {
            *b *= iv.beta_multiplier;
        }
        let rel_a = w.disease.rel_infectiousness_asymptomatic;
        let (mut sum_a, mut sum_i) = (0u64, 0u64);
        let thr = dev.lam_thr.shared.contents().cast::<u64>();
        let bits = dev.lam_bits.shared.contents().cast::<u64>();
        for &[l, a, i, o] in list {
            sum_a += u64::from(a);
            sum_i += u64::from(i);
            // Same expression and operand order as `kernels::lambda_chunk`.
            let infectious = f64::from(a) * rel_a + f64::from(i);
            if infectious > 0.0 {
                let kind = w.layout.kind_of(l) as usize;
                let occ = f64::from(o.max(1));
                let lam = 1.0 - (-beta[kind] * infectious / occ.sqrt()).exp();
                if lam > 0.0 {
                    // SAFETY: `l` < nloc; entries were zeroed by k_locs this step.
                    unsafe {
                        *thr.add(l as usize) = count_below(lam);
                        *bits.add(l as usize) = lam.to_bits();
                    }
                }
            }
        }
        let infectious_total = LamPlan::new(w, iv).total(sum_a, sum_i);
        let cpu_lambda = t_cpu.elapsed().as_secs_f64();

        // ---- command buffer 2: agent update + population statistics ----
        // SAFETY: shared stats buffer of 8 words.
        unsafe { std::ptr::write_bytes(dev.stats.shared.contents().cast::<u32>(), 0, 8) };
        let c2 = self.queue.new_command_buffer();
        {
            let e = c2.new_compute_command_encoder();
            set_params(e, &p);
            bufs(
                e,
                1,
                &[
                    &dev.id,
                    &dev.age,
                    &dev.state,
                    &dev.timer,
                    &dev.vstat,
                    &dev.vdays,
                    &dev.loc,
                    &dev.lam_thr.shared,
                    &dev.lam_bits.shared,
                    &dev.hosp,
                    &dev.binf,
                    &dev.dtab,
                    &dev.stats.shared,
                ],
            );
            self.dispatch(e, &g.update, dev.n);
            e.end_encoding();
        }
        {
            let e = c2.new_compute_command_encoder();
            set_params(e, &p);
            bufs(e, 1, &[&dev.state, &dev.stats.shared]);
            e.set_compute_pipeline_state(&g.stats);
            let tg = self.opts.threadgroup;
            let groups = dev.n.div_ceil(tg * 8).clamp(1, 1024);
            e.dispatch_thread_groups(
                MTLSize::new(groups as u64, 1, 1),
                MTLSize::new(tg as u64, 1, 1),
            );
            e.end_encoding();
        }
        c2.commit();
        c2.wait_until_completed();
        let k2 = gpu_time(c2);

        let t_rb2 = Instant::now();
        // SAFETY: 8 words written by the kernels; GPU idle.
        let words: [u32; 8] =
            unsafe { std::ptr::read(dev.stats.shared.contents().cast::<[u32; 8]>()) };
        let readback2 = t_rb2.elapsed().as_secs_f64();

        w.step += 1;
        self.expect_step = w.step;
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
        self.t.kernel_s += k1 + k2;
        self.t.cmd1_s += k1;
        self.t.cmd2_s += k2;
        self.t.cpu_lambda_s += cpu_lambda;
        self.t.readback_s += readback1 + readback2;
        self.t.step_wall_s += t_step.elapsed().as_secs_f64();
        self.t.steps += 1;
        StepStats {
            step,
            day: step / STEPS_PER_DAY,
            counts,
            new_infections,
            hospital_load: counts[state::H as usize],
            r_t,
        }
    }

    fn sync_to_host(&mut self, w: &mut World) {
        let t0 = Instant::now();
        let Some(dev) = self.dev.as_ref() else { return };
        if self.dirty || dev.n != w.agents.len() {
            return;
        }
        let g = gpu().expect("metal available");
        let n = dev.n;
        // Private buffers are copied to a shared staging buffer first.
        let fetch = |src: &Buffer, bytes: usize| -> Buffer {
            if matches!(self.mem(), Mem::Shared) {
                return src.clone();
            }
            let dst = g
                .device
                .new_buffer(bytes as u64, MTLResourceOptions::StorageModeShared);
            let c = self.queue.new_command_buffer();
            let b = c.new_blit_command_encoder();
            b.copy_from_buffer(src, 0, &dst, 0, bytes as u64);
            b.end_encoding();
            c.commit();
            c.wait_until_completed();
            dst
        };
        let st = fetch(&dev.state, n);
        let tm = fetch(&dev.timer, n * 2);
        let vd = fetch(&dev.vdays, n * 2);
        // SAFETY: the buffers hold `n` elements of the given types and the GPU is idle.
        unsafe {
            w.agents
                .state
                .copy_from_slice(std::slice::from_raw_parts(st.contents().cast::<u8>(), n));
            w.agents
                .timer
                .copy_from_slice(std::slice::from_raw_parts(tm.contents().cast::<u16>(), n));
            w.agents
                .vax_days
                .copy_from_slice(std::slice::from_raw_parts(vd.contents().cast::<u16>(), n));
        }
        self.t.readback_s += t0.elapsed().as_secs_f64();
    }

    fn host_changed(&mut self) {
        self.dirty = true;
    }
}

/// Run the exact-product test kernel (used by the unit test).
pub fn test_lt_prod(xs: &[u32], a: &[u64], b: &[u64]) -> Vec<bool> {
    let g = gpu().expect("metal available");
    let q = g.device.new_command_queue();
    let shared = MTLResourceOptions::StorageModeShared;
    let mk = |p: *const std::ffi::c_void, len: usize| {
        g.device.new_buffer_with_data(p, len as u64, shared)
    };
    let (bx, ba, bb) = (
        mk(slice_bytes(xs), xs.len() * 4),
        mk(slice_bytes(a), a.len() * 8),
        mk(slice_bytes(b), b.len() * 8),
    );
    let out = g.device.new_buffer(xs.len() as u64, shared);
    let c = q.new_command_buffer();
    let e = c.new_compute_command_encoder();
    e.set_compute_pipeline_state(&g.test_lt);
    bufs(e, 0, &[&bx, &ba, &bb, &out]);
    e.dispatch_threads(MTLSize::new(xs.len() as u64, 1, 1), MTLSize::new(64, 1, 1));
    e.end_encoding();
    c.commit();
    c.wait_until_completed();
    // SAFETY: `out` holds xs.len() bytes written by the kernel.
    unsafe { std::slice::from_raw_parts(out.contents().cast::<u8>(), xs.len()) }
        .iter()
        .map(|&v| v != 0)
        .collect()
}

/// Philox blocks computed on the GPU for the given counters.
pub fn test_philox(ctr: &[[u32; 4]], key: [u32; 2]) -> Vec<[u32; 4]> {
    let g = gpu().expect("metal available");
    let q = g.device.new_command_queue();
    let shared = MTLResourceOptions::StorageModeShared;
    let bin = g
        .device
        .new_buffer_with_data(slice_bytes(ctr), (ctr.len() * 16) as u64, shared);
    let bk = g.device.new_buffer_with_data(slice_bytes(&key), 8, shared);
    let out = g.device.new_buffer((ctr.len() * 16) as u64, shared);
    let c = q.new_command_buffer();
    let e = c.new_compute_command_encoder();
    e.set_compute_pipeline_state(&g.test_philox);
    bufs(e, 0, &[&bin, &bk, &out]);
    e.dispatch_threads(MTLSize::new(ctr.len() as u64, 1, 1), MTLSize::new(64, 1, 1));
    e.end_encoding();
    c.commit();
    c.wait_until_completed();
    // SAFETY: `out` holds ctr.len() x 16 bytes written by the kernel.
    unsafe { std::slice::from_raw_parts(out.contents().cast::<[u32; 4]>(), ctr.len()) }.to_vec()
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::rng::philox4x32_10;

    fn lcg(s: &mut u64) -> u64 {
        *s = s
            .wrapping_mul(6_364_136_223_846_793_005)
            .wrapping_add(1_442_695_040_888_963_407);
        *s >> 11
    }

    #[test]
    fn philox_matches_cpu() {
        if !available() {
            return;
        }
        let mut s = 7;
        let ctr: Vec<[u32; 4]> = (0..100_000)
            .map(|_| std::array::from_fn(|_| lcg(&mut s) as u32))
            .collect();
        let key = [0xa409_3822, 0x299f_31d0];
        let got = test_philox(&ctr, key);
        for (c, g) in ctr.iter().zip(got) {
            assert_eq!(philox4x32_10(*c, key), g);
        }
    }

    #[test]
    fn count_below_is_the_cpu_predicate() {
        for t in [0.0, 1e-12, 0.05, 0.1, 0.3, 0.7 * 0.9, 0.999_999_999_9, 1.0] {
            let c = count_below(t);
            if c > 0 {
                assert!(to_unit((c - 1) as u32) < t);
            }
            if c < (1 << 32) {
                assert!(to_unit(c as u32) >= t);
            }
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
            // Half the cases sit within +-2 of the exact decision boundary.
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
        let got = test_lt_prod(&xs, &a, &b);
        for k in 0..xs.len() {
            let want = to_unit(xs[k]) < f64::from_bits(a[k]) * f64::from_bits(b[k]);
            assert_eq!(want, got[k], "k={k}");
        }
    }
}
