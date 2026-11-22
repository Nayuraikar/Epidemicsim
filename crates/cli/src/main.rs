use clap::{Args, Parser, Subcommand};
use epidemicsim_core::{sim, Config};
use std::path::PathBuf;

#[derive(Parser)]
#[command(name = "epidemicsim", about = "Epidemic simulation toolkit")]
struct Cli {
    #[command(subcommand)]
    command: Option<Command>,
}

/// Backend selection flags shared by `run`, `compare` and `bench`.
#[derive(Args, Clone)]
struct BackendFlags {
    /// Agent-update variant for ilp/rayon: naive | optimised
    #[arg(long, default_value = "optimised")]
    ilp: String,
    /// Loop schedule for rayon/openmp: static | dynamic | guided
    #[arg(long, default_value = "static")]
    schedule: String,
    /// Chunk size for the schedule (0 = default: static block, dynamic 1024, guided min 64)
    #[arg(long, default_value_t = 0)]
    chunk: usize,
    /// SIMD kernel for the simd backend: scalar | neon | avx2
    #[arg(long, default_value = "scalar")]
    simd: String,
    /// Metal threads per threadgroup: 32 | 64 | 128 | 256
    #[arg(long, default_value_t = 64)]
    metal_tg: usize,
    /// Metal: use private (GPU-only) agent buffers instead of shared
    #[arg(long)]
    metal_private: bool,
    /// CUDA threads per block: 64 | 128 | 256 | 512
    #[arg(long, default_value_t = 256)]
    cuda_block: u32,
    /// CUDA per-location counting: atomic | segmented
    #[arg(long, default_value = "atomic")]
    cuda_count: String,
    /// CUDA update kernel: branch | pred | smem | pairs
    #[arg(long, default_value = "branch")]
    cuda_update: String,
    /// CUDA agent layout: soa | aos
    #[arg(long, default_value = "soa")]
    cuda_layout: String,
    /// CUDA: re-partition agents by health state every N steps (0 = off)
    #[arg(long, default_value_t = 0)]
    cuda_partition: u32,
}

impl BackendFlags {
    fn opts(&self, threads: usize) -> Result<epidemicsim_backends::BackendOpts, String> {
        Ok(epidemicsim_backends::BackendOpts {
            threads,
            ilp: epidemicsim_backends::parse_ilp(&self.ilp)?,
            schedule: self.schedule.parse()?,
            chunk: self.chunk,
            simd: epidemicsim_backends::parse_simd(&self.simd)?,
            metal_tg: self.metal_tg,
            metal_private: self.metal_private,
            cuda_block: self.cuda_block,
            cuda_count: self.cuda_count.clone(),
            cuda_update: self.cuda_update.clone(),
            cuda_layout: self.cuda_layout.clone(),
            cuda_partition: self.cuda_partition,
        })
    }
}

#[derive(Args)]
struct RunArgs {
    #[arg(long, default_value = "sequential")]
    backend: String,
    /// Worker threads (rayon/openmp)
    #[arg(long, default_value_t = 1)]
    threads: usize,
    #[command(flatten)]
    flags: BackendFlags,
    /// TOML config (all fields optional); flags below override it
    #[arg(long)]
    config: Option<PathBuf>,
    /// Load a prebuilt city from <cities-dir>/<city>/ (see `pipeline`); ignores --agents
    #[arg(long)]
    city: Option<String>,
    #[arg(long, default_value = "cities")]
    cities_dir: PathBuf,
    #[arg(long)]
    agents: Option<usize>,
    #[arg(long)]
    days: Option<u32>,
    #[arg(long)]
    seed: Option<u64>,
    /// Output directory for timeseries.parquet and summary.json
    #[arg(long, default_value = "results/run")]
    out: PathBuf,
}

