//! Per-agent / per-location kernels shared by the ILP, rayon and SIMD backends.
//!
//! Every kernel works on a chunk `[lo, hi)` so a scheduler can hand out arbitrary disjoint
//! ranges. Results are bit-identical to `SequentialBackend` (checked by the golden tests).
//!
//! Two flavours of the two hot agent kernels exist so the effect of instruction-level
//! parallelism can be measured:
//! * `*_naive`: straightforward per-agent code, re-deriving everything per agent.
//! * `*_optimised`: loop-invariants hoisted into a plan, 4-agent unrolling with four
//!   independent Philox chains interleaved, lookup tables instead of per-agent math, and
//!   branch-free selects where the data allows.
use epidemicsim_core::agents::state::{self, COUNT};
use epidemicsim_core::agents::Agents;
use epidemicsim_core::backend::InterventionState;
use epidemicsim_core::params::{DiseaseParams, VaccineParams};
use epidemicsim_core::rng::{purpose, to_unit, Philox};
use epidemicsim_core::schedule::{self, DAY, MORNING, NIGHT};
use epidemicsim_core::world::{duration_steps, World, STEPS_PER_DAY};
use std::sync::atomic::{AtomicU32, Ordering::Relaxed};

pub const NONE: u32 = u32::MAX;
/// Vaccine-efficacy lookup tables cover this many days; later days fall back to the formula.
const VLUT: usize = 1024;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Ilp {
    Naive,
    Optimised,
}

// ---------------------------------------------------------------------------------------
// Philox x4: four independent blocks interleaved (mirror of core::rng::philox4x32_10).
// ---------------------------------------------------------------------------------------
const M0: u32 = 0xD251_1F53;
const M1: u32 = 0xCD9E_8D57;
const W0: u32 = 0x9E37_79B9;
const W1: u32 = 0xBB67_AE85;

/// Four Philox4x32-10 blocks at once. `c[word][lane]`. The four lanes are independent
/// dependency chains, so an out-of-order core can overlap their 10 serial rounds.
#[inline(always)]
pub fn philox_x4(mut c: [[u32; 4]; 4], key: [u32; 2]) -> [[u32; 4]; 4] {
    let (mut k0, mut k1) = (key[0], key[1]);
    for r in 0..10 {
        if r > 0 {
            k0 = k0.wrapping_add(W0);
            k1 = k1.wrapping_add(W1);
        }
        let mut n = [[0u32; 4]; 4];
        for l in 0..4 {
            let p0 = u64::from(M0) * u64::from(c[0][l]);
            let p1 = u64::from(M1) * u64::from(c[2][l]);
            n[0][l] = ((p1 >> 32) as u32) ^ c[1][l] ^ k0;
            n[1][l] = p1 as u32;
            n[2][l] = ((p0 >> 32) as u32) ^ c[3][l] ^ k1;
            n[3][l] = p0 as u32;
        }
        c = n;
    }
    c
}

pub fn philox_key(seed: u64) -> [u32; 2] {
    [seed as u32, (seed >> 32) as u32]
}

// ---------------------------------------------------------------------------------------
// Per-location accumulators.
// ---------------------------------------------------------------------------------------
/// Occupancy and infectious counts per location. `inf_a`/`inf_i` count asymptomatic and
/// symptomatic infectious agents separately so the infectious weight is computed once per
/// location (exact for the default dyadic relative infectiousness 0.5).
pub struct LocAcc {
    pub occ: Vec<AtomicU32>,
    pub inf_a: Vec<AtomicU32>,
    pub inf_i: Vec<AtomicU32>,
}

fn zeros(n: usize) -> Vec<AtomicU32> {
    (0..n).map(|_| AtomicU32::new(0)).collect()
}

impl LocAcc {
    pub fn new(n: usize) -> Self {
        Self {
            occ: zeros(n),
            inf_a: zeros(n),
            inf_i: zeros(n),
        }
    }
}

