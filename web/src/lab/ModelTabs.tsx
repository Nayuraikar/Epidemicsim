import { useState } from 'react'
import { Bar, BarChart, Cell, Line, LineChart, ReferenceLine } from 'recharts'
import { C } from '../colors'
import { okOf, useJson } from './data'
import { Chart, Explain, f3, Finding, grid, legend, Need, PAL, Panel, Select, Table, tip, Xax, Yax } from './ui'

type Out = { attack_rate: number; attack_rate_p05: number; attack_rate_p95: number; peak_infections: number; peak_infections_p05: number; peak_infections_p95: number }
type Row = { parameter: string; scenario: string; base_value: number; low_value: number; high_value: number; base: Out; low: Out; high: Out }
type Sens = { city: string; params: string; note: string; agents: number; days: number; runs: number; base_seed: number; backend: string; delta: number; rows: Row[] }
type Pt = { beta_scale: number; asym_scale: number; asym_share: number; growth_mean: number; doubling_days_mean: number; doubling_days_sd: number; attack_rate_at_horizon: number; loss: number }
type Grid = { note: string; agents: number; days: number; runs_per_point: number; base_seed: number; fit_window_days: [number, number]; target_doubling_days: number; target_asym_share: number; default_asym_share: number; default_doubling_days: number; stages: [string, Pt[]][]; best: Pt }

type Metric = 'attack_rate' | 'peak_infections'
const METRICS: { v: Metric; l: string }[] = [{ v: 'attack_rate', l: 'Attack rate' }, { v: 'peak_infections', l: 'Peak infections (A + I)' }]

/** Outcome in percent of the population, so both metrics share one unit. */
const pct = (o: Out, m: Metric, n: number) => (m === 'attack_rate' ? o.attack_rate * 100 : (o.peak_infections / n) * 100)

function Tornado() {
  const f = useJson<Sens>('sensitivity/synthetic.json')
  const d = okOf(f)
  const [m, setM] = useState<Metric>('attack_rate')
  const data = d
    ? d.rows
        .map((r) => ({
          name: `${r.parameter} (${r.scenario})`,
          lo: pct(r.low, m, d.agents) - pct(r.base, m, d.agents),
          hi: pct(r.high, m, d.agents) - pct(r.base, m, d.agents),
        }))
        .sort((a, b) => Math.max(Math.abs(b.lo), Math.abs(b.hi)) - Math.max(Math.abs(a.lo), Math.abs(a.hi)))
    : []
  const unit = ' pp'
  const top = data[0]
  const flat = data.filter((x) => Math.max(Math.abs(x.lo), Math.abs(x.hi)) < 0.5)
  const lab = METRICS.find((x) => x.v === m)!.l.toLowerCase()
  return (
    <Need what="Sensitivity run (sensitivity/synthetic.json)" files={[f]}>
      {d && top && (
        <>
          <Explain>Each parameter is moved by -{d.delta * 100}% and +{d.delta * 100}% of its central value, one at a time, and the change in the outcome is measured (percentage points of the population). Parameters that only act through an intervention (vaccine efficacy, mask efficacy, isolation compliance) are exercised in that intervention&apos;s scenario, so their bars are relative to that scenario&apos;s own central run, not to the baseline.</Explain>
          <Finding>
            For {lab}, the largest swing is {top.name}: {top.lo >= 0 ? '+' : ''}{f3(top.lo)} pp at -{d.delta * 100}% and {top.hi >= 0 ? '+' : ''}{f3(top.hi)} pp at +{d.delta * 100}%.
            {flat.length > 0 ? ` Under ${f3(0.5)} pp either way: ${flat.map((x) => x.name).join(', ')}.` : ''} Differences within the ensemble spread ({d.runs} runs) should not be read as real; see the table for p05-p95.
          </Finding>
          <Panel title="Tornado: change in outcome vs the central run" settings={`${d.city} city, ${d.agents.toLocaleString()} agents, ${d.days} days, ${d.runs} runs per setting (paired seeds from ${d.base_seed}), ${d.backend} backend, parameter set: ${d.params} · ±${d.delta * 100}% one at a time`}>
            <Select label="Outcome " value={m} onChange={setM} options={METRICS} />
            <Chart h={60 + data.length * 44}>
              <BarChart data={data} layout="vertical" stackOffset="sign" margin={{ left: 120 }}>
                {grid}<Xax type="number" unit={unit} /><Yax type="category" dataKey="name" width={240} />{tip}{legend}
                <ReferenceLine x={0} stroke={C.ink} />
                <Bar dataKey="lo" name={`-${d.delta * 100}% parameter`} fill={C.S} isAnimationActive={false}>{data.map((_, i) => <Cell key={i} fill={C.S} />)}</Bar>
                <Bar dataKey="hi" name={`+${d.delta * 100}% parameter`} fill={C.I} isAnimationActive={false}>{data.map((_, i) => <Cell key={i} fill={C.I} />)}</Bar>
              </BarChart>
            </Chart>
            <Table
              head={['parameter', 'scenario', 'central → -20% / +20%', 'central run', '-20% run', '+20% run']}
              rows={d.rows.map((r) => {
                const cell = (o: Out) => `${f3(pct(o, m, d.agents))}% (${f3(m === 'attack_rate' ? o.attack_rate_p05 * 100 : (o.peak_infections_p05 / d.agents) * 100)}-${f3(m === 'attack_rate' ? o.attack_rate_p95 * 100 : (o.peak_infections_p95 / d.agents) * 100)})`
                return [r.parameter, r.scenario, `${f3(r.base_value)} → ${f3(r.low_value)} / ${f3(r.high_value)}`, cell(r.base), cell(r.low), cell(r.high)]
              })}
            />
            <p className="note">{d.note} Cells: mean (p05-p95) across runs, percent of population.</p>
          </Panel>
        </>
      )}
    </Need>
  )
}

