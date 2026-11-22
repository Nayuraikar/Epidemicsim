//! HTTP + WebSocket server for the studio, and pre-baked frame replay.
pub mod sim;

use axum::extract::ws::{Message, WebSocket, WebSocketUpgrade};
use axum::extract::{Path as UrlPath, State};
use axum::http::{header, StatusCode};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use axum::{Json, Router};
use epidemicsim_core::World;
use epidemicsim_scenarios::Scenario;
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};
use sim::{CityCtx, Grid, Sim, SimSpec};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::mpsc as std_mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tower_http::cors::CorsLayer;
use tower_http::services::{ServeDir, ServeFile};

/// Live runs are capped so the UI stays responsive (frames are produced and rendered per step).
pub const MAX_LIVE_AGENTS: usize = 250_000;
pub const DEFAULT_LIVE_AGENTS: usize = 100_000;
pub const MAX_DAYS: u32 = 365;

pub struct Dirs {
    pub cities: PathBuf,
    pub scenarios: PathBuf,
    pub results: PathBuf,
    pub web: PathBuf,
}

pub struct App {
    pub dirs: Dirs,
    ctxs: Mutex<HashMap<String, Arc<CityCtx>>>,
}

type Shared = Arc<App>;
type ApiErr = (StatusCode, String);

fn bad(m: impl ToString) -> ApiErr {
    (StatusCode::BAD_REQUEST, m.to_string())
}
fn missing(m: impl ToString) -> ApiErr {
    (StatusCode::NOT_FOUND, m.to_string())
}

/// Names used in file paths: letters, digits, `_`, `-` only.
fn safe(name: &str) -> Result<&str, ApiErr> {
    if !name.is_empty()
        && name
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
    {
        Ok(name)
    } else {
        Err(bad("invalid name"))
    }
}

impl App {
    pub fn new(dirs: Dirs) -> Self {
        Self {
            dirs,
            ctxs: Mutex::new(HashMap::new()),
        }
    }

    /// Load (once) meta, locations and the spatial grid of a built city.
    pub fn city(&self, name: &str) -> Result<Arc<CityCtx>, String> {
        if let Some(c) = self.ctxs.lock().unwrap().get(name) {
            return Ok(c.clone());
        }
        let dir = self.dirs.cities.join(name);
        let meta =
            epidemicsim_data::read_meta(&dir).map_err(|_| format!("city '{name}' is not built"))?;
        let locs = epidemicsim_data::io::read_locations(&dir).map_err(|e| e.to_string())?;
        let grid = Arc::new(Grid::new(meta.bbox, &locs));
        let ctx = Arc::new(CityCtx {
            meta,
            grid,
            locs,
            worlds: Mutex::new(HashMap::new()),
        });
        self.ctxs
            .lock()
            .unwrap()
            .insert(name.to_string(), ctx.clone());
        Ok(ctx)
    }

    /// World with about `agents` agents (1 = full city), cached per subsample factor.
    pub fn world(
        &self,
        name: &str,
        ctx: &CityCtx,
        agents: usize,
    ) -> Result<(Arc<World>, u32), String> {
        let full = ctx.meta.agents.max(1);
        let k = full.div_ceil(agents.max(1)).max(1) as u32;
        if let Some(w) = ctx.worlds.lock().unwrap().get(&k) {
            return Ok((w.clone(), k));
        }
        let cfg = epidemicsim_core::Config::default();
        let full_world = epidemicsim_data::load_world(&self.dirs.cities.join(name), &cfg)
            .map_err(|e| e.to_string())?;
        let w = Arc::new(epidemicsim_data::subsample_world(&full_world, k));
        ctx.worlds.lock().unwrap().insert(k, w.clone());
        Ok((w, k))
    }
}

fn read_json(path: &Path) -> Result<String, ApiErr> {
    std::fs::read_to_string(path).map_err(|_| missing(format!("not found: {}", path.display())))
}

fn json_body(s: String) -> Response {
    ([(header::CONTENT_TYPE, "application/json")], s).into_response()
}