fn run(a: RunArgs) -> Result<(), Box<dyn std::error::Error>> {
    let mut cfg = match &a.config {
        Some(p) => Config::from_file(p)?,
        None => Config::default(),
    };
    cfg.agents = a.agents.unwrap_or(cfg.agents);
    cfg.days = a.days.unwrap_or(cfg.days);
    cfg.seed = a.seed.unwrap_or(cfg.seed);
    let mut backend = epidemicsim_backends::make_with(&a.backend, &a.flags.opts(a.threads)?)?;
    let out = match &a.city {
        Some(name) => {
            let world = epidemicsim_data::load_world(&a.cities_dir.join(name), &cfg)?;
            cfg.agents = world.agents.len();
            sim::run_world(&cfg, world, &mut backend)
        }
        None => sim::run(&cfg, &mut backend),
    };
    std::fs::create_dir_all(&a.out)?;
    epidemicsim_core::output::write_parquet(&a.out.join("timeseries.parquet"), &out.stats)?;
    out.summary.write_json(&a.out.join("summary.json"))?;
    let s = &out.summary;
    println!(
        "agents={} days={} seed={} peak_inf={}@d{} peak_hosp={}@d{} attack={:.1}% deaths={} hash={}",
        s.agents, s.days, s.seed, s.peak_infectious, s.peak_infectious_day,
        s.peak_hospital_load, s.peak_hospital_day, s.attack_rate * 100.0, s.total_deaths, s.hash
    );
    Ok(())
}

#[derive(Args)]
struct PipelineArgs {
    /// City names (cities/<name>.toml)
    #[arg(required = true)]
    cities: Vec<String>,
    #[arg(long, default_value = "cities")]
    cities_dir: PathBuf,
    #[arg(long, default_value = "data_cache")]
    cache_dir: PathBuf,
    /// Use this .pbf instead of downloading (single city)
    #[arg(long)]
    pbf: Option<PathBuf>,
    /// Override target agents from the TOML (max 1M)
    #[arg(long)]
    agents: Option<usize>,
}

fn pipeline(a: PipelineArgs) -> Result<(), Box<dyn std::error::Error>> {
    let opts = epidemicsim_data::PipelineOpts {
        cities_dir: a.cities_dir,
        cache_dir: a.cache_dir,
        pbf: a.pbf,
        agents: a.agents,
    };
    for c in &a.cities {
        let dir = epidemicsim_data::build_city(c, &opts)?;
        let m = epidemicsim_data::read_meta(&dir)?;
        println!(
            "{c}: agents={} homes={} work={} markets={} hospitals={} weight={} -> {}",
            m.agents,
            m.counts.homes,
            m.counts.schools + m.counts.colleges + m.counts.offices,
            m.counts.markets,
            m.counts.hospitals,
            m.pop_weight,
            dir.display()
        );
    }
    Ok(())
}

#[derive(Args)]
struct CompareArgs {
    /// MPI distribution of (scenario x run) tasks: auto | off | static | dynamic.
    /// auto = static when started by mpirun, otherwise single process.
    #[arg(long, default_value = "auto")]
    mpi: String,
    /// Write the MPI timing report (JSON) here
    #[arg(long)]
    mpi_timing: Option<PathBuf>,
    /// City folder under --cities-dir; omit for a synthetic city
    #[arg(long)]
    city: Option<String>,
    #[arg(long, default_value = "cities")]
    cities_dir: PathBuf,
    /// "all" or a comma-separated list of scenario names
    #[arg(long, default_value = "all")]
    scenarios: String,
    #[arg(long, default_value = "scenarios")]
    scenarios_dir: PathBuf,
    #[arg(long, default_value_t = 50)]
    runs: usize,
    /// TOML config (all fields optional); flags below override it
    #[arg(long)]
    config: Option<PathBuf>,
    /// Synthetic city size (ignored with --city)
    #[arg(long)]
    agents: Option<usize>,
    #[arg(long)]
    days: Option<u32>,
    /// Base seed; run k uses seed + k
    #[arg(long)]
    seed: Option<u64>,
    /// Hospital beds per 1000 agents for synthetic cities (cities use their meta.json)
    #[arg(long, default_value_t = 2.0)]
    beds_per_1000: f64,
    #[arg(long, default_value = "sequential")]
    backend: String,
    /// Worker threads (rayon/openmp)
    #[arg(long, default_value_t = 1)]
    threads: usize,
    #[command(flatten)]
    flags: BackendFlags,
    #[arg(long, default_value = "results")]
    out: PathBuf,
}