function Calibration() {
  const f = useJson<Grid>('calibration/grid.json')
  const g = okOf(f)
  const coarse = g?.stages.find((s) => s[0] === 'coarse')?.[1] ?? []
  const asyms = [...new Set(coarse.map((p) => p.asym_scale))]
  const betas = [...new Set(coarse.map((p) => p.beta_scale))]
  const data = betas.map((b) => ({ b, ...Object.fromEntries(asyms.map((a) => [`a${a}`, coarse.find((p) => p.beta_scale === b && p.asym_scale === a)?.doubling_days_mean])) }))
  return (
    <Need what="Calibration grid (calibration/grid.json)" files={[f]}>
      {g && (
        <>
          <Explain>A rough fit of the transmission scale and asymptomatic fraction to an <b>assumed, illustrative</b> target (early doubling time ≈ 5 days, asymptomatic share ≈ 40%). Neither number is a citation, and this is not a forecast.</Explain>
          <Finding>
            Default parameters double in {f3(g.default_doubling_days)} days; the best grid point (beta x{g.best.beta_scale}, asymptomatic x{g.best.asym_scale}) doubles in {f3(g.best.doubling_days_mean)} ± {f3(g.best.doubling_days_sd)} days (target {g.target_doubling_days}). Run-to-run sd of the doubling time is large at low beta, so the fit error is within noise.
          </Finding>
          <Panel title="Doubling time vs beta scale (coarse grid)" settings={`${g.agents.toLocaleString()} synthetic agents, ${g.runs_per_point} runs per grid point, base seed ${g.base_seed}, growth fitted over days ${g.fit_window_days[0]}-${g.fit_window_days[1]} (stopped at 5% cumulative attack), mean over runs`}>
            <Chart h={260}>
              <LineChart data={data}>{grid}<Xax dataKey="b" label={{ value: 'beta scale', position: 'insideBottom', offset: -2, fontSize: 11 }} /><Yax unit=" d" />{tip}{legend}
                <ReferenceLine y={g.target_doubling_days} stroke={C.D} strokeDasharray="4 3" label={{ value: 'assumed target', fontSize: 11, fill: C.muted }} />
                {asyms.map((a, i) => <Line key={a} dataKey={`a${a}`} name={`asymptomatic x${a}`} stroke={PAL[i % PAL.length]} dot isAnimationActive={false} />)}
              </LineChart>
            </Chart>
          </Panel>
        </>
      )}
    </Need>
  )
}

type CityArm = { city: string; cum_frac_mean: number[]; start_days: (number | null)[]; reached: number; start_median: number | null; attack_rate: { mean: number }; imports_mean: number }
type Paired = { city: string; earlier: number; later: number; same: number; only_with_travel: number; only_without_travel: number; mean_shift_days: number | null }
type Travel = {
  note: string; params: string; scenario: string; cities: string[]; seeded: boolean[]; agents: number[]; runs: number; days: number; base_seed: number
  start_threshold: number; other_initial_frac: number; symptomatic_factor: number; flows: { from: string; to: string; per_agent_per_day: number }[]
  without_travel: { cities: CityArm[] }; with_travel: { cities: CityArm[] }; paired: Paired[]
}

const dayTxt = (v: number | null | undefined) => (v == null ? 'never' : `day ${f3(v)}`)

