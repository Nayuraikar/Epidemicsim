//! Frame-producing simulation: one scenario run on one backend, with a spatial infection grid,
//! hotspot attribution and (optional) infection-chain recording. Shared by live runs and `bake`.
use epidemicsim_backends::BackendOpts;
use epidemicsim_core::agents::state;
use epidemicsim_core::schedule;
use epidemicsim_core::world::{LocKind, STEPS_PER_DAY};
use epidemicsim_core::{Backend, InterventionState, StepStats, World};
use epidemicsim_data::io::Locations;
use epidemicsim_data::Meta;
use epidemicsim_scenarios::spec::Intervention;
use epidemicsim_scenarios::{Controller, Scenario};
use serde::Serialize;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};

const NONE: u32 = u32::MAX;
pub const ZONES: usize = 3;
pub const ZONE_NAMES: [&str; 9] = [
    "North-west",
    "North",
    "North-east",
    "West",
    "Centre",
    "East",
    "South-west",
    "South",
    "South-east",
];

/// Square-ish cells (~1 km) over the city bbox, plus a 3x3 grid of named zones for hotspots.
#[derive(Serialize)]
pub struct Grid {
    pub bbox: [f64; 4],
    pub cols: u32,
    pub rows: u32,
    pub zones: Vec<&'static str>,
    #[serde(skip)]
    pub cell_of_loc: Vec<u32>,
}

impl Grid {
    pub fn new(bbox: [f64; 4], locs: &Locations) -> Self {
        let [x0, y0, x1, y1] = bbox;
        let km_x = (x1 - x0) * 111.32 * ((y0 + y1) / 2.0).to_radians().cos();
        let km_y = (y1 - y0) * 110.57;
        let cols = (km_x.ceil() as u32).clamp(8, 64);
        let rows = (km_y.ceil() as u32).clamp(8, 64);
        let cell = |lon: f64, lat: f64| {
            let cx = (((lon - x0) / (x1 - x0)) * f64::from(cols)).clamp(0.0, f64::from(cols) - 1.0);
            let cy = (((y1 - lat) / (y1 - y0)) * f64::from(rows)).clamp(0.0, f64::from(rows) - 1.0);
            cy as u32 * cols + cx as u32
        };
        let cell_of_loc = (0..locs.lon.len())
            .map(|i| cell(locs.lon[i], locs.lat[i]))
            .collect();
        Self {
            bbox,
            cols,
            rows,
            zones: ZONE_NAMES.to_vec(),
            cell_of_loc,
        }
    }

    pub fn zone_of_cell(&self, c: u32) -> u32 {
        let (cx, cy) = (c % self.cols, c / self.cols);
        (cy * ZONES as u32 / self.rows) * ZONES as u32 + cx * ZONES as u32 / self.cols
    }
}

/// One simulation step as seen by the UI.
#[derive(Serialize, Clone)]
pub struct Frame {
    pub step: u32,
    pub day: u32,
    /// S, E, A, I, H, R, D
    pub counts: [u32; 7],
    pub new_infections: u32,
    pub hospital_load: u32,
    pub r_t: f64,
    /// Flat `[cell, agents, ...]`: agents in E/A/I/H by home cell (non-zero cells only).
    pub density: Vec<u32>,
    /// Flat `[zone, new_infections, ...]`: this step's infections by zone of the place of infection.
    pub hot: Vec<u32>,
    /// New infections by place kind: home, work/school, market, hospital.
    pub kinds: [u32; 4],
}

#[derive(Clone)]
struct Infection {
    step: u32,
    loc: u32,
    /// Co-located infectious agents at that step (possible sources); `NONE` = empty slot.
    cand: [u32; 3],
}

#[derive(Default)]
struct Chains {
    by_agent: HashMap<u32, Infection>,
    /// source candidate -> infectees (capped)
    downstream: HashMap<u32, Vec<u32>>,
}

/// Everything loaded for a city (shared across runs).
pub struct CityCtx {
    pub meta: Meta,
    pub grid: Arc<Grid>,
    pub locs: Locations,
    pub worlds: Mutex<HashMap<u32, Arc<World>>>,
}

pub struct SimSpec {
    pub days: u32,
    pub seed: u64,
    pub backend: String,
    pub threads: usize,
    pub scenario: Scenario,
    pub chains: bool,
}

pub struct Sim {
    world: World,
    backend: Box<dyn Backend + Send>,
    ctl: Controller,
    ctx: Arc<CityCtx>,
    has_vax: bool,
    last: Option<StepStats>,
    prev: Vec<u8>,
    chains: Option<Chains>,
    pub total_steps: u32,
    pub capacity: u32,
    pub subsample: u32,
    pub backend_name: &'static str,
}

