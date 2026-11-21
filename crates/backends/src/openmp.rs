//! `openmp` backend: the whole step runs in C (`kernels-c/openmp/kernels.c`) through FFI.
use crate::par::{PhaseStat, Schedule, ThreadStat, PHASE_NAMES};
use epidemicsim_core::agents::state;
use epidemicsim_core::backend::{Backend, InterventionState, StepStats};
use epidemicsim_core::world::{World, STEPS_PER_DAY};
use epidemicsim_kernels_c as kc;

/// Days covered by the vaccine-efficacy tables handed to C (later days use a C fallback).
const LUT: usize = 4096;

pub fn available() -> bool {
    kc::HAS_OPENMP
}

pub struct OpenMpBackend {
    threads: usize,
    schedule: Schedule,
    chunk: usize,
    loc: Vec<u32>,
    occ: Vec<u32>,
    inf_a: Vec<u32>,
    inf_i: Vec<u32>,
    lambda: Vec<f64>,
    ve_inf: Vec<f64>,
    ve_sev: Vec<f64>,
    work_s: Vec<f64>,
    items: Vec<i64>,
}

impl OpenMpBackend {
    pub fn new(threads: usize, schedule: Schedule, chunk: usize) -> Result<Self, String> {
        if !kc::HAS_OPENMP {
            return Err(
                "openmp backend unavailable: libomp was not found at build time \
                        (brew install libomp, then rebuild)"
                    .into(),
            );
        }
        kc::check_layout();
        Ok(Self {
            threads: threads.clamp(1, kc::MAX_THREADS),
            schedule,
            chunk: if chunk == 0 {
                schedule.default_chunk()
            } else {
                chunk
            },
            loc: Vec::new(),
            occ: Vec::new(),
            inf_a: Vec::new(),
            inf_i: Vec::new(),
            lambda: Vec::new(),
            ve_inf: Vec::new(),
            ve_sev: Vec::new(),
            work_s: vec![0.0; kc::PHASES * kc::MAX_THREADS],
            items: vec![0; kc::PHASES * kc::MAX_THREADS],
        })
    }

    pub fn reset_stats(&mut self) {
        self.work_s.fill(0.0);
        self.items.fill(0);
    }

    pub fn stats(&self) -> Vec<PhaseStat> {
        (0..kc::PHASES)
            .map(|p| PhaseStat {
                phase: PHASE_NAMES[p],
                threads: (0..self.threads)
                    .map(|t| ThreadStat {
                        thread: t,
                        work_s: self.work_s[p * kc::MAX_THREADS + t],
                        items: self.items[p * kc::MAX_THREADS + t] as u64,
                    })
                    .collect(),
            })
            .collect()
    }
}

impl Backend for OpenMpBackend {
    fn name(&self) -> &'static str {
        "openmp"
    }

    fn step(&mut self, w: &mut World, iv: &InterventionState) -> StepStats {
        let n = w.agents.len();
        let nloc = w.layout.total();
        if self.loc.len() != n {
            self.loc = vec![u32::MAX; n];
        }
        if self.lambda.len() != nloc {
            self.lambda = vec![0.0; nloc];
            self.occ = vec![0; nloc];
            self.inf_a = vec![0; nloc];
            self.inf_i = vec![0; nloc];
        }
        // Rebuild the efficacy tables if the vaccine parameters are new to this backend.
        if self.ve_inf.is_empty() {
            self.ve_inf = (0..LUT)
                .map(|d| w.vaccine.eff_infection(1, d as u16))
                .collect();
            self.ve_sev = (0..LUT)
                .map(|d| w.vaccine.eff_severe(1, d as u16))
                .collect();
        }
        let step = w.step;
        let (d, v, l) = (&w.disease, &w.vaccine, &w.layout);
        let mut beta = d.beta;
        for b in &mut beta {
            *b *= iv.beta_multiplier;
        }
        let key = crate::kernels::philox_key(w.seed);
        let a = &mut w.agents;
        let mut c = kc::EpCtx {
            n: n as i64,
            nloc: nloc as i64,
            id: a.id.as_ptr(),
            age: a.age_group.as_ptr(),
            state: a.state.as_mut_ptr(),
            timer: a.timer.as_mut_ptr(),
            home: a.home_loc.as_ptr(),
            work: a.work_loc.as_ptr(),
            vstat: a.vax_status.as_ptr(),
            vdays: a.vax_days.as_mut_ptr(),
            loc: self.loc.as_mut_ptr(),
            occ: self.occ.as_mut_ptr(),
            inf_a: self.inf_a.as_mut_ptr(),
            inf_i: self.inf_i.as_mut_ptr(),
            lambda: self.lambda.as_mut_ptr(),
            ve_inf_lut: self.ve_inf.as_ptr(),
            ve_sev_lut: self.ve_sev.as_ptr(),
            lut_n: LUT as i64,
            work_s: self.work_s.as_mut_ptr(),
            items: self.items.as_mut_ptr(),
            chunk: self.chunk as i64,
            nh: u64::from(l.homes.end.max(1)),
            mk_n: u64::from(l.markets.end - l.markets.start),
            hp_n: u64::from(l.hospitals.end - l.hospitals.start),
            sum_a: 0,
            sum_i: 0,
            beta,
            rel_a: d.rel_infectiousness_asymptomatic,
            p_asym: d.p_asymptomatic,
            p_hosp: d.p_hospitalised,
            p_death: d.p_death_hospitalised,
            latent_days: d.latent_days,
            asym_days: d.asymptomatic_days,
            sym_days: d.symptomatic_days,
            hosp_days: d.hospital_days,
            ve_inf: v.ve_infection,
            ve_sev: v.ve_severe,
            ramp_days: v.ramp_days,
            half_life: v.waning_half_life_days,
            iso: iv.isolation_compliance,
            step,
            day: step / STEPS_PER_DAY,
            slot: step % STEPS_PER_DAY,
            key0: key[0],
            key1: key[1],
            day_end: u32::from(step % STEPS_PER_DAY == STEPS_PER_DAY - 1),
            homes_end: l.homes.end,
            works_end: l.works.end,
            markets_end: l.markets.end,
            mk_start: l.markets.start,
            hp_start: l.hospitals.start,
            col_lo: l.colleges.start,
            col_hi: l.colleges.end,
            sched: match self.schedule {
                Schedule::Static => 0,
                Schedule::Dynamic => 1,
                Schedule::Guided => 2,
            },
            nthreads: self.threads as i32,
            counts: [0; 7],
            new_inf: 0,
            closed: iv.closed.map(u8::from),
            col_closed: u8::from(iv.colleges_closed),
            pad: [0; 3],
        };
        // SAFETY: all pointers refer to live buffers of the sizes recorded in `c`; the C code
        // only touches indices < n / nloc and synchronises concurrent writes itself.
        unsafe { kc::ep_step(&mut c) };
        w.step += 1;

        let infectious_total = c.sum_a as f64 * d.rel_infectiousness_asymptomatic + c.sum_i as f64;
        let r_t = if infectious_total > 0.0 {
            f64::from(c.new_inf) / infectious_total * w.infectious_steps_per_case
        } else {
            0.0
        };
        StepStats {
            step,
            day: step / STEPS_PER_DAY,
            counts: c.counts,
            new_infections: c.new_inf,
            hospital_load: c.counts[state::H as usize],
            r_t,
        }
    }
}