// ---------- REST ----------

async fn list_cities(State(app): State<Shared>) -> Result<Json<Value>, ApiErr> {
    let mut out = Vec::new();
    let rd = std::fs::read_dir(&app.dirs.cities).map_err(|e| missing(e.to_string()))?;
    let mut names: Vec<String> = rd
        .filter_map(|e| e.ok())
        .filter_map(|e| {
            e.path()
                .file_stem()
                .map(|s| (e.path(), s.to_string_lossy().to_string()))
        })
        .filter(|(p, _)| p.extension().is_some_and(|x| x == "toml"))
        .map(|(_, n)| n)
        .collect();
    names.sort();
    for name in names {
        let dir = app.dirs.cities.join(&name);
        let res = app.dirs.results.join(&name);
        let (built, bbox, agents, beds) = match epidemicsim_data::read_meta(&dir) {
            Ok(m) => (true, Some(m.bbox), Some(m.agents), Some(m.total_beds)),
            Err(_) => {
                let cfg = epidemicsim_data::CityConfig::from_file(
                    &app.dirs.cities.join(format!("{name}.toml")),
                )
                .ok();
                (false, cfg.map(|c| c.bbox), None, None)
            }
        };
        let centroid = bbox.map(|b| [(b[0] + b[2]) / 2.0, (b[1] + b[3]) / 2.0]);
        let frames: Vec<String> = std::fs::read_dir(&res)
            .map(|rd| {
                rd.filter_map(|e| e.ok())
                    .filter(|e| e.path().join("frames/frames.json").exists())
                    .map(|e| e.file_name().to_string_lossy().to_string())
                    .collect()
            })
            .unwrap_or_default();
        out.push(json!({
            "name": name, "built": built, "agents": agents, "total_beds": beds,
            "bbox": bbox, "centroid": centroid,
            "has_comparison": res.join("comparison.json").exists(),
            "baked_scenarios": frames,
        }));
    }
    Ok(Json(
        json!({ "cities": out, "max_live_agents": MAX_LIVE_AGENTS, "default_live_agents": DEFAULT_LIVE_AGENTS,
        "backends": epidemicsim_backends::NAMES }),
    ))
}

async fn list_scenarios(State(app): State<Shared>) -> Result<Json<Value>, ApiErr> {
    let all =
        epidemicsim_scenarios::load_all(&app.dirs.scenarios).map_err(|e| missing(e.to_string()))?;
    Ok(Json(json!({ "scenarios": all })))
}

async fn comparison(
    State(app): State<Shared>,
    UrlPath(city): UrlPath<String>,
) -> Result<Response, ApiErr> {
    Ok(json_body(read_json(
        &app.dirs.results.join(safe(&city)?).join("comparison.json"),
    )?))
}

async fn summary(
    State(app): State<Shared>,
    UrlPath((city, sc)): UrlPath<(String, String)>,
) -> Result<Response, ApiErr> {
    let p = app
        .dirs
        .results
        .join(safe(&city)?)
        .join(safe(&sc)?)
        .join("summary.json");
    Ok(json_body(read_json(&p)?))
}

async fn bands(
    State(app): State<Shared>,
    UrlPath((city, sc)): UrlPath<(String, String)>,
) -> Result<Json<Value>, ApiErr> {
    let p = app
        .dirs
        .results
        .join(safe(&city)?)
        .join(safe(&sc)?)
        .join("bands.parquet");
    if !p.exists() {
        return Err(missing("no bands: run `epidemicsim compare` for this city"));
    }
    let cols =
        epidemicsim_data::io::read_numeric_columns(&p).map_err(|e| missing(e.to_string()))?;
    let mut series: serde_json::Map<String, Value> = serde_json::Map::new();
    let mut days = Value::Null;
    for (name, v) in cols {
        if name == "day" {
            days = json!(v);
            continue;
        }
        if let Some((s, q)) = name.rsplit_once('_') {
            if matches!(q, "mean" | "p05" | "p95") {
                let e = series.entry(s.to_string()).or_insert_with(|| json!({}));
                e[q] = json!(v
                    .iter()
                    .map(|x| (x * 1000.0).round() / 1000.0)
                    .collect::<Vec<_>>());
            }
        }
    }
    Ok(Json(
        json!({ "steps_per_day": 4, "day": days, "series": series }),
    ))
}