impl Sim {
    pub fn new(
        ctx: Arc<CityCtx>,
        world: Arc<World>,
        subsample: u32,
        spec: &SimSpec,
    ) -> Result<Self, String> {
        let bopts = BackendOpts {
            threads: spec.threads.max(1),
            ..Default::default()
        };
        let backend = epidemicsim_backends::make_with(&spec.backend, &bopts)?;
        let backend_name = backend.name();
        let mut w = (*world).clone();
        let frac = epidemicsim_core::Config::default()
            .disease
            .initial_infected_frac;
        w.reseed(spec.seed, frac);
        let n = w.agents.len();
        let capacity = (ctx.meta.total_beds * n as f64 / ctx.meta.agents.max(1) as f64)
            .ceil()
            .max(1.0) as u32;
        let has_vax = spec
            .scenario
            .interventions
            .iter()
            .any(|i| matches!(i, Intervention::Vaccination { .. }));
        let mut chains = spec.chains.then(Chains::default);
        if let Some(c) = &mut chains {
            for i in 0..n {
                if w.agents.state[i] == state::E {
                    c.by_agent.insert(
                        i as u32,
                        Infection {
                            step: 0,
                            loc: w.agents.home_loc[i],
                            cand: [NONE; 3],
                        },
                    );
                }
            }
        }
        Ok(Self {
            prev: w.agents.state.clone(),
            ctl: Controller::new(&spec.scenario, n, capacity),
            world: w,
            backend,
            ctx,
            has_vax,
            last: None,
            chains,
            total_steps: spec.days * STEPS_PER_DAY,
            capacity,
            subsample,
            backend_name,
        })
    }

    pub fn agents(&self) -> usize {
        self.world.agents.len()
    }

    pub fn done(&self) -> bool {
        self.world.step >= self.total_steps
    }

    pub fn step(&mut self) -> Frame {
        let edits = self.has_vax && self.world.step.is_multiple_of(STEPS_PER_DAY);
        if edits {
            self.backend.sync_to_host(&mut self.world);
        }
        let iv: InterventionState = self.ctl.update(&mut self.world, self.last.as_ref()).clone();
        if edits {
            self.backend.host_changed();
        }
        // Possible sources, from start-of-step infectious agents (only when recording chains).
        let sources = self.chains.is_some().then(|| self.infectious_by_loc(&iv));
        let s = self.backend.step(&mut self.world, &iv);
        self.backend.sync_to_host(&mut self.world);
        let frame = self.frame(&s, &iv, sources);
        self.prev.copy_from_slice(&self.world.agents.state);
        self.last = Some(s);
        frame
    }

    fn infectious_by_loc(&self, iv: &InterventionState) -> HashMap<u32, [u32; 3]> {
        let w = &self.world;
        let step = w.step;
        let mut m: HashMap<u32, [u32; 3]> = HashMap::new();
        for i in 0..w.agents.len() {
            if matches!(w.agents.state[i], state::A | state::I) {
                let slots = m
                    .entry(schedule::location_of(w, i, step, iv))
                    .or_insert([NONE; 3]);
                if let Some(s) = slots.iter_mut().find(|s| **s == NONE) {
                    *s = i as u32;
                }
            }
        }
        m
    }

    fn frame(
        &mut self,
        s: &StepStats,
        iv: &InterventionState,
        sources: Option<HashMap<u32, [u32; 3]>>,
    ) -> Frame {
        let w = &self.world;
        let grid = &self.ctx.grid;
        let mut dens = vec![0u32; (grid.cols * grid.rows) as usize];
        let mut hot = [0u32; 9];
        let mut kinds = [0u32; 4];
        let mut new_ids = Vec::new();
        for i in 0..w.agents.len() {
            let st = w.agents.state[i];
            if matches!(st, state::E | state::A | state::I | state::H) {
                dens[grid.cell_of_loc[w.agents.home_loc[i] as usize] as usize] += 1;
            }
            if st == state::E && self.prev[i] == state::S {
                new_ids.push(i);
            }
        }
        for &i in &new_ids {
            let loc = schedule::location_of(w, i, s.step, iv);
            hot[grid.zone_of_cell(grid.cell_of_loc[loc as usize]) as usize] += 1;
            kinds[w.layout.kind_of(loc) as usize] += 1;
            if let (Some(c), Some(src)) = (&mut self.chains, &sources) {
                let cand = src.get(&loc).copied().unwrap_or([NONE; 3]);
                for &a in cand.iter().filter(|&&a| a != NONE) {
                    let v = c.downstream.entry(a).or_default();
                    if v.len() < 50 {
                        v.push(i as u32);
                    }
                }
                c.by_agent.insert(
                    i as u32,
                    Infection {
                        step: s.step,
                        loc,
                        cand,
                    },
                );
            }
        }
        Frame {
            step: s.step,
            day: s.day,
            counts: s.counts,
            new_infections: s.new_infections,
            hospital_load: s.hospital_load,
            r_t: s.r_t,
            density: dens
                .iter()
                .enumerate()
                .filter(|(_, &c)| c > 0)
                .flat_map(|(c, &n)| [c as u32, n])
                .collect(),
            hot: hot
                .iter()
                .enumerate()
                .filter(|(_, &c)| c > 0)
                .flat_map(|(z, &n)| [z as u32, n])
                .collect(),
            kinds,
        }
    }