/// Increment a counter. `atomic = true`: a real atomic RMW (needed when several threads can
/// hit the same location). `atomic = false` (single thread): a plain load+store, which avoids
/// the cost of the atomic instruction and is correct only because nothing else touches it.
#[inline(always)]
fn bump(v: &[AtomicU32], l: usize, atomic: bool) {
    if atomic {
        v[l].fetch_add(1, Relaxed);
    } else {
        v[l].store(v[l].load(Relaxed) + 1, Relaxed);
    }
}

// ---------------------------------------------------------------------------------------
// P1: placement (where is each agent this step?) + occupancy / infectious accumulation.
// ---------------------------------------------------------------------------------------
fn slot_plan(day: u32, slot: u32, stays_home: bool) -> (f64, bool) {
    let weekend = schedule::is_weekend(day);
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

/// Everything about a step that does not depend on the agent (hoisted out of the loops).
#[derive(Clone)]
pub struct PlacePlan {
    pub step: u32,
    pub philox: Philox,
    pub key: [u32; 2],
    mp_stay: f64,
    mp_go: f64,
    any_market: bool,
    at_work: bool,
    homes_end: u32,
    works_end: u32,
    markets_end: u32,
    mk_start: u32,
    mk_n: u64,
    hp_start: u32,
    hp_n: u64,
    nh: u64,
    closed: [bool; 4],
    col_closed: bool,
    col_lo: u32,
    col_hi: u32,
    iso: f64,
    pub atomic: bool,
}

impl PlacePlan {
    pub fn new(w: &World, iv: &InterventionState, atomic: bool) -> Self {
        let day = w.step / STEPS_PER_DAY;
        let slot = w.step % STEPS_PER_DAY;
        let (mp_stay, at_work) = slot_plan(day, slot, true);
        let (mp_go, _) = slot_plan(day, slot, false);
        let l = &w.layout;
        Self {
            step: w.step,
            philox: w.philox,
            key: philox_key(w.seed),
            mp_stay,
            mp_go,
            any_market: mp_stay > 0.0 || mp_go > 0.0,
            at_work,
            homes_end: l.homes.end,
            works_end: l.works.end,
            markets_end: l.markets.end,
            mk_start: l.markets.start,
            mk_n: u64::from(l.markets.end - l.markets.start),
            hp_start: l.hospitals.start,
            hp_n: u64::from(l.hospitals.end - l.hospitals.start),
            nh: u64::from(l.homes.end.max(1)),
            closed: iv.closed,
            col_closed: iv.colleges_closed,
            col_lo: l.colleges.start,
            col_hi: l.colleges.end,
            iso: iv.isolation_compliance,
            atomic,
        }
    }

    #[inline(always)]
    fn market_for(&self, home: u32) -> u32 {
        self.mk_start + (u64::from(home) * self.mk_n / self.nh) as u32
    }

    #[inline(always)]
    fn hospital_for(&self, home: u32) -> u32 {
        self.hp_start + (u64::from(home) * self.hp_n / self.nh) as u32
    }
}

/// Naive placement: one call to the generic `schedule::location_of` per agent.
#[inline(never)]
pub fn place_naive(
    w: &World,
    iv: &InterventionState,
    lo: usize,
    hi: usize,
    out: &mut [u32],
    acc: &LocAcc,
    atomic: bool,
) {
    let step = w.step;
    for i in lo..hi {
        let st = w.agents.state[i];
        if st == state::D {
            out[i - lo] = NONE;
            continue;
        }
        let l = schedule::location_of(w, i, step, iv);
        out[i - lo] = l;
        bump(&acc.occ, l as usize, atomic);
        match st {
            state::A => bump(&acc.inf_a, l as usize, atomic),
            state::I => bump(&acc.inf_i, l as usize, atomic),
            _ => {}
        }
    }
}

/// Location of one agent given its (possibly precomputed) schedule uniform `u`.
#[inline(always)]
fn lane_loc(p: &PlacePlan, id: u32, s: u8, home: u32, work: u32, need: bool, u: f64) -> u32 {
    let mp = if work == home { p.mp_stay } else { p.mp_go };
    let visit = need & (u < mp);
    let l0 = if visit {
        p.market_for(home)
    } else if p.at_work {
        work
    } else {
        home
    };
    let mut l = l0;
    if s == state::I && p.iso > 0.0 && p.philox.uniform(id, 0, purpose::ISOLATE) < p.iso {
        l = home;
    } else {
        // Location kind as a sum of comparisons (no branches, no range-table walk).
        let kind = usize::from(l0 >= p.homes_end)
            + usize::from(l0 >= p.works_end)
            + usize::from(l0 >= p.markets_end);
        let col = p.col_closed & (l0 >= p.col_lo) & (l0 < p.col_hi);
        if p.closed[kind] | col {
            l = home;
        }
    }
    if s == state::H {
        l = p.hospital_for(home);
    }
    if s == state::D {
        l = NONE;
    }
    l
}

#[inline(always)]
fn account(acc: &LocAcc, s: u8, l: u32, atomic: bool) {
    if s != state::D {
        bump(&acc.occ, l as usize, atomic);
        if s == state::A {
            bump(&acc.inf_a, l as usize, atomic);
        } else if s == state::I {
            bump(&acc.inf_i, l as usize, atomic);
        }
    }
}

/// Optimised placement: plan hoisted, 4 agents per iteration with interleaved Philox chains.
#[inline(never)]
pub fn place_optimised(
    p: &PlacePlan,
    a: &Agents,
    lo: usize,
    hi: usize,
    out: &mut [u32],
    acc: &LocAcc,
) {
    let id = &a.id[lo..hi];
    let st = &a.state[lo..hi];
    let home = &a.home_loc[lo..hi];
    let work = &a.work_loc[lo..hi];
    let n = hi - lo;
    let n4 = n & !3;
    let mut b = 0;
    while b < n4 {
        let mut need = [false; 4];
        let mut any = false;
        for k in 0..4 {
            let mp = if work[b + k] == home[b + k] {
                p.mp_stay
            } else {
                p.mp_go
            };
            need[k] = (mp > 0.0) & (st[b + k] != state::D) & (st[b + k] != state::H);
            any |= need[k];
        }
        let mut u = [0.0f64; 4];
        // Skip Philox entirely when no lane can visit a market (night slots, closed days).
        if p.any_market & any {
            let r = philox_x4(
                [
                    [id[b], id[b + 1], id[b + 2], id[b + 3]],
                    [p.step; 4],
                    [purpose::SCHEDULE; 4],
                    [0; 4],
                ],
                p.key,
            );
            for k in 0..4 {
                u[k] = to_unit(r[0][k]);
            }
        }
        for k in 0..4 {
            let i = b + k;
            let l = lane_loc(p, id[i], st[i], home[i], work[i], need[k], u[k]);
            out[i] = l;
            account(acc, st[i], l, p.atomic);
        }
        b += 4;
    }
    for i in n4..n {
        let s = st[i];
        let mp = if work[i] == home[i] {
            p.mp_stay
        } else {
            p.mp_go
        };
        let need = (mp > 0.0) & (s != state::D) & (s != state::H);
        let u = if need {
            p.philox.uniform(id[i], p.step, purpose::SCHEDULE)
        } else {
            0.0
        };
        let l = lane_loc(p, id[i], s, home[i], work[i], need, u);
        out[i] = l;
        account(acc, s, l, p.atomic);
    }
}

// ---------------------------------------------------------------------------------------
// P2: per-location contact processing -> force of infection.
// ---------------------------------------------------------------------------------------
pub struct LamPlan {
    /// Transmission coefficient by location kind, already times the intervention multiplier.
    beta: [f64; 4],
    rel_a: f64,
    homes_end: u32,
    works_end: u32,
    markets_end: u32,
}

impl LamPlan {
    pub fn new(w: &World, iv: &InterventionState) -> Self {
        let mut beta = w.disease.beta;
        for b in &mut beta {
            *b *= iv.beta_multiplier;
        }
        Self {
            beta,
            rel_a: w.disease.rel_infectiousness_asymptomatic,
            homes_end: w.layout.homes.end,
            works_end: w.layout.works.end,
            markets_end: w.layout.markets.end,
        }
    }

    /// Infectious weight of the whole population from summed A / I counts.
    pub fn total(&self, sum_a: u64, sum_i: u64) -> f64 {
        sum_a as f64 * self.rel_a + sum_i as f64
    }
}

/// Lambda for locations `lo..lo+out.len()`; also resets the accumulators for the next step
/// (so no separate zeroing pass is needed). Returns the chunk's summed A and I counts.
#[inline(never)]
pub fn lambda_chunk(p: &LamPlan, acc: &LocAcc, lo: usize, out: &mut [f64]) -> (u64, u64) {
    let (mut sa, mut si) = (0u64, 0u64);
    for (j, o) in out.iter_mut().enumerate() {
        let l = lo + j;
        let a = acc.inf_a[l].load(Relaxed);
        let i = acc.inf_i[l].load(Relaxed);
        let infectious = f64::from(a) * p.rel_a + f64::from(i);
        if infectious > 0.0 {
            let kind = usize::from(l as u32 >= p.homes_end)
                + usize::from(l as u32 >= p.works_end)
                + usize::from(l as u32 >= p.markets_end);
            let occ = f64::from(acc.occ[l].load(Relaxed).max(1));
            *o = 1.0 - (-p.beta[kind] * infectious / occ.sqrt()).exp();
        } else {
            *o = 0.0;
        }
        sa += u64::from(a);
        si += u64::from(i);
        acc.inf_a[l].store(0, Relaxed);
        acc.inf_i[l].store(0, Relaxed);
        acc.occ[l].store(0, Relaxed);
    }
    (sa, si)
}

// ---------------------------------------------------------------------------------------
// P3: agent update (progression of infected agents + new infections + vaccine clock + counts).
// ---------------------------------------------------------------------------------------
/// Lookup tables: per-age probabilities, duration means in steps, vaccine efficacy by day.
pub struct Tables {
    pub p_asym: [f64; 5],
    pub p_hosp: [f64; 5],
    pub p_death: [f64; 5],
    pub asym4: f64,
    pub sym4: f64,
    pub hosp4: f64,
    pub latent4: f64,
    ve_inf: Vec<f64>,
    ve_sev: Vec<f64>,
    sig: [f64; 4],
}

fn vsig(v: &VaccineParams) -> [f64; 4] {
    [
        v.ve_infection,
        v.ve_severe,
        v.ramp_days,
        v.waning_half_life_days,
    ]
}

impl Tables {
    pub fn new(d: &DiseaseParams, v: &VaccineParams) -> Self {
        let s4 = f64::from(STEPS_PER_DAY);
        Self {
            p_asym: d.p_asymptomatic,
            p_hosp: d.p_hospitalised,
            p_death: d.p_death_hospitalised,
            asym4: d.asymptomatic_days * s4,
            sym4: d.symptomatic_days * s4,
            hosp4: d.hospital_days * s4,
            latent4: d.latent_days * s4,
            ve_inf: (0..VLUT).map(|k| v.eff_infection(1, k as u16)).collect(),
            ve_sev: (0..VLUT).map(|k| v.eff_severe(1, k as u16)).collect(),
            sig: vsig(v),
        }
    }

    /// True while the tables still describe `d`/`v` (cheap check done once per step).
    pub fn matches(&self, d: &DiseaseParams, v: &VaccineParams) -> bool {
        let s4 = f64::from(STEPS_PER_DAY);
        self.sig == vsig(v)
            && self.p_asym == d.p_asymptomatic
            && self.p_hosp == d.p_hospitalised
            && self.p_death == d.p_death_hospitalised
            && self.asym4 == d.asymptomatic_days * s4
            && self.sym4 == d.symptomatic_days * s4
            && self.hosp4 == d.hospital_days * s4
            && self.latent4 == d.latent_days * s4
    }

    #[inline(always)]
    pub fn ve_inf(&self, v: &VaccineParams, status: u8, days: u16) -> f64 {
        if status == 0 {
            0.0
        } else if (days as usize) < VLUT {
            self.ve_inf[days as usize]
        } else {
            v.eff_infection(1, days)
        }
    }

    #[inline(always)]
    pub fn ve_sev(&self, v: &VaccineParams, status: u8, days: u16) -> f64 {
        if status == 0 {
            0.0
        } else if (days as usize) < VLUT {
            self.ve_sev[days as usize]
        } else {
            v.eff_severe(1, days)
        }
    }
}

/// Step-wide inputs of the agent update.
pub struct UpdPlan<'a> {
    pub step: u32,
    pub key: [u32; 2],
    pub philox: Philox,
    pub lambda: &'a [f64],
    pub d: &'a DiseaseParams,
    pub v: &'a VaccineParams,
    pub t: &'a Tables,
    /// True on the last step of a day (vaccine clocks tick).
    pub day_end: bool,
}

