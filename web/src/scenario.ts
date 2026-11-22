import type { Intervention, ScenarioSpec, Trigger } from './types'

/** Slider/toggle model behind the scenario builder; converts to and from scenario TOML-shaped JSON. */
export interface Builder {
  masksOn: boolean; maskCompliance: number; maskEfficacy: number; maskStart: number
  vaxOn: boolean; vaxStrategy: 'random' | 'age' | 'location'; vaxCoverage: number; vaxRate: number; vaxStart: number
  isoOn: boolean; isoCompliance: number; isoStart: number
  collegeClosure: boolean; marketClosure: boolean; lockdown: boolean
  /** Closures and lockdown start when this metric is reached (or on closureDay when mode is 'day'). */
  closureMode: 'day' | 'threshold'; closureDay: number
  trigMetric: Trigger['metric']; trigAbove: number; closureDuration: number
}

export const EMPTY: Builder = {
  masksOn: false, maskCompliance: 0.6, maskEfficacy: 0.35, maskStart: 7,
  vaxOn: false, vaxStrategy: 'random', vaxCoverage: 0.4, vaxRate: 0.01, vaxStart: 10,
  isoOn: false, isoCompliance: 0.6, isoStart: 7,
  collegeClosure: false, marketClosure: false, lockdown: false,
  closureMode: 'threshold', closureDay: 14, trigMetric: 'infectious', trigAbove: 0.005, closureDuration: 30,
}

export function fromScenario(sc: ScenarioSpec): Builder {
  const b = { ...EMPTY }
  for (const iv of sc.interventions) {
    switch (iv.kind) {
      case 'masks': b.masksOn = true; b.maskCompliance = iv.compliance; b.maskEfficacy = iv.efficacy; b.maskStart = iv.start_day ?? 0; break
      case 'vaccination': b.vaxOn = true; b.vaxStrategy = iv.strategy; b.vaxCoverage = iv.coverage; b.vaxRate = iv.rate_per_day; b.vaxStart = iv.start_day ?? 0; break
      case 'isolation': b.isoOn = true; b.isoCompliance = iv.compliance; b.isoStart = iv.start_day ?? 0; break
      case 'college_closure': b.collegeClosure = true; timing(b, iv); break
      case 'market_closure': b.marketClosure = true; timing(b, iv); break
      case 'lockdown': b.lockdown = true; timing(b, iv); break
    }
  }
  return b
}

function timing(b: Builder, iv: Intervention) {
  if (iv.trigger) { b.closureMode = 'threshold'; b.trigMetric = iv.trigger.metric; b.trigAbove = iv.trigger.above }
  else { b.closureMode = 'day'; b.closureDay = iv.start_day ?? 0 }
  b.closureDuration = iv.duration_days ?? 0
}

export function toScenario(b: Builder, name: string, description = ''): ScenarioSpec {
  const closure = () => ({
    ...(b.closureMode === 'day' ? { start_day: b.closureDay } : { trigger: { metric: b.trigMetric, above: b.trigAbove } }),
    ...(b.closureDuration > 0 ? { duration_days: b.closureDuration } : {}),
  })
  const iv: Intervention[] = []
  if (b.masksOn) iv.push({ kind: 'masks', start_day: b.maskStart, compliance: b.maskCompliance, efficacy: b.maskEfficacy })
  if (b.isoOn) iv.push({ kind: 'isolation', start_day: b.isoStart, compliance: b.isoCompliance })
  if (b.vaxOn) iv.push({ kind: 'vaccination', strategy: b.vaxStrategy, start_day: b.vaxStart, coverage: b.vaxCoverage, rate_per_day: b.vaxRate })
  if (b.collegeClosure) iv.push({ kind: 'college_closure', ...closure() })
  if (b.marketClosure) iv.push({ kind: 'market_closure', ...closure() })
  if (b.lockdown) iv.push({ kind: 'lockdown', ...closure() })
  return { name, description, interventions: iv }
}

export const sameBuilder = (a: Builder, b: Builder) => JSON.stringify(toScenario(a, 'x')) === JSON.stringify(toScenario(b, 'x'))