fn compare(a: CompareArgs) -> Result<(), Box<dyn std::error::Error>> {
    use epidemicsim_scenarios::{compare::CompareOpts, EnsembleOpts, WorldSource};
    let mut cfg = match &a.config {
        Some(p) => Config::from_file(p)?,
        None => Config::default(),
    };
    cfg.agents = a.agents.unwrap_or(cfg.agents);
    cfg.days = a.days.unwrap_or(cfg.days);
    cfg.seed = a.seed.unwrap_or(cfg.seed);
    let bopts = a.flags.opts(a.threads)?;
    epidemicsim_backends::make_with(&a.backend, &bopts)?;

    let all = epidemicsim_scenarios::load_all(&a.scenarios_dir)?;
    let chosen = if a.scenarios == "all" {
        all
    } else {
        let mut v = Vec::new();
        for name in a.scenarios.split(',').map(str::trim) {
            let s = all
                .iter()
                .find(|s| s.name == name)
                .ok_or_else(|| format!("no scenario '{name}' in {}", a.scenarios_dir.display()))?;
            v.push(s.clone());
        }
        v
    };

    let (src, label, beds) = match &a.city {
        Some(name) => {
            let dir = a.cities_dir.join(name);
            let world = epidemicsim_data::load_world(&dir, &cfg)?;
            let beds = epidemicsim_data::read_meta(&dir)?.total_beds;
            let frac = cfg.disease.initial_infected_frac;
            (
                WorldSource::City {
                    world: Box::new(world),
                    infected_frac: frac,
                },
                name.clone(),
                beds,
            )
        }
        None => (
            WorldSource::Synth(Box::new(cfg.clone())),
            "synthetic".to_string(),
            a.beds_per_1000 * cfg.agents as f64 / 1000.0,
        ),
    };
    let opts = CompareOpts {
        city: label.clone(),
        backend: a.backend.clone(),
        ensemble: EnsembleOpts {
            runs: a.runs,
            days: cfg.days,
            base_seed: cfg.seed,
            hospital_capacity: beds.ceil().max(1.0) as u32,
        },
    };
    let backend = a.backend.clone();
    #[cfg(feature = "mpi")]
    {
        let launched = epidemicsim_mpi::launched_by_mpirun();
        let mode = match a.mpi.as_str() {
            "auto" => launched.then_some(epidemicsim_mpi::Mode::Static),
            "off" => None,
            m => Some(m.parse::<epidemicsim_mpi::Mode>()?),
        };
        if launched && mode.is_none() {
            return Err("started by mpirun but --mpi off: every rank would repeat the work".into());
        }
        if let Some(mode) = mode {
            let rep = epidemicsim_mpi::compare_mpi(
                &src,
                &chosen,
                &opts,
                &a.out,
                move || epidemicsim_backends::make_with(&backend, &bopts).expect("validated above"),
                mode,
            )?;
            if rep.per_rank[0].rank == 0 {
                if let Some(p) = &a.mpi_timing {
                    epidemicsim_mpi::scaling::write(p, &rep)?;
                }
                println!(
                    "wrote {}/{label}/comparison.json ({} scenarios x {} runs, {} ranks, {:?})",
                    a.out.display(),
                    chosen.len(),
                    a.runs,
                    rep.ranks,
                    mode
                );
            }
            return Ok(());
        }
    }
    #[cfg(not(feature = "mpi"))]
    if a.mpi != "auto" && a.mpi != "off" {
        return Err("built without the `mpi` feature".into());
    }
    epidemicsim_scenarios::compare(&src, &chosen, &opts, &a.out, move || {
        epidemicsim_backends::make_with(&backend, &bopts).expect("validated above")
    })?;
    println!(
        "wrote {}/{label}/comparison.json ({} scenarios x {} runs)",
        a.out.display(),
        chosen.len(),
        a.runs
    );
    Ok(())
}

#[derive(Args)]
struct BenchArgs {
    #[arg(long, default_value = "sequential")]
    backend: String,
    #[command(flatten)]
    flags: BackendFlags,
    /// Comma-separated agent counts
    #[arg(long, value_delimiter = ',', default_values_t = [10_000usize, 100_000, 1_000_000])]
    agents: Vec<usize>,
    /// Also benchmark 10M agents (needs several GB of RAM)
    #[arg(long)]
    include_10m: bool,
    /// Comma-separated thread counts; the first is the speedup reference
    #[arg(long, value_delimiter = ',', default_values_t = [1usize])]
    threads: Vec<usize>,
    /// Simulated days per timed run (4 steps/day)
    #[arg(long, default_value_t = 20)]
    days: u32,
    #[arg(long, default_value_t = 1)]
    warmup: usize,
    #[arg(long, default_value_t = 5)]
    reps: usize,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value = "results/bench")]
    out: PathBuf,
    /// scaling (default) | race | imbalance | simd-align | metal | mpi | cuda | cuda-variants | warp
    /// (the last three need `--features cuda`)
    #[arg(long, default_value = "scaling")]
    suite: String,
    /// City for --suite imbalance (loaded from --cities-dir)
    #[arg(long, default_value = "mumbai")]
    city: String,
    #[arg(long, default_value = "cities")]
    cities_dir: PathBuf,
    /// Output directory for the GPU suites (cuda, cuda-variants, warp)
    #[arg(long, default_value = "results/gpu")]
    gpu_out: PathBuf,
    /// Output directory for race / imbalance results
    #[arg(long, default_value = "results/tlp")]
    tlp_out: PathBuf,
}

