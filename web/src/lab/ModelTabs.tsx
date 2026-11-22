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

export function ModelTab() {
  return (
    <>
      <h2 className="lab-h">Calibration (rough, assumed target)</h2>
      <Calibration />
      <h2 className="lab-h">Sensitivity</h2>
      <Tornado />
    </>
  )
}
