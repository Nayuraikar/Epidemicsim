import type { Run } from '../run'
import { C } from '../colors'

const STATE_COL: Record<string, string> = { S: C.S, E: C.E, A: C.I, I: C.I, H: C.I, R: C.R, D: C.D }
const AGE = ['0–17', '18–39', '40–59', '60–74', '75+']
const Pill = ({ s }: { s: string }) => <span className="pill" style={{ background: STATE_COL[s] }}>{s}</span>

export function Inspector({ run }: { run: Run | null }) {
  const a = run?.inspected
  return (
    <div className="card inspector fade-in">
      <div className="chart-title"><h3>Agent inspector</h3><span className="note">click a map cell during a live run</span></div>
      {!run && <div className="empty">No run yet.</div>}
      {run && !a && <div className="note">{run.inspectNote || (run.mode === 'replay' ? 'Replay frames hold no agent data. Start a live run (with “Record infection chains”) and click a map cell.' : 'Click a map cell to inspect an agent living there (an infected one if any).')}</div>}
      {a && (
        <>
          <div className="row" style={{ marginBottom: 8 }}>
            <b>Agent #{a.id}</b> <Pill s={a.state} /> <span className="note">age {AGE[a.age_group]} · {a.vaccinated ? 'vaccinated' : 'unvaccinated'} · lives at {a.home.lat.toFixed(4)}, {a.home.lon.toFixed(4)}{a.work ? ` · ${a.work.kind}` : ''}</span>
          </div>
          <table>
            <thead><tr><th>Slot</th><th>Weekday</th><th>Weekend</th></tr></thead>
            <tbody>
              {['night', 'morning', 'day', 'evening'].map((s) => (
                <tr key={s}><td>{s}</td>
                  {['weekday', 'weekend'].map((d) => <td key={d}>{a.schedule.find((x) => x.day_type === d && x.slot === s)?.kind}</td>)}</tr>
              ))}
            </tbody>
          </table>
          <div className="note">{a.schedule_note}</div>
          <h3 style={{ margin: '12px 0 4px' }}>Infection chain</h3>
          {!a.chains_recorded && <div className="note">Chains were not recorded for this run. Enable “Record infection chains” and rerun.</div>}
          {a.chains_recorded && a.chain.length === 0 && <div className="note">This agent has not been infected (yet).</div>}
          {a.chain.map((h, i) => (
            <div key={i} className="note" style={{ color: C.ink }}>
              {i === 0 ? 'Infected' : '↑ from'} <b>#{h.agent.id}</b> on day {h.day} at a {h.place.kind}
              {h.seed ? ' (initial seed case)' : h.sources.length ? ` — possible sources: ${h.sources.map((s) => `#${s.id} (${s.state})`).join(', ')}` : ''}
            </div>
          ))}
          {a.chains_recorded && <div className="note" style={{ marginTop: 6 }}>Possible onward infections: {a.infected_others.length ? a.infected_others.slice(0, 12).map((o) => `#${o.id}`).join(', ') + (a.infected_others.length > 12 ? '…' : '') : 'none recorded'}</div>}
          <div className="note" style={{ marginTop: 6 }}>{a.attribution_note}</div>
        </>
      )}
    </div>
  )
}