fn bench_suite(a: &BenchArgs) -> Result<bool, Box<dyn std::error::Error>> {
    use epidemicsim_bench::tlp;
    match a.suite.as_str() {
        "scaling" => Ok(false),
        "race" => {
            let o = tlp::RaceOpts {
                threads: *a.threads.last().unwrap_or(&8),
                ..Default::default()
            };
            let r = tlp::race(&o)?;
            let path = a.tlp_out.join("race_demo.json");
            tlp::write_json(&path, &r)?;
            println!("wrote {}", path.display());
            Ok(true)
        }
        "imbalance" => {
            let cfg = Config::default();
            let world = epidemicsim_data::load_world(&a.cities_dir.join(&a.city), &cfg)?;
            let o = tlp::ImbalanceOpts {
                city: a.city.clone(),
                threads: if a.threads == [1] {
                    vec![1, 2, 4, 8]
                } else {
                    a.threads.clone()
                },
                days: a.days,
                reps: a.reps.min(3),
                chunk: a.flags.chunk,
            };
            let r = tlp::imbalance(&world, &o)?;
            let path = a.tlp_out.join(format!("imbalance_{}.json", a.city));
            tlp::write_json(&path, &r)?;
            println!("wrote {}", path.display());
            Ok(true)
        }
        #[cfg(feature = "mpi")]
        "mpi" => {
            let procs = if a.threads == [1] {
                vec![1, 2, 4, 8]
            } else {
                a.threads.clone()
            };
            let (agents, runs) = (50_000, 6);
            let o = epidemicsim_mpi::scaling::Opts {
                exe: std::env::current_exe()?,
                procs,
                reps: a.reps.max(1),
                agents,
                days: 90,
                runs,
                scenarios_dir: "scenarios".into(),
                backend: "sequential".into(),
                work_dir: std::env::temp_dir().join("epidemicsim_mpi_scaling"),
            };
            let n_scen = epidemicsim_scenarios::load_all(std::path::Path::new("scenarios"))?.len();
            let machine = epidemicsim_bench::machine_info();
            let sc = epidemicsim_mpi::scaling::run(&o, &machine.cpu, n_scen)?;
            epidemicsim_mpi::scaling::write(std::path::Path::new("results/mpi/scaling.json"), &sc)?;
            // Same JSON layout as the other backends: `threads` = MPI processes.
            for (mode, name) in [("static", "mpi"), ("dynamic", "mpi_dynamic")] {
                let steps = o.days * 4;
                let points = sc
                    .points
                    .iter()
                    .filter(|p| p.mode == mode)
                    .map(|p| {
                        let work = (sc.tasks * agents) as f64 * f64::from(steps);
                        epidemicsim_bench::Point {
                            agents,
                            threads: p.procs,
                            steps,
                            wall_s: epidemicsim_bench::Timing {
                                median: p.wall_s.median,
                                min: p.wall_s.min,
                                max: p.wall_s.max,
                            },
                            agents_per_sec: work / p.wall_s.median,
                            contacts_per_sec: work / p.wall_s.median,
                            speedup: p.speedup,
                            efficiency: p.efficiency,
                        }
                    })
                    .collect::<Vec<_>>();
                let pairs: Vec<(f64, f64)> = points
                    .iter()
                    .map(|p| (p.threads as f64, p.speedup))
                    .collect();
                let amdahl = epidemicsim_bench::amdahl_fit(&pairs)
                    .map(|s| {
                        vec![epidemicsim_bench::Amdahl {
                            agents,
                            serial_fraction: s,
                            max_speedup: if s > 0.0 { 1.0 / s } else { f64::INFINITY },
                        }]
                    })
                    .unwrap_or_default();
                let rep = epidemicsim_bench::Report {
                    backend: name.into(),
                    machine: machine.clone(),
                    days: o.days,
                    warmup: 1,
                    reps: o.reps,
                    seed: 42,
                    points,
                    amdahl,
                };
                println!(
                    "wrote {}",
                    epidemicsim_bench::write_report(&a.out, &rep)?.display()
                );
            }
            println!("wrote results/mpi/scaling.json");
            Ok(true)
        }
        #[cfg(target_os = "macos")]
        "metal" => {
            let agents = if a.agents == [10_000, 100_000, 1_000_000] {
                vec![100_000, 1_000_000]
            } else {
                a.agents.clone()
            };
            let b = epidemicsim_backends::mtl::measure::breakdown(
                &agents,
                &[32, 64, 128, 256],
                a.days,
                a.warmup,
                a.reps.max(1),
                a.seed,
            )?;
            let path = std::path::Path::new("results/gpu/metal_breakdown.json");
            b.write(path)?;
            println!("wrote {}", path.display());
            Ok(true)
        }
        #[cfg(not(feature = "mpi"))]
        "mpi" => Err("built without the `mpi` feature".into()),
        #[cfg(feature = "cuda")]
        "cuda" | "cuda-variants" | "warp" => {
            use epidemicsim_backends::cuda::measure;
            let big = a.agents == [10_000, 100_000, 1_000_000];
            let reps = a.reps.max(1);
            let (name, b) = match a.suite.as_str() {
                "cuda" => {
                    let agents = if big {
                        vec![100_000, 1_000_000]
                    } else {
                        a.agents.clone()
                    };
                    (
                        "cuda_breakdown",
                        measure::breakdown(&agents, a.days, a.warmup, reps, a.seed)?,
                    )
                }
                "cuda-variants" => {
                    let agents = if big {
                        vec![1_000_000]
                    } else {
                        a.agents.clone()
                    };
                    let blk = a.flags.cuda_block;
                    (
                        "cuda_variants",
                        measure::variants(&agents, blk, a.days, a.warmup, reps, a.seed)?,
                    )
                }
                _ => {
                    let n = if big {
                        1_000_000
                    } else {
                        *a.agents.last().unwrap_or(&1_000_000)
                    };
                    let prev = [0.0005, 0.005, 0.02, 0.1, 0.4];
                    let blk = a.flags.cuda_block;
                    (
                        "cuda_warp",
                        measure::warp(n, &prev, blk, a.days, a.warmup, reps, a.seed)?,
                    )
                }
            };
            let path = a.gpu_out.join(format!("{name}.json"));
            b.write(&path)?;
            println!("wrote {}", path.display());
            Ok(true)
        }
        #[cfg(not(feature = "cuda"))]
        "cuda" | "cuda-variants" | "warp" => {
            Err("this suite needs a build with `--features cuda` and an NVIDIA GPU".into())
        }
        "simd-align" => {
            let path = epidemicsim_bench::tlp::align_bench(&a.out, a.reps.max(3))?;
            println!("wrote {}", path.display());
            Ok(true)
        }
        s => Err(format!("unknown --suite '{s}'").into()),
    }
}