function TravelCompare() {
  const f = useJson<Travel>('travel/calibrated_baseline.json')
  const d = okOf(f)
  const others = d ? d.cities.filter((_, i) => !d.seeded[i]) : []
  const [pick, setPick] = useState('')
  const city = pick && others.includes(pick) ? pick : others[0]
  const ci = d && city ? d.cities.indexOf(city) : -1
  const data = d && ci >= 0 ? d.with_travel.cities[ci].cum_frac_mean.map((v, day) => ({ day, with: v * 100, without: d.without_travel.cities[ci].cum_frac_mean[day] * 100 })) : []
  const pr = d && ci >= 0 ? d.paired[ci] : null
  const src = d ? d.cities.filter((_, i) => d.seeded[i]).join(', ') : ''
  return (
    <Need what="Inter-city travel run (travel/calibrated_baseline.json)" files={[f]}>
      {d && pr && ci >= 0 && (
        <>
          <Explain><b>Assumed, illustrative flows.</b> Each day an infectious agent (symptomless: full rate, symptomatic: ×{d.symptomatic_factor}) may travel to another city and infects one random susceptible resident there. The per-agent trip rates in config/travel.toml are guesses, not commuter data, and nothing here is a forecast. {src} starts with the normal initial infections; the other cities start with a trace ({f3(d.other_initial_frac * 100)}% of agents, at least one). Both arms use identical seeds, so the only difference is travel.</Explain>
          <Finding>
            In {city}, the epidemic (cumulative infected ≥ {f3(d.start_threshold * 100)}%) started {dayTxt(d.without_travel.cities[ci].start_median)} (median of {d.without_travel.cities[ci].reached}/{d.runs} runs that got there) without travel and {dayTxt(d.with_travel.cities[ci].start_median)} ({d.with_travel.cities[ci].reached}/{d.runs}) with travel. Paired by seed: earlier in {pr.earlier}, later in {pr.later}, same day in {pr.same} of {d.runs} runs{pr.mean_shift_days != null ? `; mean shift ${f3(pr.mean_shift_days)} days (negative = earlier)` : ''}. {pr.earlier > pr.later ? 'Travel brought the start forward.' : pr.later > pr.earlier ? 'Travel did not bring the start forward here.' : 'No consistent effect on the start day.'}
          </Finding>
          <Panel title={`Cumulative infected in ${city}, with vs without travel`} settings={`${d.cities.join(' + ')} (${d.agents.map((a) => a.toLocaleString()).join(' / ')} agents), scenario ${d.scenario}, ${d.runs} runs per arm (paired seeds from ${d.base_seed}), ${d.days} days, parameter set: ${d.params}, mean over runs`}>
            <Select label="City " value={city} onChange={setPick} options={others.map((c) => ({ v: c, l: c }))} />
            <Chart h={280}>
              <LineChart data={data}>{grid}<Xax dataKey="day" label={{ value: 'day', position: 'insideBottom', offset: -2, fontSize: 11 }} /><Yax unit="%" />{tip}{legend}
                <ReferenceLine y={d.start_threshold * 100} stroke={C.D} strokeDasharray="4 3" label={{ value: 'epidemic start', fontSize: 11, fill: C.muted }} />
                <Line dataKey="without" name="without travel" stroke={C.S} dot={false} isAnimationActive={false} />
                <Line dataKey="with" name="with travel" stroke={C.I} dot={false} isAnimationActive={false} />
              </LineChart>
            </Chart>
            <Table
              head={['city', 'start without', 'start with', 'earlier / later / same', 'mean imports per run', 'attack rate without → with']}
              rows={d.cities.map((c, i) => [c, dayTxt(d.without_travel.cities[i].start_median), dayTxt(d.with_travel.cities[i].start_median), `${d.paired[i].earlier} / ${d.paired[i].later} / ${d.paired[i].same}`, f3(d.with_travel.cities[i].imports_mean), `${f3(d.without_travel.cities[i].attack_rate.mean * 100)}% → ${f3(d.with_travel.cities[i].attack_rate.mean * 100)}%`])}
            />
            <p className="note">{d.note} Imports = arrivals that hit a susceptible resident. Assumed daily trip rates: {d.flows.map((x) => `${x.from}→${x.to} ${x.per_agent_per_day}`).join(', ')}.</p>
          </Panel>
        </>
      )}
    </Need>
  )
}

export function ModelTab() {
  return (
    <>
      <h2 className="lab-h">Calibration (rough, assumed target)</h2>
      <Calibration />
      <h2 className="lab-h">Sensitivity</h2>
      <Tornado />
      <h2 className="lab-h">Inter-city travel (assumed flows)</h2>
      <TravelCompare />
    </>
  )
}
