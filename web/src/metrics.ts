import type { BandsResp, Comparison, Dist, Frame, Pt } from './types'

export const METRICS = [
  { key: 'peak_infections', label: 'Peak infections', fmt: (v: number) => Math.round(v).toLocaleString() },
  { key: 'peak_day', label: 'Peak day', fmt: (v: number) => `day ${Math.round(v)}` },
  { key: 'attack_rate', label: 'Attack rate', fmt: (v: number) => `${(v * 100).toFixed(1)}%` },
  { key: 'deaths', label: 'Deaths', fmt: (v: number) => Math.round(v).toLocaleString() },
  { key: 'days_over_capacity', label: 'Days over capacity', fmt: (v: number) => v.toFixed(v % 1 ? 1 : 0) },
] as const
export type MetricKey = (typeof METRICS)[number]['key']
export type MetricVals = Record<MetricKey, number>

export function metricsFromFrames(frames: Frame[], agents: number, capacity: number): MetricVals {
  let peak = 0, peakDay = 0, over = 0
  for (const f of frames) {
    const inf = f.counts[2] + f.counts[3]
    if (inf > peak) { peak = inf; peakDay = f.day }
    if (f.hospital_load > capacity) over++
  }
  const last = frames[frames.length - 1]
  return {
    peak_infections: peak, peak_day: peakDay,
    attack_rate: last ? 1 - last.counts[0] / Math.max(agents, 1) : 0,
    deaths: last ? last.counts[6] : 0, days_over_capacity: over / 4,
  }
}

export function metricsFromEnsemble(m: Record<string, Dist>): MetricVals {
  const g = (k: string) => m[k]?.mean ?? 0
  return { peak_infections: g('peak_infections'), peak_day: g('peak_day'), attack_rate: g('attack_rate'), deaths: g('deaths'), days_over_capacity: g('days_over_capacity') }
}

export function baselineFromComparison(c: Comparison | null): MetricVals | null {
  const row = c?.scenarios.find((s) => s.scenario === 'baseline')
  return row ? metricsFromEnsemble(row.metrics) : null
}

/** End-of-day state counts and daily-mean R_t from frames; no bands (single run). */
export function dailyFromFrames(frames: Frame[]): Pt[] {
  const out: Pt[] = []
  for (let i = 0; i < frames.length; i += 4) {
    const day = frames[i].day
    const end = frames[Math.min(i + 3, frames.length - 1)]
    const block = frames.slice(i, i + 4)
    const rt = block.reduce((s, f) => s + f.r_t, 0) / block.length
    out.push({ day, S: end.counts[0], E: end.counts[1], I: end.counts[2] + end.counts[3], R: end.counts[5], D: end.counts[6], hosp: end.hospital_load, rt })
  }
  return out
}

/** Daily points with 5-95% bands from ensemble bands. I = A + I (band edges are sums of the A and I band edges). */
export function dailyFromBands(b: BandsResp): Pt[] {
  const s = b.series
  const n = b.day.length
  const out: Pt[] = []
  const at = (k: string, q: 'mean' | 'p05' | 'p95', i: number) => s[k]?.[q][i] ?? 0
  const sum = (ks: string[], q: 'mean' | 'p05' | 'p95', i: number) => ks.reduce((t, k) => t + at(k, q, i), 0)
  for (let i = 3; i < n; i += 4) {
    const p: Pt = { day: b.day[i] }
    for (const [name, ks] of [['S', ['S']], ['E', ['E']], ['I', ['A', 'I']], ['R', ['R']], ['D', ['D']]] as const) {
      p[name] = sum([...ks], 'mean', i)
      p[`${name}_b`] = [sum([...ks], 'p05', i), sum([...ks], 'p95', i)]
    }
    p.hosp = at('hospital_load', 'mean', i)
    p.hosp_b = [at('hospital_load', 'p05', i), at('hospital_load', 'p95', i)]
    let r = 0, lo = 0, hi = 0
    for (let j = i - 3; j <= i; j++) { r += at('r_t', 'mean', j); lo += at('r_t', 'p05', j); hi += at('r_t', 'p95', j) }
    p.rt = r / 4
    p.rt_b = [lo / 4, hi / 4]
    out.push(p)
  }
  return out
}