/// One chunk of agents (all slices have the same length; `state`/`timer`/`vdays` are written).
pub struct Chunk<'a> {
    pub id: &'a [u32],
    pub age: &'a [u8],
    pub vstat: &'a [u8],
    pub loc: &'a [u32],
    pub state: &'a mut [u8],
    pub timer: &'a mut [u16],
    pub vdays: &'a mut [u16],
}

pub type UpdResult = ([u32; COUNT], u32);

/// Naive agent update: a direct, branchy port of the sequential steps 3-5.
#[inline(never)]
pub fn update_naive(p: &UpdPlan, c: &mut Chunk) -> UpdResult {
    let n = c.state.len();
    let step = p.step;
    for i in 0..n {
        let st = c.state[i];
        if matches!(st, state::S | state::R | state::D) {
            continue;
        }
        let t = c.timer[i].saturating_sub(1);
        if t > 0 {
            c.timer[i] = t;
            continue;
        }
        let u = p.philox.uniforms(c.id[i], step, purpose::TRANSITION);
        let age = c.age[i] as usize;
        let d = p.d;
        let (new_state, mean_days) = match st {
            state::E if u[0] < d.p_asymptomatic[age] => (state::A, d.asymptomatic_days),
            state::E => (state::I, d.symptomatic_days),
            state::A => (state::R, 0.0),
            state::I => {
                let ve = p.v.eff_severe(c.vstat[i], c.vdays[i]);
                if u[0] < d.p_hospitalised[age] * (1.0 - ve) {
                    (state::H, d.hospital_days)
                } else {
                    (state::R, 0.0)
                }
            }
            _ => {
                if u[0] < d.p_death_hospitalised[age] {
                    (state::D, 0.0)
                } else {
                    (state::R, 0.0)
                }
            }
        };
        c.state[i] = new_state;
        c.timer[i] = if mean_days > 0.0 {
            duration_steps(mean_days, u[1])
        } else {
            0
        };
    }
    let mut new_inf = 0u32;
    for i in 0..n {
        if c.state[i] != state::S {
            continue;
        }
        let pr = p.lambda[c.loc[i] as usize];
        if pr <= 0.0 {
            continue;
        }
        let ve = p.v.eff_infection(c.vstat[i], c.vdays[i]);
        let u = p.philox.uniforms(c.id[i], step, purpose::INFECT);
        if u[0] < pr * (1.0 - ve) {
            c.state[i] = state::E;
            c.timer[i] = duration_steps(p.d.latent_days, u[1]);
            new_inf += 1;
        }
    }
    if p.day_end {
        for i in 0..n {
            if c.vstat[i] != 0 {
                c.vdays[i] = c.vdays[i].saturating_add(1);
            }
        }
    }
    let mut counts = [0u32; COUNT];
    for &s in c.state.iter() {
        counts[s as usize] += 1;
    }
    (counts, new_inf)
}

