//! Synthetic random-city generator (deterministic given the seed).
use crate::agents::Agents;
use crate::params::{Config, N_AGE};
use crate::rng::{purpose, Philox};
use crate::world::{Layout, World};

fn pick(cdf: &[f64], u: f64) -> usize {
    cdf.iter().position(|&c| u < c).unwrap_or(cdf.len() - 1)
}

fn count(x: f64) -> u32 {
    x.round().max(1.0) as u32
}

pub fn generate(cfg: &Config) -> World {
    let n = cfg.agents;
    let sp = &cfg.synth;
    let ph = Philox::new(cfg.seed);

    // Households: consecutive agents share a home.
    let mut home = Vec::with_capacity(n);
    let mut hh = 0u32;
    while home.len() < n {
        let size = pick(&sp.household_size_cdf, ph.uniform(hh, 0, purpose::SYNTH_HH)) + 1;
        for _ in 0..size.min(n - home.len()) {
            home.push(hh);
        }
        hh += 1;
    }
    let n_homes = hh;

    // Location counts from expected students and workers.
    let mut pmf = [0.0; N_AGE];
    let mut prev = 0.0;
    for g in 0..N_AGE {
        pmf[g] = sp.age_cdf[g] - prev;
        prev = sp.age_cdf[g];
    }
    let nf = n as f64;
    let students = nf * pmf[0] * sp.employment[0];
    let workers: f64 = (1..N_AGE).map(|g| nf * pmf[g] * sp.employment[g]).sum();
    let n_schools = count(students / sp.school_size);
    let n_workplaces = count(workers / sp.workplace_size);
    let layout = Layout::new(
        n_homes,
        n_schools + n_workplaces,
        count(nf / sp.agents_per_market),
        count(nf / sp.agents_per_hospital),
    );
    let school0 = layout.works.start;
    let work0 = school0 + n_schools;

    let mut agents = Agents::with_capacity(n);
    for (i, &h) in home.iter().enumerate() {
        let u = ph.uniforms(i as u32, 0, purpose::SYNTH_AGENT);
        let age = pick(&sp.age_cdf, u[0]);
        let work = if u[1] < sp.employment[age] {
            if age == 0 {
                school0 + (u[2] * f64::from(n_schools)) as u32
            } else {
                work0 + (u[2] * f64::from(n_workplaces)) as u32
            }
        } else {
            h
        };
        let (vax, days) = if u[3] < sp.initial_vax_frac {
            let d = ph.uniform(i as u32, 0, purpose::SYNTH_VAX);
            (1, (d * 365.0) as u16)
        } else {
            (0, 0)
        };
        agents.push(age as u8, h, work, vax, days);
    }

    let mut world = World::new(
        cfg.seed,
        agents,
        layout,
        cfg.disease.clone(),
        cfg.vaccine.clone(),
    );
    world.seed_infections(cfg.disease.initial_infected_frac);
    world
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn generates_requested_agents_deterministically() {
        let cfg = Config {
            agents: 5000,
            ..Default::default()
        };
        let a = generate(&cfg);
        let b = generate(&cfg);
        assert_eq!(a.agents.len(), 5000);
        assert_eq!(a.agents.home_loc, b.agents.home_loc);
        assert_eq!(a.agents.work_loc, b.agents.work_loc);
        assert!(a.agents.counts()[1] >= 1);
        let max_loc = a.layout.total() as u32;
        assert!(a.agents.work_loc.iter().all(|&l| l < max_loc));
    }
}