fn bench(a: BenchArgs) -> Result<(), Box<dyn std::error::Error>> {
    if bench_suite(&a)? {
        return Ok(());
    }
    let bopts = a.flags.opts(1)?;
    epidemicsim_backends::make_with(&a.backend, &bopts)?;
    let mut agents = a.agents.clone();
    if a.include_10m && !agents.contains(&10_000_000) {
        agents.push(10_000_000);
    }
    let opts = epidemicsim_bench::BenchOpts {
        agents,
        threads: a.threads.clone(),
        days: a.days,
        warmup: a.warmup,
        reps: a.reps.max(1),
        seed: a.seed,
    };
    let backend = a.backend.clone();
    let b2 = bopts.clone();
    let factory = move |t: usize| {
        let mut o = b2.clone();
        o.threads = t;
        epidemicsim_backends::make_with(&backend, &o)
    };
    // Variant label (e.g. rayon_dynamic_c1024) keeps reports of different variants apart.
    let label = epidemicsim_backends::label(&a.backend, &bopts);
    let report = epidemicsim_bench::run(&label, &factory, &opts)?;
    let path = epidemicsim_bench::write_report(&a.out, &report)?;
    println!("wrote {}", path.display());
    Ok(())
}

#[derive(Args)]
struct ServeArgs {
    #[arg(long, default_value = "127.0.0.1:8080")]
    addr: String,
    #[arg(long, default_value = "cities")]
    cities_dir: PathBuf,
    #[arg(long, default_value = "scenarios")]
    scenarios_dir: PathBuf,
    #[arg(long, default_value = "results")]
    results_dir: PathBuf,
    /// Built web app to serve (npm run build in web/)
    #[arg(long, default_value = "web/dist")]
    web_dir: PathBuf,
}