#[inline(always)]
pub fn dur(mean4: f64, u: f64) -> u16 {
    (mean4 * (0.5 + u)).round().max(1.0) as u16
}

#[inline(always)]
fn dur_opt(mean4: f64, u: f64) -> u16 {
    if mean4 > 0.0 {
        dur(mean4, u)
    } else {
        0
    }
}

/// Apply the infection decision for candidate `i` given its two uniforms.
#[inline(always)]
pub fn apply_infect(p: &UpdPlan, c: &mut Chunk, i: usize, u0: f64, u1: f64) -> bool {
    let lam = p.lambda[c.loc[i] as usize];
    let ve = p.t.ve_inf(p.v, c.vstat[i], c.vdays[i]);
    if u0 < lam * (1.0 - ve) {
        c.state[i] = state::E;
        c.timer[i] = dur(p.t.latent4, u1);
        true
    } else {
        false
    }
}

/// Apply the progression decision (states E, I, H) for agent `i` given its two uniforms.
#[inline(always)]
pub fn apply_trans(p: &UpdPlan, c: &mut Chunk, i: usize, u0: f64, u1: f64) {
    let age = c.age[i] as usize;
    let t = p.t;
    let (ns, tm) = match c.state[i] {
        state::E => {
            if u0 < t.p_asym[age] {
                (state::A, dur_opt(t.asym4, u1))
            } else {
                (state::I, dur_opt(t.sym4, u1))
            }
        }
        state::I => {
            let ve = t.ve_sev(p.v, c.vstat[i], c.vdays[i]);
            if u0 < t.p_hosp[age] * (1.0 - ve) {
                (state::H, dur_opt(t.hosp4, u1))
            } else {
                (state::R, 0)
            }
        }
        _ => {
            if u0 < t.p_death[age] {
                (state::D, 0)
            } else {
                (state::R, 0)
            }
        }
    };
    c.state[i] = ns;
    c.timer[i] = tm;
}

