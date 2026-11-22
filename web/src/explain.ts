import type { Builder } from './scenario'

const pct = (v: number) => `${Math.round(v * 100)}%`

/** "preset 60% -> now 80% (stronger)" when `now` differs from the preset value, else null. */
export function sliderHint<T extends number>(ref: T | undefined, now: T, fmt: (v: number) => string, up: string, down: string, refName: string): string | null {
  if (ref === undefined || Math.abs(ref - now) < 1e-9) return null
  return `${refName} ${fmt(ref)} -> now ${fmt(now)} (${now > ref ? up : down})`
}

function closureWhen(b: Builder): string {
  const when = b.closureMode === 'day'
    ? `day ${b.closureDay}`
    : `${b.trigMetric} >= ${b.trigMetric === 'infectious' || b.trigMetric === 'cumulative' ? (b.trigAbove * 100).toFixed(1) + '%' : pct(b.trigAbove) + ' of capacity'}`
  return `${when}${b.closureDuration > 0 ? `, ${b.closureDuration} d` : ''}`
}

/** Every intervention setting as short text, computed from the builder state (empty string when off). */
function parts(b: Builder): Record<string, string> {
  return {
    masks: b.masksOn ? `masks on (${pct(b.maskCompliance)}, day ${b.maskStart})` : '',
    vaccination: b.vaxOn ? `vaccination on (${b.vaxStrategy}, ${pct(b.vaxCoverage)} at ${(b.vaxRate * 100).toFixed(2)}%/day, day ${b.vaxStart})` : '',
    isolation: b.isoOn ? `isolation on (${pct(b.isoCompliance)}, day ${b.isoStart})` : '',
    'college closure': b.collegeClosure ? `college closure on (${closureWhen(b)})` : '',
    'market closure': b.marketClosure ? `market closure on (${closureWhen(b)})` : '',
    lockdown: b.lockdown ? `lockdown on (${closureWhen(b)})` : '',
  }
}

/** One line listing how `b` differs from `base` (the baseline scenario), or null when it does not. */
export function diffFromBaseline(b: Builder, base: Builder): string | null {
  const now = parts(b), was = parts(base)
  const out: string[] = []
  for (const k of Object.keys(now)) {
    if (now[k] === was[k]) continue
    out.push(now[k] || `${k} off`)
  }
  return out.length ? `Differs from baseline: ${out.join(', ')}` : null
}
