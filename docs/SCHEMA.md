# Schema

## Run output (`epidemicsim run --out <dir>`)

### `timeseries.parquet` — one row per step (4 steps/day: night, morning, day, evening; day 0 = Monday)

| column | type | meaning |
|---|---|---|
| step | u32 | global step index |
| day | u32 | step / 4 |
| S,E,A,I,H,R,D | u32 | agents per state at end of step (A = symptomless infectious, I = symptomatic) |
| new_infections | u32 | S→E transitions this step |
| hospital_load | u32 | agents in H |
| r_t | f64 | new_infections / infectiousness-weighted infectious count × expected infectious steps per case (0 when none infectious) |

### `summary.json`
backend, agents, days, seed, steps, peak_infectious (A+I) and day, peak_hospital_load and day,
attack_rate (1 − S_end/N), total_deaths, max_r_t, final_counts {S..D},
hash (FNV-1a 64 over per-step step/counts/new_infections; integers only).

## Config (TOML, all fields optional)
`seed, agents, days`, `[disease]`, `[vaccine]`, `[synth]`; defaults in `crates/core/src/params.rs`.

## Agents (SoA)
id, age_group (0: 0-17, 1: 18-39, 2: 40-59, 3: 60-74, 4: 75+), state (0..6 = S,E,A,I,H,R,D),
timer (steps left), home_loc, work_loc (work/school, = home if none), vax_status, vax_days.

## City TOML (`cities/<name>.toml`)
`name, geofabrik_url, bbox [min_lon,min_lat,max_lon,max_lat], target_agents (<=1M), hospital_beds_per_1000`,
optional `worldpop_tif` (north-up EPSG:4326 float GeoTIFF clipped to the city; default `data_cache/<name>_worldpop.tif`), `seed`.

## City folder (`epidemicsim pipeline <name>` -> `cities/<name>/`)
- `locations.parquet`: loc_id u32 (row index), kind u8 (0 home, 1 work/school, 2 market, 3 hospital), subtype u8
  (work only: 1 school, 2 college, 3 office), lon, lat f64, size f32 (home: residents, work: assigned agents,
  hospital: beds, market: 0), synthetic u8 (1 = topped up because OSM had too few).
  Ids are contiguous by kind (homes, schools, colleges, offices, markets, hospitals), Hilbert-ordered within kind.
- `agents.parquet`: id u32, age_group u8, home_loc u32, work_loc u32 (= home_loc if none). Sorted by home_loc.
- `meta.json`: name, source_url, bbox, seed, agents, pop_weight (worldpop | building_density), beds, counts, synthetic counts.

## Scenario TOML (`scenarios/<name>.toml`)
`name`, `description`, and `[[interventions]]` tables. Each has `kind` plus timing, either
`start_day = N` or `trigger = { metric, above }` (not both; neither = day 0), and optional `duration_days`.
Triggers fire once (checked daily on the previous step's stats) and stay on until `duration_days` ends.
Metrics: `infectious` = (A+I)/N, `hospital` = H/capacity, `cumulative` = 1 - S/N.

| kind | fields |
|---|---|
| `masks` | `compliance`, `efficacy` (beta x (1 - compliance x efficacy)) |
| `vaccination` | `strategy` = `random` / `age` / `location`, `coverage` (of population), `rate_per_day` (default 0.01), `ages` (for `age`, default [3,4]), `block_homes` (for `location`, default 1000: blocks of homes ranked by current infections) |
| `isolation` | `compliance` (symptomatic I stay home) |
| `college_closure`, `market_closure` | none (college closure is a no-op on synthetic cities, which have no colleges) |
| `lockdown` | none (closes work/school/college and markets) |

## Compare output (`epidemicsim compare --city X --scenarios all --runs 50`)
Run k of every scenario uses seed `base_seed + k` (paired comparison). Hospital capacity = city `total_beds`
(synthetic: `--beds-per-1000`, default 2).
- `results/<city>/<scenario>/bands.parquet`: step, day, then `<S|E|A|I|H|R|D|new_infections|hospital_load|r_t>_<mean|p05|p95>` per step.
- `results/<city>/<scenario>/summary.json`: config, interventions, `metrics` {name: {mean,p05,p95}}, `run_hashes`.
  Metrics: peak_infections (A+I), peak_day, attack_rate, deaths, peak_hospital_load, peak_hospital_ratio (vs capacity),
  days_over_capacity, r_initial (mean R_t, first 7 days), r_max (max daily-mean R_t), vaccine_doses.
- `results/<city>/comparison.json`: per scenario `metrics` and `vs_baseline` (relative change of means; needs `baseline`).

## Bench output (`epidemicsim bench --backend B`)
`results/bench/<backend>_<hardware>.json` (hardware = slug of CPU name): `machine` {os, arch, cpu, logical_cores, memory_gb},
`points` per (agents, threads): `wall_s` {median,min,max} for the whole `days`-day run, `agents_per_sec` (agents x steps / s),
`contacts_per_sec` (agent-location visits = alive agents x steps / s; not pairwise contacts), `speedup` and `efficiency`
vs the first thread count, and `amdahl` (least-squares serial fraction per agent count; needs >= 2 thread counts).