/// Up to four pending Philox jobs (agent index, stream purpose), flushed four at a time.
struct Pending {
    items: [(usize, u32); 4],
    n: usize,
}

impl Pending {
    #[inline(always)]
    fn push(&mut self, p: &UpdPlan, c: &mut Chunk, i: usize, purp: u32, new_inf: &mut u32) {
        self.items[self.n] = (i, purp);
        self.n += 1;
        if self.n == 4 {
            self.flush_philox_batch(p, c, new_inf);
        }
    }

    #[inline(always)]
    fn flush_philox_batch(&mut self, p: &UpdPlan, c: &mut Chunk, new_inf: &mut u32) {
        flush_batch(&self.items, self.n, p, c, new_inf);
        self.n = 0;
    }
}

/// Run the pending Philox jobs: four interleaved chains when the batch is full, scalar
/// otherwise. Not inlined: it is called from several push sites and holds the unrolled rounds.
#[inline(never)]
pub fn flush_batch(
    it: &[(usize, u32); 4],
    n: usize,
    p: &UpdPlan,
    c: &mut Chunk,
    new_inf: &mut u32,
) {
    if n == 4 {
        let r = philox_x4(
            [
                [c.id[it[0].0], c.id[it[1].0], c.id[it[2].0], c.id[it[3].0]],
                [p.step; 4],
                [it[0].1, it[1].1, it[2].1, it[3].1],
                [0; 4],
            ],
            p.key,
        );
        for k in 0..4 {
            finish_item(p, c, it[k], to_unit(r[0][k]), to_unit(r[1][k]), new_inf);
        }
    } else {
        for &(i, purp) in &it[..n] {
            let b = p.philox.block(c.id[i], p.step, purp);
            finish_item(p, c, (i, purp), to_unit(b[0]), to_unit(b[1]), new_inf);
        }
    }
}

