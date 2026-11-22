import type { ReactNode } from 'react'
import type { Builder as B } from '../scenario'

function Slider({ label, value, min, max, step, fmt, onChange, disabled }: { label: string; value: number; min: number; max: number; step: number; fmt: (v: number) => string; onChange: (v: number) => void; disabled?: boolean }) {
  return (
    <div className="field" style={{ opacity: disabled ? 0.45 : 1 }}>
      <label><span>{label}</span><b>{fmt(value)}</b></label>
      <input type="range" min={min} max={max} step={step} value={value} disabled={disabled} onChange={(e) => onChange(+e.target.value)} />
    </div>
  )
}
const pct = (v: number) => `${Math.round(v * 100)}%`
const day = (v: number) => `day ${v}`

function Toggle({ on, set, children }: { on: boolean; set: (v: boolean) => void; children: ReactNode }) {
  return <label className="toggle"><input type="checkbox" checked={on} onChange={(e) => set(e.target.checked)} /> <b>{children}</b></label>
}

interface Props {
  b: B
  set: (b: B) => void
  presets: string[]
  preset: string
  modified: boolean
  onPreset: (name: string) => void
  disabled: boolean
  canReplay: boolean
  replayHint: string
  liveAgents: number
  maxAgents: number
  setLiveAgents: (n: number) => void
  backend: string
  backends: string[]
  setBackend: (s: string) => void
  chains: boolean
  setChains: (v: boolean) => void
  onLive: () => void
  onReplay: () => void
  onExport: () => void
}

export function ScenarioBuilder(p: Props) {
  const { b, set, disabled } = p
  const u = (patch: Partial<B>) => set({ ...b, ...patch })
  return (
    <div className="card fade-in">
      <h3>Scenario</h3>
      <div className="field">
        <label><span>Preset</span>{p.modified && <span className="note">modified → custom</span>}</label>
        <select value={p.preset} onChange={(e) => p.onPreset(e.target.value)} disabled={disabled}>
          {p.presets.map((n) => <option key={n} value={n}>{n}</option>)}
        </select>
      </div>

      <div className="group">
        <Toggle on={b.masksOn} set={(v) => u({ masksOn: v })}>Masks</Toggle>
        <Slider label="Compliance" value={b.maskCompliance} min={0} max={1} step={0.05} fmt={pct} disabled={!b.masksOn || disabled} onChange={(v) => u({ maskCompliance: v })} />
        <Slider label="Start" value={b.maskStart} min={0} max={60} step={1} fmt={day} disabled={!b.masksOn || disabled} onChange={(v) => u({ maskStart: v })} />
      </div>
      <div className="group">
        <Toggle on={b.vaxOn} set={(v) => u({ vaxOn: v })}>Vaccination</Toggle>
        <div className="field">
          <label><span>Strategy</span></label>
          <select value={b.vaxStrategy} disabled={!b.vaxOn || disabled} onChange={(e) => u({ vaxStrategy: e.target.value as B['vaxStrategy'] })}>
            <option value="random">Random</option><option value="age">Age (60+)</option><option value="location">Location (hotspot blocks)</option>
          </select>
        </div>
        <Slider label="Coverage" value={b.vaxCoverage} min={0} max={1} step={0.05} fmt={pct} disabled={!b.vaxOn || disabled} onChange={(v) => u({ vaxCoverage: v })} />
        <Slider label="Rate / day" value={b.vaxRate} min={0.0025} max={0.05} step={0.0025} fmt={(v) => `${(v * 100).toFixed(2)}%`} disabled={!b.vaxOn || disabled} onChange={(v) => u({ vaxRate: v })} />
        <Slider label="Start" value={b.vaxStart} min={0} max={60} step={1} fmt={day} disabled={!b.vaxOn || disabled} onChange={(v) => u({ vaxStart: v })} />
      </div>
      <div className="group">
        <Toggle on={b.isoOn} set={(v) => u({ isoOn: v })}>Isolation of symptomatic</Toggle>
        <Slider label="Compliance" value={b.isoCompliance} min={0} max={1} step={0.05} fmt={pct} disabled={!b.isoOn || disabled} onChange={(v) => u({ isoCompliance: v })} />
        <Slider label="Start" value={b.isoStart} min={0} max={60} step={1} fmt={day} disabled={!b.isoOn || disabled} onChange={(v) => u({ isoStart: v })} />
      </div>
      <div className="group">
        <h3 style={{ marginTop: 10 }}>Closures</h3>
        <Toggle on={b.collegeClosure} set={(v) => u({ collegeClosure: v })}>College closure</Toggle>
        <Toggle on={b.marketClosure} set={(v) => u({ marketClosure: v })}>Market closure</Toggle>
        <Toggle on={b.lockdown} set={(v) => u({ lockdown: v })}>Lockdown (work, school, colleges, markets)</Toggle>
        <div className="field">
          <label><span>Trigger</span></label>
          <select value={b.closureMode} onChange={(e) => u({ closureMode: e.target.value as B['closureMode'] })}>
            <option value="threshold">When a metric crosses a threshold</option><option value="day">On a fixed day</option>
          </select>
        </div>
        {b.closureMode === 'day'
          ? <Slider label="Start" value={b.closureDay} min={0} max={90} step={1} fmt={day} onChange={(v) => u({ closureDay: v })} />
          : <>
              <div className="field">
                <select value={b.trigMetric} onChange={(e) => u({ trigMetric: e.target.value as B['trigMetric'] })}>
                  <option value="infectious">Infectious share of population</option><option value="hospital">Hospital load / capacity</option><option value="cumulative">Cumulative infected share</option>
                </select>
              </div>
              <Slider label="Threshold" value={b.trigAbove} min={0.001} max={b.trigMetric === 'infectious' ? 0.05 : 1} step={b.trigMetric === 'infectious' ? 0.001 : 0.01} fmt={(v) => `${(v * 100).toFixed(1)}%`} onChange={(v) => u({ trigAbove: v })} />
            </>}
        <Slider label="Duration (0 = until end)" value={b.closureDuration} min={0} max={120} step={5} fmt={(v) => (v ? `${v} d` : '∞')} onChange={(v) => u({ closureDuration: v })} />
      </div>

      <div className="group">
        <h3 style={{ marginTop: 10 }}>Live engine</h3>
        <div className="field">
          <label><span>Backend</span></label>
          <select value={p.backend} onChange={(e) => p.setBackend(e.target.value)}>{p.backends.map((n) => <option key={n}>{n}</option>)}</select>
        </div>
        <Slider label="Agents (capped)" value={p.liveAgents} min={10000} max={p.maxAgents} step={10000} fmt={(v) => `${(v / 1000).toFixed(0)}k`} onChange={p.setLiveAgents} />
        <Toggle on={p.chains} set={p.setChains}>Record infection chains</Toggle>
      </div>

      <div className="row" style={{ marginTop: 12 }}>
        <button className="btn primary" disabled={disabled} onClick={p.onLive}>Run live</button>
        <button className="btn" disabled={disabled || !p.canReplay} onClick={p.onReplay} title={p.replayHint}>Replay</button>
        <button className="btn" onClick={p.onExport}>Export JSON</button>
      </div>
      {!p.canReplay && !disabled && <div className="note" style={{ marginTop: 6 }}>{p.replayHint}</div>}
    </div>
  )
}
