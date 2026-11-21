use epidemicsim_core::{sim, Config, SequentialBackend};
use epidemicsim_data::{build_from_pbf, load_world, osm, read_meta, CityConfig};
use std::path::{Path, PathBuf};

const PBF: &str = concat!(env!("CARGO_MANIFEST_DIR"), "/tests/data/tiny.osm.pbf");

fn city(agents: usize) -> CityConfig {
    CityConfig {
        name: "tiny".into(),
        geofabrik_url: "file://tiny".into(),
        bbox: [77.0, 12.0, 77.02, 12.02],
        target_agents: agents,
        hospital_beds_per_1000: 2.0,
        worldpop_tif: None,
        seed: Some(7),
    }
}

fn out(name: &str) -> PathBuf {
    let p = Path::new(env!("CARGO_TARGET_TMPDIR")).join(name);
    let _ = std::fs::remove_dir_all(&p);
    p
}

#[test]
fn extracts_and_clips() {
    let p = osm::extract(Path::new(PBF), &city(1).bbox()).unwrap();
    assert_eq!(p.homes.len(), 12); // outside-bbox building and tagged shop excluded
    assert_eq!(p.homes.iter().map(|h| h.weight).sum::<f64>(), 15.0); // 4-level apartment
    assert_eq!(
        (
            p.offices.len(),
            p.schools.len(),
            p.colleges.len(),
            p.markets.len(),
            p.hospitals.len()
        ),
        (2, 1, 1, 2, 1)
    );
}

#[test]
fn builds_city_deterministically_and_sim_runs() {
    let cfg = city(300);
    let (a, b) = (out("tiny_a"), out("tiny_b"));
    build_from_pbf(&cfg, Path::new(PBF), Path::new("nonexistent"), &a).unwrap();
    build_from_pbf(&cfg, Path::new(PBF), Path::new("nonexistent"), &b).unwrap();
    for f in ["agents.parquet", "locations.parquet", "meta.json"] {
        assert_eq!(
            std::fs::read(a.join(f)).unwrap(),
            std::fs::read(b.join(f)).unwrap(),
            "{f}"
        );
    }
    let m = read_meta(&a).unwrap();
    assert_eq!(m.agents, 300);
    assert_eq!(m.pop_weight, "building_density");
    assert!(m.synthetic.offices > 0 && m.counts.offices >= 2);
    assert!((m.total_beds - 0.6).abs() < 1e-9);

    let sim_cfg = Config {
        days: 10,
        agents: 300,
        ..Default::default()
    };
    let world = load_world(&a, &sim_cfg).unwrap();
    assert_eq!(world.agents.len(), 300);
    assert!(
        world.agents.home_loc.windows(2).all(|w| w[0] <= w[1]),
        "agents sorted by home"
    );
    let total = world.layout.total() as u32;
    assert!(world.agents.work_loc.iter().all(|&l| l < total));
    let o = sim::run_world(&sim_cfg, world, &mut SequentialBackend::default());
    assert_eq!(o.stats.len(), 40);
}

#[test]
fn rejects_oversized_city() {
    assert!(city(1_000_001).validate().is_err());
}