#[inline(always)]
fn finish_item(p: &UpdPlan, c: &mut Chunk, it: (usize, u32), u0: f64, u1: f64, new_inf: &mut u32) {
    if it.1 == purpose::INFECT {
        *new_inf += u32::from(apply_infect(p, c, it.0, u0, u1));
    } else {
        apply_trans(p, c, it.0, u0, u1);
    }
}

/// Classification flags: bit 0 = infection candidate, bit 1 = infected (progressing).
#[inline(always)]
fn classify(p: &UpdPlan, c: &Chunk, i: usize) -> u8 {
    let s = c.state[i];
    // Only susceptible agents have a valid location; others index slot 0 (result unused).
    let li = if s == state::S { c.loc[i] as usize } else { 0 };
    let cand = (s == state::S) & (p.lambda[li] > 0.0);
    let act = s.wrapping_sub(1) < 4;
    u8::from(cand) | (u8::from(act) << 1)
}

#[inline(always)]
fn scan_one(p: &UpdPlan, c: &mut Chunk, i: usize, f: u8, pend: &mut Pending, ni: &mut u32) {
    if f & 1 != 0 {
        pend.push(p, c, i, purpose::INFECT, ni);
    } else if f & 2 != 0 {
        let t = c.timer[i].saturating_sub(1);
        if t > 0 {
            c.timer[i] = t;
        } else if c.state[i] == state::A {
            // A -> R needs no random number: skip the Philox call the naive code makes.
            c.state[i] = state::R;
            c.timer[i] = 0;
        } else {
            pend.push(p, c, i, purpose::TRANSITION, ni);
        }
    }
}