async fn frames(
    State(app): State<Shared>,
    UrlPath((city, sc)): UrlPath<(String, String)>,
) -> Result<Response, ApiErr> {
    let d = app
        .dirs
        .results
        .join(safe(&city)?)
        .join(safe(&sc)?)
        .join("frames");
    if !d.join("frames.json").exists() {
        return Err(missing(format!(
            "no baked frames for {city}/{sc}: run `epidemicsim bake --city {city}`"
        )));
    }
    let (meta, fr) = (
        read_json(&d.join("meta.json"))?,
        read_json(&d.join("frames.json"))?,
    );
    Ok(json_body(format!("{{\"meta\":{meta},\"frames\":{fr}}}")))
}

// ---------- WebSocket live runs ----------

#[derive(Deserialize)]
struct StartMsg {
    city: String,
    scenario: Scenario,
    #[serde(default)]
    backend: Option<String>,
    #[serde(default)]
    agents: Option<usize>,
    #[serde(default)]
    days: Option<u32>,
    #[serde(default)]
    seed: Option<u64>,
    #[serde(default)]
    chains: bool,
    #[serde(default)]
    speed: Option<f64>,
}

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum ClientMsg {
    Start(Box<StartMsg>),
    Pause,
    Resume,
    Stop,
    Speed {
        steps_per_sec: f64,
    },
    Inspect {
        agent: Option<u32>,
        cell: Option<u32>,
    },
}

enum Cmd {
    Pause,
    Resume,
    Stop,
    Speed(f64),
    Inspect(Option<u32>, Option<u32>),
}

