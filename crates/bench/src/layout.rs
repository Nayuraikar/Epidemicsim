//! Memory-layout experiment on the CPU (single thread): the agent-update kernel on SoA vs AoS,
//! with agents in generated order, sorted by location, or shuffled. Same pure per-agent
//! function in every variant; order-independent checksums must match across all of them.
use crate::{hardware_tag, machine_info, thermal_note, timing, tlp::write_json, Machine, Timing};
use epidemicsim_core::agents::state as st;
use epidemicsim_core::params::DiseaseParams;
use epidemicsim_core::rng::{purpose, Philox};
use epidemicsim_core::world::duration_steps;
use epidemicsim_core::{synth, Config};
use serde::Serialize;
use std::{error::Error, path::Path, time::Instant};

#[derive(Clone, Copy)]
#[repr(C)]
struct Rec {
    id: u32,
    loc: u32,
    timer: u16,
    vdays: u16,
    state: u8,
    age: u8,
    vax: u8,
    _pad: u8,
}

#[derive(Clone)]
struct Soa {
    id: Vec<u32>,
    loc: Vec<u32>,
    timer: Vec<u16>,
    vdays: Vec<u16>,
    state: Vec<u8>,
    age: Vec<u8>,
    vax: Vec<u8>,
}

struct Ctx<'a> {
    philox: &'a Philox,
    d: &'a DiseaseParams,
    lambda: &'a [f64],
    step: u32,
}

/// Infection of susceptibles (random gather of the location's lambda) and progression of
/// infected agents: the same decisions as the simulation's update, with vaccine efficacy fixed.
#[inline(always)]
#[allow(clippy::too_many_arguments)]
fn step_one(
    c: &Ctx,
    id: u32,
    loc: u32,
    state: u8,
    timer: u16,
    age: u8,
    vax: u8,
    vdays: u16,
) -> (u8, u16) {
    let _ = vdays;
    match state {
        st::S => {
            let p = c.lambda[loc as usize];
            if p <= 0.0 {
                return (state, timer);
            }
            let ve = if vax != 0 { 0.5 } else { 0.0 };
            let u = c.philox.uniforms(id, c.step, purpose::INFECT);
            if u[0] < p * (1.0 - ve) {
                (st::E, duration_steps(c.d.latent_days, u[1]))
            } else {
                (state, timer)
            }
        }
        st::E | st::A | st::I | st::H => {
            let t = timer.saturating_sub(1);
            if t > 0 {
                return (state, t);
            }
            let u = c.philox.uniforms(id, c.step, purpose::TRANSITION);
            let a = age as usize;
            let (ns, days) = match state {
                st::E if u[0] < c.d.p_asymptomatic[a] => (st::A, c.d.asymptomatic_days),
                st::E => (st::I, c.d.symptomatic_days),
                st::A => (st::R, 0.0),
                st::I => {
                    let ve = if vax != 0 { 0.3 } else { 0.0 };
                    if u[0] < c.d.p_hospitalised[a] * (1.0 - ve) {
                        (st::H, c.d.hospital_days)
                    } else {
                        (st::R, 0.0)
                    }
                }
                _ => {
                    if u[0] < c.d.p_death_hospitalised[a] {
                        (st::D, 0.0)
                    } else {
                        (st::R, 0.0)
                    }
                }
            };
            (
                ns,
                if days > 0.0 {
                    duration_steps(days, u[1])
                } else {
                    0
                },
            )
        }
        _ => (state, timer),
    }
}

fn pass_soa(c: &Ctx, a: &mut Soa) {
    for i in 0..a.id.len() {
        let (s, t) = step_one(
            c, a.id[i], a.loc[i], a.state[i], a.timer[i], a.age[i], a.vax[i], a.vdays[i],
        );
        a.state[i] = s;
        a.timer[i] = t;
    }
}

fn pass_aos(c: &Ctx, a: &mut [Rec]) {
    for r in a.iter_mut() {
        let (s, t) = step_one(c, r.id, r.loc, r.state, r.timer, r.age, r.vax, r.vdays);
        r.state = s;
        r.timer = t;
    }
}

fn mix(id: u32, s: u8, t: u16) -> u64 {
    (u64::from(id) * 0x9E37_79B9_7F4A_7C15)
        ^ (u64::from(s) << 32 | u64::from(t)).wrapping_mul(0xD6E8_FEB8_6659_FD93)
}

struct Xor(u64);
impl Xor {
    fn next(&mut self) -> u64 {
        self.0 ^= self.0 << 13;
        self.0 ^= self.0 >> 7;
        self.0 ^= self.0 << 17;
        self.0
    }
}

#[derive(Serialize)]
pub struct Row {
    pub layout: &'static str,
    pub order: &'static str,
    /// Wall time of one pass over all agents.
    pub pass_s: Timing,
    pub ns_per_agent: f64,
    pub bytes_per_agent: usize,
}

#[derive(Serialize)]
pub struct LayoutReport {
    pub machine: Machine,
    pub agents: usize,
    pub locations: usize,
    pub passes_per_sample: usize,
    pub warmup: usize,
    pub reps: usize,
    pub threads: usize,
    pub thermal_before: Option<String>,
    pub thermal_after: Option<String>,
    pub checksums_match: bool,
    pub note: &'static str,
    pub rows: Vec<Row>,
}