/// Vaccine clock tick + state histogram with four independent sub-histograms (avoids the
/// store-to-load dependency of a single `counts[s] += 1` chain).
#[inline(always)]
pub fn finish_chunk(p: &UpdPlan, c: &mut Chunk) -> [u32; COUNT] {
    let n = c.state.len();
    if p.day_end {
        for i in 0..n {
            c.vdays[i] = c.vdays[i].saturating_add(u16::from(c.vstat[i] != 0));
        }
    }
    let mut h = [[0u32; COUNT]; 4];
    let n4 = n & !3;
    let mut b = 0;
    while b < n4 {
        for k in 0..4 {
            h[k][c.state[b + k] as usize] += 1;
        }
        b += 4;
    }
    for i in n4..n {
        h[0][c.state[i] as usize] += 1;
    }
    let mut counts = [0u32; COUNT];
    for s in 0..COUNT {
        counts[s] = h[0][s] + h[1][s] + h[2][s] + h[3][s];
    }
    counts
}

/// Optimised agent update: single classification pass over 4 agents at a time, Philox work
/// batched four-wide, lookup tables, then one finishing pass (vaccine clock + counts).
#[inline(never)]
pub fn update_optimised(p: &UpdPlan, c: &mut Chunk) -> UpdResult {
    let n = c.state.len();
    let mut pend = Pending {
        items: [(0, 0); 4],
        n: 0,
    };
    let mut new_inf = 0u32;
    let n4 = n & !3;
    let mut b = 0;
    while b < n4 {
        let mut f = [0u8; 4];
        for k in 0..4 {
            f[k] = classify(p, c, b + k);
        }
        // Branch reduction: one test per 4 agents skips the (common) all-idle group.
        if (f[0] | f[1] | f[2] | f[3]) != 0 {
            for k in 0..4 {
                scan_one(p, c, b + k, f[k], &mut pend, &mut new_inf);
            }
        }
        b += 4;
    }
    for i in n4..n {
        let f = classify(p, c, i);
        scan_one(p, c, i, f, &mut pend, &mut new_inf);
    }
    pend.flush_philox_batch(p, c, &mut new_inf);
    let counts = finish_chunk(p, c);
    (counts, new_inf)
}

#[cfg(test)]
mod tests {
    use super::*;
    use epidemicsim_core::rng::philox4x32_10;

    #[test]
    fn philox_x4_matches_scalar_lane_by_lane() {
        let key = [0xa409_3822, 0x299f_31d0];
        let mut c = [[0u32; 4]; 4];
        for (w, row) in c.iter_mut().enumerate() {
            for (l, x) in row.iter_mut().enumerate() {
                *x =
                    (w as u32 + 1).wrapping_mul(0x9e37_79b1) ^ (l as u32).wrapping_mul(0x85eb_ca6b);
            }
        }
        let r = philox_x4(c, key);
        for l in 0..4 {
            let s = philox4x32_10([c[0][l], c[1][l], c[2][l], c[3][l]], key);
            assert_eq!([r[0][l], r[1][l], r[2][l], r[3][l]], s);
        }
    }
}
