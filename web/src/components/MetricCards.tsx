import { HELP } from '../helptext'
import { Help } from './Help'
import { METRICS, type MetricVals } from '../metrics'

export interface CardData { label: string; color: string; vals: MetricVals; source: string; base: MetricVals | null; isBase: boolean }

/** Delta vs baseline: lower is better for every metric here. */
function Delta({ x, b }: { x: number; b: number | undefined }) {
  if (b === undefined || b === null) return <div className="delta flat">–</div>
  if (b === 0) return <div className="delta flat">{x === 0 ? '±0%' : 'new'}</div>
  const d = (x - b) / b
  const cls = Math.abs(d) < 0.005 ? 'flat' : d < 0 ? 'good' : 'bad'
  return <div className={`delta ${cls}`}>{d > 0 ? '+' : ''}{(d * 100).toFixed(1)}% vs baseline</div>
}

const METRIC_HELP = { peak_infections: HELP.peakInfections, peak_day: HELP.peakDay, attack_rate: HELP.attackRate, deaths: HELP.deaths, days_over_capacity: HELP.daysOver } as const

export function MetricCards({ cards }: { cards: CardData[] }) {
  return (
    <div className="metrics" style={{ ['--n' as string]: cards.length }}>
      {cards.map((c, ci) => (
        <div className="card fade-in" key={c.label}>
          <div className="chart-title">
            <b data-export-text style={{ color: c.color }}>{c.label}</b><Help h={HELP.metricsDelta} once={ci > 0} />
            <span className="src">{c.source}</span>
          </div>
          <div className="metric-grid">
            {METRICS.map((m) => (
              <div className="metric" key={m.key}>
                <div className="label">{m.label}<Help h={METRIC_HELP[m.key]} once={ci > 0} /></div>
                <div className="value">{m.fmt(c.vals[m.key])}</div>
                {c.isBase ? <div className="delta flat">baseline</div> : m.key === 'peak_day' && c.base
                  ? <div className="delta flat">{c.vals.peak_day - c.base.peak_day >= 0 ? '+' : ''}{Math.round(c.vals.peak_day - c.base.peak_day)} d vs baseline</div>
                  : <Delta x={c.vals[m.key]} b={c.base?.[m.key]} />}
              </div>
            ))}
          </div>
        </div>
      ))}
    </div>
  )
}