pub fn run(
    out_dir: &Path,
    agents: usize,
    reps: usize,
) -> Result<std::path::PathBuf, Box<dyn Error>> {
    const PASSES: u32 = 4;
    let thermal_before = thermal_note();
    let w = synth::generate(&Config {
        agents,
        seed: 42,
        ..Default::default()
    });
    let nloc = w.layout.total();
    let mut rng = Xor(0x1234_5678_9ABC_DEF1);
    // Half of the locations have an infectious presence (lambda > 0).
    let lambda: Vec<f64> = (0..nloc)
        .map(|_| {
            let r = rng.next();
            if r & 1 == 0 {
                (r >> 11) as f64 / (1u64 << 53) as f64 * 0.02
            } else {
                0.0
            }
        })
        .collect();
    let n = w.agents.len();
    let mut base: Vec<Rec> = (0..n)
        .map(|i| {
            let h = rng.next();
            let state = match h % 100 {
                0..=4 => st::E,
                5..=9 => st::A,
                10..=16 => st::I,
                17..=19 => st::H,
                20..=29 => st::R,
                _ => st::S,
            };
            Rec {
                id: w.agents.id[i],
                loc: if i % 2 == 0 {
                    w.agents.home_loc[i]
                } else {
                    w.agents.work_loc[i]
                },
                timer: if matches!(state, st::E | st::A | st::I | st::H) {
                    1 + ((h >> 8) % 12) as u16
                } else {
                    0
                },
                vdays: 0,
                state,
                age: w.agents.age_group[i],
                vax: u8::from((h >> 20).is_multiple_of(4)),
                _pad: 0,
            }
        })
        .collect();
    let philox = Philox::new(42);
    let d = DiseaseParams::default();
    let mut rows = Vec::new();
    let mut sums = Vec::new();
    for order in ["generated", "sorted_by_location", "shuffled"] {
        match order {
            "sorted_by_location" => base.sort_by_key(|r| r.loc),
            "shuffled" => {
                for i in (1..n).rev() {
                    base.swap(i, (rng.next() % (i as u64 + 1)) as usize);
                }
            }
            _ => {}
        }
        for layout in ["soa", "aos"] {
            let mut samples = Vec::new();
            let mut sum = 0u64;
            for rep in 0..reps + 1 {
                let mut soa = Soa {
                    id: base.iter().map(|r| r.id).collect(),
                    loc: base.iter().map(|r| r.loc).collect(),
                    timer: base.iter().map(|r| r.timer).collect(),
                    vdays: base.iter().map(|r| r.vdays).collect(),
                    state: base.iter().map(|r| r.state).collect(),
                    age: base.iter().map(|r| r.age).collect(),
                    vax: base.iter().map(|r| r.vax).collect(),
                };
                let mut aos = base.clone();
                let t0 = Instant::now();
                for s in 0..PASSES {
                    let c = Ctx {
                        philox: &philox,
                        d: &d,
                        lambda: &lambda,
                        step: s,
                    };
                    if layout == "soa" {
                        pass_soa(&c, &mut soa);
                    } else {
                        pass_aos(&c, &mut aos);
                    }
                }
                let secs = t0.elapsed().as_secs_f64() / f64::from(PASSES);
                if rep > 0 {
                    samples.push(secs);
                }
                sum = if layout == "soa" {
                    (0..n).fold(0u64, |a, i| {
                        a.wrapping_add(mix(soa.id[i], soa.state[i], soa.timer[i]))
                    })
                } else {
                    aos.iter()
                        .fold(0u64, |a, r| a.wrapping_add(mix(r.id, r.state, r.timer)))
                };
            }
            let t = timing(&samples);
            eprintln!(
                "layout {layout} {order}: median {:.2} ms/pass +-{:.2} (95% CI)",
                t.median * 1e3,
                t.ci95 * 1e3
            );
            sums.push(sum);
            rows.push(Row {
                layout: if layout == "soa" { "soa" } else { "aos" },
                order: match order {
                    "generated" => "generated",
                    "sorted_by_location" => "sorted_by_location",
                    _ => "shuffled",
                },
                ns_per_agent: t.median * 1e9 / n as f64,
                pass_s: t,
                bytes_per_agent: if layout == "soa" {
                    15
                } else {
                    std::mem::size_of::<Rec>()
                },
            });
        }
    }
    let machine = machine_info();
    // Order does not change the multiset of (id, state, timer): all six checksums are equal.
    let report = LayoutReport {
        agents: n,
        locations: nloc,
        passes_per_sample: PASSES as usize,
        warmup: 1,
        reps,
        threads: 1,
        thermal_before,
        thermal_after: thermal_note(),
        checksums_match: sums.windows(2).all(|p| p[0] == p[1]),
        note: "Update kernel = infection of susceptibles (random gather of lambda by location, Philox) + progression of E/A/I/H, 4 steps per sample from the same initial state (data rebuilt untimed each rep). SoA touches 15 B/agent in 7 arrays, AoS a 16 B record. Single thread.",
        rows,
        machine: machine.clone(),
    };
    let path = out_dir.join(format!("layout_{}.json", hardware_tag(&machine)));
    write_json(&path, &report)?;
    Ok(path)
}
