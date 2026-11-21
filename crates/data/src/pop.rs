//! Synthetic population: households, ages, gravity-model workplaces/schools/colleges.
use crate::city::CityConfig;
use crate::geo::Bbox;
use crate::io::*;
use crate::osm::{Pois, Site};
use crate::weight::PopWeight;
use epidemicsim_core::{params::SynthParams, rng::Philox};
use std::collections::{HashMap, HashSet};

const P_HH_SIZE: u32 = 101;
const P_HH_PLACE: u32 = 102;
const P_AGENT: u32 = 103;
const P_TOPUP: u32 = 104;
/// Share of age-group-1 (18-39) workers that are college students.
const COLLEGE_FRAC: f64 = 0.2;
const COLLEGE_SIZE: f64 = 1000.0;
/// Gravity distance scale (km): weight = 1 / (1 + (d/D0)^2).
const D0_KM: f64 = 3.0;
const HOME_CELL_KM: f64 = 1.0;
const MARKET_CELL_KM: f64 = 0.5;

pub struct City {
    pub locations: Locations,
    pub agents: AgentRows,
    pub meta: Meta,
}

fn pick(cdf: &[f64], u: f64) -> usize {
    cdf.partition_point(|&c| c <= u).min(cdf.len() - 1)
}

fn prefix(w: impl Iterator<Item = f64>) -> Vec<f64> {
    let mut acc = 0.0;
    w.map(|x| {
        acc += x;
        acc
    })
    .collect()
}

/// Keep one site per `cell_km` grid cell.
fn thin(sites: Vec<Site>, bbox: &Bbox, cell_km: f64) -> Vec<Site> {
    let mut seen = HashSet::new();
    sites
        .into_iter()
        .filter(|s| {
            let (x, y) = bbox.to_km(s.lon, s.lat);
            seen.insert(((x / cell_km).floor() as i64, (y / cell_km).floor() as i64))
        })
        .collect()
}

#[derive(Clone, Copy, PartialEq)]
enum Dest {
    None,
    School,
    College,
    Office,
}

