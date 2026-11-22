export interface CityInfo {
  name: string; built: boolean; agents: number | null; total_beds: number | null
  bbox: [number, number, number, number] | null; centroid: [number, number] | null
  has_comparison: boolean; baked_scenarios: string[]; has_agents: boolean; build_command: string
}
export interface CitiesResp { cities: CityInfo[]; max_live_agents: number; default_live_agents: number; backends: string[] }
export type Trigger = { metric: 'infectious' | 'hospital' | 'cumulative'; above: number }
export interface When { start_day?: number; trigger?: Trigger; duration_days?: number }
export type Intervention = When & (
  | { kind: 'masks'; compliance: number; efficacy: number }
  | { kind: 'vaccination'; strategy: 'random' | 'age' | 'location'; coverage: number; rate_per_day: number; ages?: number[]; block_homes?: number }
  | { kind: 'isolation'; compliance: number }
  | { kind: 'college_closure' } | { kind: 'market_closure' } | { kind: 'lockdown' })
export interface ScenarioSpec { name: string; description: string; interventions: Intervention[] }
export interface Grid { bbox: [number, number, number, number]; cols: number; rows: number; zones: string[] }
export interface RunMeta { grid: Grid; agents: number; city_agents: number; subsample: number; capacity: number; days: number; backend: string; scenario: string; chains?: boolean }
/** counts: S, E, A, I, H, R, D */
export interface Frame {
  step: number; day: number; counts: number[]; new_infections: number; hospital_load: number; r_t: number
  density: number[]; hot: number[]; kinds: number[]
}
export interface Dist { mean: number; p05: number; p95: number }
export interface CompareRow { scenario: string; metrics: Record<string, Dist>; vs_baseline: Record<string, number> | null }
export interface Comparison { city: string; runs: number; agents: number; days: number; hospital_capacity: number; scenarios: CompareRow[] }
export interface BandsResp { steps_per_day: number; day: number[]; series: Record<string, { mean: number[]; p05: number[]; p95: number[] }> }
export interface Place { loc: number; kind: string; lon: number; lat: number }
export interface Brief { id: number; state: string; age_group: number }
export interface Inspected {
  id: number; state: string; age_group: number; vaccinated: boolean; home: Place; work: Place | null
  schedule: (Place & { day_type: string; slot: string })[]; schedule_note: string; chains_recorded: boolean
  infection: unknown; chain: { agent: Brief; step: number; day: number; place: Place; sources: Brief[]; seed: boolean }[]
  infected_others: Brief[]; attribution_note: string
}
/** One daily point for charts (mean with optional 5-95% band). */
export interface Pt { day: number; [k: string]: number | [number, number] }