    fn place(&self, loc: u32) -> serde_json::Value {
        let l = &self.ctx.locs;
        let k = self.world.layout.kind_of(loc);
        let kind = match k {
            LocKind::Home => "home",
            LocKind::Work => match l.subtype[loc as usize] {
                1 => "school",
                2 => "college",
                _ => "work",
            },
            LocKind::Market => "market",
            LocKind::Hospital => "hospital",
        };
        serde_json::json!({"loc": loc, "kind": kind, "lon": l.lon[loc as usize], "lat": l.lat[loc as usize]})
    }

    fn agent_brief(&self, id: u32) -> serde_json::Value {
        let a = &self.world.agents;
        serde_json::json!({"id": id, "state": state::NAMES[a.state[id as usize] as usize], "age_group": a.age_group[id as usize]})
    }

    /// Agent inspector: profile, baseline weekly schedule, and recorded infection chain.
    pub fn inspect(
        &self,
        agent: Option<u32>,
        cell: Option<u32>,
    ) -> Result<serde_json::Value, String> {
        let w = &self.world;
        let n = w.agents.len() as u32;
        let id = match (agent, cell) {
            (Some(a), _) if a < n => a,
            (_, Some(c)) => {
                let g = &self.ctx.grid;
                let in_cell = |i: usize| g.cell_of_loc[w.agents.home_loc[i] as usize] == c;
                let infected =
                    |i: usize| w.agents.state[i] != state::S && w.agents.state[i] != state::D;
                (0..n as usize)
                    .find(|&i| in_cell(i) && infected(i))
                    .or_else(|| (0..n as usize).find(|&i| in_cell(i)))
                    .ok_or("no agents in that cell")? as u32
            }
            _ => return Err("unknown agent".into()),
        };
        let i = id as usize;
        let a = &w.agents;
        let iv = InterventionState::default();
        let mut sched = Vec::new();
        for (label, day) in [("weekday", 0u32), ("weekend", 5u32)] {
            for (slot, name) in ["night", "morning", "day", "evening"].iter().enumerate() {
                let step = day * STEPS_PER_DAY + slot as u32;
                let mut p = self.place(schedule::location_of(w, i, step, &iv));
                p["day_type"] = label.into();
                p["slot"] = (*name).into();
                sched.push(p);
            }
        }
        let (mut chain, mut infection) = (Vec::new(), serde_json::Value::Null);
        if let Some(c) = &self.chains {
            let mut cur = id;
            for hop in 0..8 {
                let Some(inf) = c.by_agent.get(&cur) else {
                    break;
                };
                let srcs: Vec<_> = inf
                    .cand
                    .iter()
                    .filter(|&&x| x != NONE)
                    .map(|&x| self.agent_brief(x))
                    .collect();
                let mut entry = serde_json::json!({"agent": self.agent_brief(cur), "step": inf.step, "day": inf.step / STEPS_PER_DAY, "place": self.place(inf.loc), "sources": srcs});
                if hop == 0 {
                    infection = entry.clone();
                }
                entry["seed"] = (inf.cand[0] == NONE).into();
                chain.push(entry);
                if inf.cand[0] == NONE || inf.cand[0] == cur {
                    break;
                }
                cur = inf.cand[0];
            }
        }
        let others: Vec<_> = self
            .chains
            .as_ref()
            .and_then(|c| c.downstream.get(&id))
            .map(|v| v.iter().map(|&x| self.agent_brief(x)).collect())
            .unwrap_or_default();
        Ok(serde_json::json!({
            "id": id,
            "state": state::NAMES[a.state[i] as usize],
            "age_group": a.age_group[i],
            "vaccinated": a.vax_status[i] == 1,
            "home": self.place(a.home_loc[i]),
            "work": (a.work_loc[i] != a.home_loc[i]).then(|| self.place(a.work_loc[i])),
            "schedule": sched,
            "schedule_note": "baseline schedule, no interventions; hospitalised agents are shown at the hospital",
            "chains_recorded": self.chains.is_some(),
            "infection": infection,
            "chain": chain,
            "infected_others": others,
            "attribution_note": "sources are co-located infectious agents at the infection step (up to 3), not a unique infector",
        }))
    }
}