#[derive(Serialize)]
#[serde(tag = "type", rename_all = "lowercase")]
enum Out<'a> {
    Status { message: &'a str },
    Ready(Ready<'a>),
    Frame(&'a sim::Frame),
    Agent { agent: Value },
    Done { stopped: bool },
    Error { message: &'a str },
}

#[derive(Serialize)]
struct Ready<'a> {
    grid: &'a Grid,
    agents: usize,
    city_agents: usize,
    subsample: u32,
    capacity: u32,
    days: u32,
    backend: &'a str,
    scenario: &'a str,
    chains: bool,
}

fn txt(o: &Out) -> String {
    serde_json::to_string(o).unwrap_or_default()
}

fn build_sim(app: &App, m: &StartMsg) -> Result<(Sim, Arc<CityCtx>), String> {
    let ctx = app.city(safe(&m.city).map_err(|e| e.1)?)?;
    let want = m
        .agents
        .unwrap_or(DEFAULT_LIVE_AGENTS)
        .clamp(5_000, MAX_LIVE_AGENTS);
    let (world, k) = app.world(&m.city, &ctx, want)?;
    let spec = SimSpec {
        days: m.days.unwrap_or(120).clamp(1, MAX_DAYS),
        seed: m.seed.unwrap_or(42),
        backend: m.backend.clone().unwrap_or_else(|| "rayon".into()),
        threads: std::thread::available_parallelism().map_or(1, |n| n.get()),
        scenario: m.scenario.clone(),
        chains: m.chains,
    };
    m.scenario.validate()?;
    Ok((Sim::new(ctx.clone(), world, k, &spec)?, ctx))
}

/// Runs on its own thread: owns the simulation, honours pause/stop/speed, answers inspections.
fn run_live(app: Shared, m: StartMsg, cmds: std_mpsc::Receiver<Cmd>, out: mpsc::Sender<String>) {
    let send = |o: Out| out.blocking_send(txt(&o)).is_ok();
    send(Out::Status {
        message: "loading city",
    });
    let (mut sim, ctx) = match build_sim(&app, &m) {
        Ok(s) => s,
        Err(e) => {
            send(Out::Error { message: &e });
            return;
        }
    };
    let days = sim.total_steps / epidemicsim_core::world::STEPS_PER_DAY;
    if !send(Out::Ready(Ready {
        grid: &ctx.grid,
        agents: sim.agents(),
        city_agents: ctx.meta.agents,
        subsample: sim.subsample,
        capacity: sim.capacity,
        days,
        backend: sim.backend_name,
        scenario: &m.scenario.name,
        chains: m.chains,
    })) {
        return;
    }
    let (mut paused, mut speed) = (false, m.speed.unwrap_or(20.0).clamp(0.5, 1000.0));
    let mut next = Instant::now();
    loop {
        let cmd = if paused {
            match cmds.recv_timeout(Duration::from_millis(100)) {
                Ok(c) => Some(c),
                Err(std_mpsc::RecvTimeoutError::Timeout) => continue,
                Err(_) => return,
            }
        } else {
            match cmds.try_recv() {
                Ok(c) => Some(c),
                Err(std_mpsc::TryRecvError::Empty) => None,
                Err(_) => return,
            }
        };
        if let Some(c) = cmd {
            match c {
                Cmd::Pause => paused = true,
                Cmd::Resume => {
                    paused = false;
                    next = Instant::now();
                }
                Cmd::Stop => {
                    send(Out::Done { stopped: true });
                    return;
                }
                Cmd::Speed(s) => speed = s.clamp(0.5, 1000.0),
                Cmd::Inspect(a, c) => {
                    let ok = match sim.inspect(a, c) {
                        Ok(agent) => send(Out::Agent { agent }),
                        Err(e) => send(Out::Error { message: &e }),
                    };
                    if !ok {
                        return;
                    }
                }
            }
            continue;
        }
        if sim.done() {
            send(Out::Done { stopped: false });
            return;
        }
        let f = sim.step();
        if !send(Out::Frame(&f)) {
            return;
        }
        next += Duration::from_secs_f64(1.0 / speed);
        let now = Instant::now();
        if next > now {
            std::thread::sleep(next - now);
        } else {
            next = now;
        }
    }
}

async fn ws_live(ws: WebSocketUpgrade, State(app): State<Shared>) -> Response {
    ws.on_upgrade(move |s| ws_session(s, app))
}

async fn ws_session(mut socket: WebSocket, app: Shared) {
    let mut ctl: Option<std_mpsc::Sender<Cmd>> = None;
    let mut rx: Option<mpsc::Receiver<String>> = None;
    loop {
        let from_run = async {
            match rx.as_mut() {
                Some(r) => r.recv().await,
                None => std::future::pending().await,
            }
        };
        tokio::select! {
            msg = socket.recv() => {
                let Some(Ok(msg)) = msg else { break };
                let Message::Text(t) = msg else { if matches!(msg, Message::Close(_)) { break } else { continue } };
                let reply = |m: &str| txt(&Out::Error { message: m });
                match serde_json::from_str::<ClientMsg>(&t) {
                    Err(e) => { let _ = socket.send(Message::text(reply(&e.to_string()))).await; }
                    Ok(ClientMsg::Start(m)) => {
                        if let Some(c) = ctl.take() { let _ = c.send(Cmd::Stop); }
                        let (ctx_tx, ctx_rx) = std_mpsc::channel();
                        let (out_tx, out_rx) = mpsc::channel(64);
                        let a = app.clone();
                        std::thread::spawn(move || run_live(a, *m, ctx_rx, out_tx));
                        ctl = Some(ctx_tx);
                        rx = Some(out_rx);
                    }
                    Ok(other) => {
                        let cmd = match other {
                            ClientMsg::Pause => Cmd::Pause,
                            ClientMsg::Resume => Cmd::Resume,
                            ClientMsg::Stop => Cmd::Stop,
                            ClientMsg::Speed { steps_per_sec } => Cmd::Speed(steps_per_sec),
                            ClientMsg::Inspect { agent, cell } => Cmd::Inspect(agent, cell),
                            ClientMsg::Start(_) => unreachable!(),
                        };
                        match &ctl {
                            Some(c) => { let _ = c.send(cmd); }
                            None => { let _ = socket.send(Message::text(reply("no run in progress"))).await; }
                        }
                    }
                }
            }
            out = from_run => match out {
                Some(s) => { if socket.send(Message::text(s)).await.is_err() { break } }
                None => { rx = None; ctl = None; }
            }
        }
    }
    if let Some(c) = ctl {
        let _ = c.send(Cmd::Stop);
    }
}

pub fn router(app: Shared) -> Router {
    let index = app.dirs.web.join("index.html");
    let spa = ServeDir::new(&app.dirs.web).not_found_service(ServeFile::new(index));
    Router::new()
        .route("/api/cities", get(list_cities))
        .route("/api/scenarios", get(list_scenarios))
        .route("/api/cities/{city}/comparison", get(comparison))
        .route("/api/cities/{city}/scenarios/{sc}/summary", get(summary))
        .route("/api/cities/{city}/scenarios/{sc}/bands", get(bands))
        .route("/api/cities/{city}/scenarios/{sc}/frames", get(frames))
        .route("/ws/live", get(ws_live))
        .fallback_service(spa)
        .layer(CorsLayer::permissive())
        .with_state(app)
}

pub fn serve(dirs: Dirs, addr: &str) -> Result<(), Box<dyn std::error::Error>> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        let listener = tokio::net::TcpListener::bind(addr).await?;
        eprintln!("epidemicsim studio on http://{}", listener.local_addr()?);
        axum::serve(listener, router(Arc::new(App::new(dirs)))).await?;
        Ok(())
    })
}