#[derive(Args)]
struct BakeArgs {
    #[arg(long)]
    city: String,
    /// "all" or a comma-separated list of scenario names
    #[arg(long, default_value = "all")]
    scenarios: String,
    #[arg(long, default_value = "rayon")]
    backend: String,
    /// Worker threads (rayon/openmp); 0 = all cores
    #[arg(long, default_value_t = 0)]
    threads: usize,
    /// Agents to simulate (0 = the full city)
    #[arg(long, default_value_t = 0)]
    agents: usize,
    #[arg(long, default_value_t = 120)]
    days: u32,
    #[arg(long, default_value_t = 42)]
    seed: u64,
    #[arg(long, default_value = "cities")]
    cities_dir: PathBuf,
    #[arg(long, default_value = "scenarios")]
    scenarios_dir: PathBuf,
    #[arg(long, default_value = "results")]
    out: PathBuf,
}

fn bake(a: BakeArgs) -> Result<(), Box<dyn std::error::Error>> {
    let threads = if a.threads == 0 {
        std::thread::available_parallelism().map_or(1, |n| n.get())
    } else {
        a.threads
    };
    let dirs = epidemicsim_server::Dirs {
        cities: a.cities_dir,
        scenarios: a.scenarios_dir,
        results: a.out,
        web: PathBuf::new(),
    };
    epidemicsim_server::bake(
        &dirs,
        &epidemicsim_server::BakeOpts {
            city: a.city,
            scenarios: (a.scenarios != "all").then(|| {
                a.scenarios
                    .split(',')
                    .map(|s| s.trim().to_string())
                    .collect()
            }),
            backend: a.backend,
            threads,
            agents: a.agents,
            days: a.days,
            seed: a.seed,
        },
    )
}

fn serve(a: ServeArgs) -> Result<(), Box<dyn std::error::Error>> {
    epidemicsim_server::serve(
        epidemicsim_server::Dirs {
            cities: a.cities_dir,
            scenarios: a.scenarios_dir,
            results: a.results_dir,
            web: a.web_dir,
        },
        &a.addr,
    )
}

#[derive(Subcommand)]
enum Command {
    /// Run a synthetic-city simulation
    Run(RunArgs),
    /// Run scenarios as Monte Carlo ensembles and compare them
    Compare(CompareArgs),
    /// Benchmark a backend over several agent counts
    Bench(BenchArgs),
    /// Build city folders from OSM data
    Pipeline(PipelineArgs),
    /// Start the studio server (REST + WebSocket + static web/dist)
    Serve(ServeArgs),
    /// Pre-bake replay frames under results/<city>/<scenario>/frames
    Bake(BakeArgs),
    /// Run everything (stub)
    All,
}

fn main() {
    let cli = Cli::parse();
    if let Some(Command::Run(a)) = cli.command {
        if let Err(e) = run(a) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(Command::Pipeline(a)) = cli.command {
        if let Err(e) = pipeline(a) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(Command::Compare(a)) = cli.command {
        if let Err(e) = compare(a) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(Command::Serve(a)) = cli.command {
        if let Err(e) = serve(a) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(Command::Bake(a)) = cli.command {
        if let Err(e) = bake(a) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    if let Some(Command::Bench(a)) = cli.command {
        if let Err(e) = bench(a) {
            eprintln!("error: {e}");
            std::process::exit(1);
        }
        return;
    }
    println!("epidemicsim");
    if let Some(cmd) = cli.command {
        let name = match cmd {
            Command::Run(_) => "run",
            Command::Compare(_) => "compare",
            Command::Bench(_) => "bench",
            Command::Pipeline(_) => "pipeline",
            Command::Serve(_) | Command::Bake(_) => unreachable!(),
            Command::All => "all",
        };
        println!("{name}: not implemented yet");
    }
}