pub fn build(
    cfg: &CityConfig,
    mut pois: Pois,
    weighter: &dyn PopWeight,
    sp: &SynthParams,
) -> Result<City, Box<dyn std::error::Error>> {
    if pois.homes.is_empty() {
        return Err("no residential buildings found in bbox".into());
    }
    let bbox = cfg.bbox();
    let seed = cfg.seed.unwrap_or(42);
    let ph = Philox::new(seed);
    let n = cfg.target_agents;

    // 1. Households placed on buildings, weighted by the population-weighting trait.
    let mut w = weighter.weights(&pois.homes);
    if w.iter().sum::<f64>() <= 0.0 {
        w = vec![1.0; pois.homes.len()];
    }
    let bcdf = prefix(w.iter().copied());
    let btotal = *bcdf.last().unwrap();
    let mut hh: Vec<(usize, u8)> = Vec::new(); // (building, size)
    let mut placed = 0;
    while placed < n {
        let k = hh.len() as u32;
        let size = (pick(&sp.household_size_cdf, ph.uniform(k, 0, P_HH_SIZE)) + 1).min(n - placed);
        let b = pick(&bcdf, ph.uniform(k, 0, P_HH_PLACE) * btotal);
        hh.push((b, size as u8));
        placed += size;
    }
    // Spatial (Hilbert) order, so home ids and the market/hospital proportional mapping are local.
    hh.sort_by_key(|&(b, _)| bbox.hilbert(pois.homes[b].lon, pois.homes[b].lat));
    let n_homes = hh.len() as u32;

    // 2. Agents in household order: age and destination type.
    let mut age = Vec::with_capacity(n);
    let mut home_loc = Vec::with_capacity(n);
    for (h, &(_, size)) in hh.iter().enumerate() {
        for _ in 0..size {
            home_loc.push(h as u32);
        }
    }
    let mut dest = Vec::with_capacity(n);
    let mut u3 = Vec::with_capacity(n);
    let mut need = [0usize; 3];
    for i in 0..n {
        let u = ph.uniforms(i as u32, 0, P_AGENT);
        let g = pick(&sp.age_cdf, u[0]);
        age.push(g as u8);
        let d = if u[1] >= sp.employment[g] {
            Dest::None
        } else if g == 0 {
            Dest::School
        } else if g == 1 && u[2] < COLLEGE_FRAC {
            Dest::College
        } else {
            Dest::Office
        };
        match d {
            Dest::School => need[0] += 1,
            Dest::College => need[1] += 1,
            Dest::Office => need[2] += 1,
            Dest::None => {}
        }
        dest.push(d);
        u3.push(u[3]);
    }

    // 3. Sites: thin markets, top up categories OSM under-maps, order spatially.
    pois.markets = thin(std::mem::take(&mut pois.markets), &bbox, MARKET_CELL_KM);
    let nf = n as f64;
    let want = |x: f64, size: f64| ((x / size).ceil() as usize).max(1);
    let mut synthetic = Counts::default();
    let topup = |sites: &mut Vec<Site>, min: usize, code: u32| -> u32 {
        let before = sites.len();
        for j in 0..min.saturating_sub(before) {
            let b = pick(&bcdf, ph.uniform(j as u32, code, P_TOPUP) * btotal);
            let h = pois.homes[b];
            sites.push(Site { weight: 1.0, ..h });
        }
        (sites.len() - before) as u32
    };
    synthetic.schools = topup(&mut pois.schools, want(need[0] as f64, sp.school_size), 1);
    synthetic.colleges = topup(&mut pois.colleges, want(need[1] as f64, COLLEGE_SIZE), 2);
    synthetic.offices = topup(
        &mut pois.offices,
        want(need[2] as f64, sp.workplace_size),
        3,
    );
    synthetic.markets = topup(&mut pois.markets, want(nf, sp.agents_per_market), 4);
    synthetic.hospitals = topup(&mut pois.hospitals, want(nf, sp.agents_per_hospital), 5);
    // Synthetic sites are appended last; remember how many per category before sorting.
    let n_syn = [
        synthetic.schools,
        synthetic.colleges,
        synthetic.offices,
        synthetic.markets,
        synthetic.hospitals,
    ];
    let flag = |sites: &mut Vec<Site>, syn: u32| -> Vec<u8> {
        let first_syn = sites.len() - syn as usize;
        let mut tagged: Vec<(Site, u8)> = sites
            .iter()
            .enumerate()
            .map(|(i, s)| (*s, u8::from(i >= first_syn)))
            .collect();
        tagged.sort_by_key(|(s, _)| bbox.hilbert(s.lon, s.lat));
        *sites = tagged.iter().map(|t| t.0).collect();
        tagged.iter().map(|t| t.1).collect()
    };
    let f_school = flag(&mut pois.schools, n_syn[0]);
    let f_college = flag(&mut pois.colleges, n_syn[1]);
    let f_office = flag(&mut pois.offices, n_syn[2]);
    let f_market = flag(&mut pois.markets, n_syn[3]);
    let f_hosp = flag(&mut pois.hospitals, n_syn[4]);

    let (ns, nc, no) = (pois.schools.len(), pois.colleges.len(), pois.offices.len());
    let work0 = n_homes;
    let off = [work0, work0 + ns as u32, work0 + (ns + nc) as u32];

    // 4. Gravity-model assignment, grouped by 1 km home cell (CDFs computed once per cell).
    let home_km: Vec<(f64, f64)> = hh
        .iter()
        .map(|&(b, _)| bbox.to_km(pois.homes[b].lon, pois.homes[b].lat))
        .collect();
    let mut cells: HashMap<(i64, i64), Vec<u32>> = HashMap::new();
    for (i, &h) in home_loc.iter().enumerate() {
        if dest[i] != Dest::None {
            let (x, y) = home_km[h as usize];
            cells
                .entry((
                    (x / HOME_CELL_KM).floor() as i64,
                    (y / HOME_CELL_KM).floor() as i64,
                ))
                .or_default()
                .push(i as u32);
        }
    }
    let km =
        |s: &[Site]| -> Vec<(f64, f64)> { s.iter().map(|p| bbox.to_km(p.lon, p.lat)).collect() };
    let site_km = [km(&pois.schools), km(&pois.colleges), km(&pois.offices)];
    let mut work_loc: Vec<u32> = home_loc.clone();
    let mut assigned: HashMap<u32, u32> = HashMap::new();
    let mut keys: Vec<_> = cells.keys().copied().collect();
    keys.sort_unstable();
    for key in keys {
        let (cx, cy) = (
            (key.0 as f64 + 0.5) * HOME_CELL_KM,
            (key.1 as f64 + 0.5) * HOME_CELL_KM,
        );
        let mut cdfs: [Option<Vec<f64>>; 3] = [None, None, None];
        for &i in &cells[&key] {
            let c = match dest[i as usize] {
                Dest::School => 0,
                Dest::College => 1,
                Dest::Office => 2,
                Dest::None => continue,
            };
            let cdf = cdfs[c].get_or_insert_with(|| {
                prefix(site_km[c].iter().map(|&(x, y)| {
                    let d = ((x - cx).powi(2) + (y - cy).powi(2)).sqrt() / D0_KM;
                    1.0 / (1.0 + d * d)
                }))
            });
            let j = pick(cdf, u3[i as usize] * cdf.last().unwrap());
            let loc = off[c] + j as u32;
            work_loc[i as usize] = loc;
            *assigned.entry(loc).or_default() += 1;
        }
    }

    // 5. Location table: homes, work (schools, colleges, offices), markets, hospitals.
    let beds = cfg.hospital_beds_per_1000 * nf / 1000.0;
    let mut loc = Locations::default();
    let mut add = |k: u8, sub: u8, lon: f64, lat: f64, size: f32, syn: u8| {
        loc.kind.push(k);
        loc.subtype.push(sub);
        loc.lon.push(lon);
        loc.lat.push(lat);
        loc.size.push(size);
        loc.synthetic.push(syn);
    };
    let mut residents = vec![0f32; n_homes as usize];
    for &h in &home_loc {
        residents[h as usize] += 1.0;
    }
    for (h, &(b, _)) in hh.iter().enumerate() {
        add(
            KIND_HOME,
            0,
            pois.homes[b].lon,
            pois.homes[b].lat,
            residents[h],
            0,
        );
    }
    let mut id = n_homes;
    let mut work =
        |sites: &[Site], flags: &[u8], sub: u8, add: &mut dyn FnMut(u8, u8, f64, f64, f32, u8)| {
            for (s, &f) in sites.iter().zip(flags) {
                add(
                    KIND_WORK,
                    sub,
                    s.lon,
                    s.lat,
                    *assigned.get(&id).unwrap_or(&0) as f32,
                    f,
                );
                id += 1;
            }
        };
    work(&pois.schools, &f_school, SUB_SCHOOL, &mut add);
    work(&pois.colleges, &f_college, SUB_COLLEGE, &mut add);
    work(&pois.offices, &f_office, SUB_OFFICE, &mut add);
    for (s, &f) in pois.markets.iter().zip(&f_market) {
        add(KIND_MARKET, 0, s.lon, s.lat, 0.0, f);
    }
    let bed_each = (beds / pois.hospitals.len() as f64) as f32;
    for (s, &f) in pois.hospitals.iter().zip(&f_hosp) {
        add(KIND_HOSPITAL, 0, s.lon, s.lat, bed_each, f);
    }

    let meta = Meta {
        name: cfg.name.clone(),
        source_url: cfg.geofabrik_url.clone(),
        bbox: cfg.bbox,
        seed,
        target_agents: n,
        agents: n,
        pop_weight: weighter.name().to_string(),
        hospital_beds_per_1000: cfg.hospital_beds_per_1000,
        total_beds: beds,
        counts: Counts {
            homes: n_homes,
            schools: ns as u32,
            colleges: nc as u32,
            offices: no as u32,
            markets: pois.markets.len() as u32,
            hospitals: pois.hospitals.len() as u32,
        },
        synthetic,
    };
    Ok(City {
        locations: loc,
        agents: AgentRows {
            age_group: age,
            home_loc,
            work_loc,
        },
        meta,
    })
}
