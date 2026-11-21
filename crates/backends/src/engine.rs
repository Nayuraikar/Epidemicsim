//! The step skeleton shared by the ILP, rayon and SIMD backends: three parallel regions
//! (placement, per-location force of infection, agent update) over a pluggable executor.
use crate::kernels::*;
use crate::par::{Par, Phase, SharedMut};
use crate::simd::{self, SimdKind};
use epidemicsim_core::agents::state::{self, COUNT};
use epidemicsim_core::agents::Agents;
use epidemicsim_core::backend::{Backend, InterventionState, StepStats};
use epidemicsim_core::world::{World, STEPS_PER_DAY};
use std::sync::atomic::{AtomicU32, AtomicU64, Ordering::Relaxed};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Variant {
    Naive,
    Optimised,
    Simd(SimdKind),
}

pub struct Engine<P: Par> {
    pub par: P,
    pub variant: Variant,
    name: &'static str,
    loc: Vec<u32>,
    acc: LocAcc,
    lambda: Vec<f64>,
    tables: Option<Tables>,
}

impl<P: Par> Engine<P> {
    pub fn new(par: P, variant: Variant, name: &'static str) -> Self {
        Self {
            par,
            variant,
            name,
            loc: Vec::new(),
            acc: LocAcc::new(0),
            lambda: Vec::new(),
            tables: None,
        }
    }
}

impl<P: Par> Backend for Engine<P> {
    fn name(&self) -> &'static str {
        self.name
    }

    fn step(&mut self, w: &mut World, iv: &InterventionState) -> StepStats {
        let n = w.agents.len();
        let nloc = w.layout.total();
        if self.loc.len() != n {
            self.loc = vec![NONE; n];
        }
        if self.lambda.len() != nloc {
            self.lambda = vec![0.0; nloc];
            self.acc = LocAcc::new(nloc);
        }
        if !self
            .tables
            .as_ref()
            .is_some_and(|t| t.matches(&w.disease, &w.vaccine))
        {
            self.tables = Some(Tables::new(&w.disease, &w.vaccine));
        }
        let step = w.step;
        let variant = self.variant;
        // Single-threaded executors can use plain load/store accumulators.
        let atomic = !self.par.serial();

        // P1: place agents, count occupancy / infectious per location.
        {
            let plan = PlacePlan::new(w, iv, atomic);
            let wr: &World = w;
            let acc = &self.acc;
            let loc_sh = SharedMut::new(&mut self.loc);
            self.par.run(Phase::Place, n, &|_t, r| {
                // SAFETY: the executor hands each index range to exactly one thread.
                let out = unsafe { loc_sh.slice(r.clone()) };
                match variant {
                    Variant::Naive => place_naive(wr, iv, r.start, r.end, out, acc, atomic),
                    _ => place_optimised(&plan, &wr.agents, r.start, r.end, out, acc),
                }
            });
        }

        // P2: per-location contact processing (also clears the accumulators).
        let lam_plan = LamPlan::new(w, iv);
        let sums = [AtomicU64::new(0), AtomicU64::new(0)];
        {
            let acc = &self.acc;
            let lam_sh = SharedMut::new(&mut self.lambda);
            self.par.run(Phase::Location, nloc, &|_t, r| {
                // SAFETY: disjoint location ranges.
                let out = unsafe { lam_sh.slice(r.clone()) };
                let (a, i) = lambda_chunk(&lam_plan, acc, r.start, out);
                sums[0].fetch_add(a, Relaxed);
                sums[1].fetch_add(i, Relaxed);
            });
        }
        let infectious_total = lam_plan.total(sums[0].load(Relaxed), sums[1].load(Relaxed));

        // P3: progression + new infections + vaccine clock + state counts, fused per agent.
        let new_inf = AtomicU32::new(0);
        let counts: [AtomicU32; COUNT] = std::array::from_fn(|_| AtomicU32::new(0));
        {
            let tables = self.tables.as_ref().expect("built above");
            let World {
                agents,
                disease,
                vaccine,
                philox,
                seed,
                ..
            } = &mut *w;
            let Agents {
                id,
                age_group,
                state: st,
                timer,
                vax_status,
                vax_days,
                ..
            } = agents;
            let plan = UpdPlan {
                step,
                key: philox_key(*seed),
                philox: *philox,
                lambda: &self.lambda,
                d: disease,
                v: vaccine,
                t: tables,
                day_end: step % STEPS_PER_DAY == STEPS_PER_DAY - 1,
            };
            let (id, age_group, vax_status) = (&id[..], &age_group[..], &vax_status[..]);
            let loc = &self.loc[..];
            let st_sh = SharedMut::new(st);
            let tm_sh = SharedMut::new(timer);
            let vd_sh = SharedMut::new(vax_days);
            self.par.run(Phase::Update, n, &|_t, r| {
                // SAFETY: disjoint agent ranges; read-only slices are never written.
                let mut c = unsafe {
                    Chunk {
                        id: &id[r.clone()],
                        age: &age_group[r.clone()],
                        vstat: &vax_status[r.clone()],
                        loc: &loc[r.clone()],
                        state: st_sh.slice(r.clone()),
                        timer: tm_sh.slice(r.clone()),
                        vdays: vd_sh.slice(r.clone()),
                    }
                };
                let (cnt, ni) = match variant {
                    Variant::Naive => update_naive(&plan, &mut c),
                    Variant::Optimised => update_optimised(&plan, &mut c),
                    Variant::Simd(k) => simd::update(k, &plan, &mut c),
                };
                new_inf.fetch_add(ni, Relaxed);
                for (a, v) in counts.iter().zip(cnt) {
                    a.fetch_add(v, Relaxed);
                }
            });
        }
        w.step += 1;

        let new_infections = new_inf.load(Relaxed);
        let counts: [u32; COUNT] = std::array::from_fn(|s| counts[s].load(Relaxed));
        let r_t = if infectious_total > 0.0 {
            f64::from(new_infections) / infectious_total * w.infectious_steps_per_case
        } else {
            0.0
        };
        StepStats {
            step,
            day: step / STEPS_PER_DAY,
            counts,
            new_infections,
            hospital_load: counts[state::H as usize],
            r_t,
        }
    }
}