// ---------- bake ----------

pub struct BakeOpts {
    pub city: String,
    /// `None` = all scenarios in the scenarios dir.
    pub scenarios: Option<Vec<String>>,
    pub backend: String,
    pub threads: usize,
    /// 0 = full city.
    pub agents: usize,
    pub days: u32,
    pub seed: u64,
}

/// Run each scenario once and store frames under `results/<city>/<scenario>/frames/`.
pub fn bake(dirs: &Dirs, o: &BakeOpts) -> Result<(), Box<dyn std::error::Error>> {
    let app = App::new(Dirs {
        cities: dirs.cities.clone(),
        scenarios: dirs.scenarios.clone(),
        results: dirs.results.clone(),
        web: dirs.web.clone(),
    });
    let ctx = app.city(&o.city)?;
    let want = if o.agents == 0 {
        ctx.meta.agents
    } else {
        o.agents
    };
    let (world, k) = app.world(&o.city, &ctx, want)?;
    let all = epidemicsim_scenarios::load_all(&dirs.scenarios)?;
    for sc in &all {
        if o.scenarios.as_ref().is_some_and(|l| !l.contains(&sc.name)) {
            continue;
        }
        let t = Instant::now();
        let spec = SimSpec {
            days: o.days,
            seed: o.seed,
            backend: o.backend.clone(),
            threads: o.threads,
            scenario: sc.clone(),
            chains: false,
        };
        let mut sim = Sim::new(ctx.clone(), world.clone(), k, &spec)?;
        let mut frames = Vec::with_capacity(sim.total_steps as usize);
        while !sim.done() {
            frames.push(sim.step());
        }
        let dir = dirs.results.join(&o.city).join(&sc.name).join("frames");
        std::fs::create_dir_all(&dir)?;
        let meta = json!({ "grid": &*ctx.grid, "agents": sim.agents(), "city_agents": ctx.meta.agents,
            "subsample": k, "capacity": sim.capacity, "days": o.days, "seed": o.seed,
            "backend": sim.backend_name, "scenario": sc.name });
        std::fs::write(dir.join("meta.json"), serde_json::to_string(&meta)?)?;
        std::fs::write(dir.join("frames.json"), serde_json::to_string(&frames)?)?;
        eprintln!(
            "{:<14} {} frames in {:.1}s",
            sc.name,
            frames.len(),
            t.elapsed().as_secs_f64()
        );
    }
    Ok(())
}
