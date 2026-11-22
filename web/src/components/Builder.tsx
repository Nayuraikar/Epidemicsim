import type { ReactNode } from 'react'
import type { Builder as B } from '../scenario'
import { diffFromBaseline, sliderHint } from '../explain'
import { HELP } from '../helptext'
import { Help } from './Help'
import type { HelpText } from './Help'

function Slider({ label, value, min, max, step, fmt, onChange, disabled, help, hint }: { label: string; value: number; min: number; max: number; step: number; fmt: (v: number) => string; onChange: (v: number) => void; disabled?: boolean; help: HelpText; hint?: string | null }) {
  return (
    <div className="field" style={{ opacity: disabled ? 0.45 : 1 }}>
      <label><span>{label}</span><Help h={help} /><b>{fmt(value)}</b></label>
      <input type="range" min={min} max={max} step={step} value={value} disabled={disabled} onChange={(e) => onChange(+e.target.value)} />
      {hint && !disabled && <div className="hint">{hint}</div>}
    </div>
  )
}
const pct = (v: number) => `${Math.round(v * 100)}%`
const day = (v: number) => `day ${v}`

function Toggle({ on, set, children, help }: { on: boolean; set: (v: boolean) => void; children: ReactNode; help: HelpText }) {
  return <div className="toggle"><label><input type="checkbox" checked={on} onChange={(e) => set(e.target.checked)} /> <b>{children}</b></label><Help h={help} /></div>
}

interface Props {
  b: B
  /** Builder state of the selected preset and of the baseline preset (for change hints). */
  ref0: B | null
  base: B
  set: (b: B) => void
  presets: string[]
  preset: string
  modified: boolean
  onPreset: (name: string) => void
  disabled: boolean
  canReplay: boolean
  /** Non-empty when the city has no agent data: the command that builds it (Live is then blocked). */
  liveHint: string
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
  const r = p.ref0
  const n = p.preset
  const diff = diffFromBaseline(b, p.base)
  const anyClosure = b.collegeClosure || b.marketClosure || b.lockdown
  const dec = (v: number) => `${(v * 100).toFixed(2)}%`
  const pct1 = (v: number) => `${(v * 100).toFixed(1)}%`
  const dur = (v: number) => (v ? `${v} d` : '∞')
  return (
    <div className="card fade-in">
      <h3>Scenario</h3>
      <div className="field">
        <label><span>Preset</span><Help h={HELP.preset} />{p.modified && <span className="note">modified → custom</span>}</label>
        <select value={p.preset} onChange={(e) => p.onPreset(e.target.value)} disabled={disabled}>
          {p.presets.map((n) => <option key={n} value={n}>{n}</option>)}
        </select>
      </div>

      <div className="group">
        <Toggle on={b.masksOn} set={(v) => u({ masksOn: v })} help={HELP.masks(b.maskEfficacy)}>Masks</Toggle>
        <Slider label="Compliance" value={b.maskCompliance} min={0} max={1} step={0.05} fmt={pct} disabled={!b.masksOn || disabled} onChange={(v) => u({ maskCompliance: v })} help={HELP.maskCompliance} hint={sliderHint(r?.maskCompliance, b.maskCompliance, pct, 'stronger', 'weaker', n)} />
        <Slider label="Start" value={b.maskStart} min={0} max={60} step={1} fmt={day} disabled={!b.masksOn || disabled} onChange={(v) => u({ maskStart: v })} help={HELP.start} hint={sliderHint(r?.maskStart, b.maskStart, day, 'later', 'earlier', n)} />
      </div>
      <div className="group">
        <Toggle on={b.vaxOn} set={(v) => u({ vaxOn: v })} help={HELP.vax}>Vaccination</Toggle>
        <div className="field">
          <label><span>Strategy</span><Help h={HELP.vaxStrategy} /></label>
          <select value={b.vaxStrategy} disabled={!b.vaxOn || disabled} onChange={(e) => u({ vaxStrategy: e.target.value as B['vaxStrategy'] })}>
            <option value="random">Random</option><option value="age">Age (60+)</option><option value="location">Location (hotspot blocks)</option>
          </select>
        </div>
        <Slider label="Coverage" value={b.vaxCoverage} min={0} max={1} step={0.05} fmt={pct} disabled={!b.vaxOn || disabled} onChange={(v) => u({ vaxCoverage: v })} help={HELP.coverage} hint={sliderHint(r?.vaxCoverage, b.vaxCoverage, pct, 'stronger', 'weaker', n)} />
        <Slider label="Rate / day" value={b.vaxRate} min={0.0025} max={0.05} step={0.0025} fmt={dec} disabled={!b.vaxOn || disabled} onChange={(v) => u({ vaxRate: v })} help={HELP.rate} hint={sliderHint(r?.vaxRate, b.vaxRate, dec, 'faster', 'slower', n)} />
        <Slider label="Start" value={b.vaxStart} min={0} max={60} step={1} fmt={day} disabled={!b.vaxOn || disabled} onChange={(v) => u({ vaxStart: v })} help={HELP.start} hint={sliderHint(r?.vaxStart, b.vaxStart, day, 'later', 'earlier', n)} />
      </div>
      <div className="group">
        <Toggle on={b.isoOn} set={(v) => u({ isoOn: v })} help={HELP.iso}>Isolation of symptomatic</Toggle>
        <Slider label="Compliance" value={b.isoCompliance} min={0} max={1} step={0.05} fmt={pct} disabled={!b.isoOn || disabled} onChange={(v) => u({ isoCompliance: v })} help={HELP.isoCompliance} hint={sliderHint(r?.isoCompliance, b.isoCompliance, pct, 'stronger', 'weaker', n)} />
        <Slider label="Start" value={b.isoStart} min={0} max={60} step={1} fmt={day} disabled={!b.isoOn || disabled} onChange={(v) => u({ isoStart: v })} help={HELP.start} hint={sliderHint(r?.isoStart, b.isoStart, day, 'later', 'earlier', n)} />
      </div>
      <div className="group">
        <h3 style={{ marginTop: 10 }}>Closures</h3>
        <Toggle on={b.collegeClosure} set={(v) => u({ collegeClosure: v })} help={HELP.collegeClosure}>College closure</Toggle>
        <Toggle on={b.marketClosure} set={(v) => u({ marketClosure: v })} help={HELP.marketClosure}>Market closure</Toggle>
        <Toggle on={b.lockdown} set={(v) => u({ lockdown: v })} help={HELP.lockdown}>Lockdown (work, school, colleges, markets)</Toggle>
        <div className="field">
          <label><span>Trigger</span><Help h={HELP.trigMode} /></label>
          <select value={b.closureMode} onChange={(e) => u({ closureMode: e.target.value as B['closureMode'] })}>
            <option value="threshold">When a metric crosses a threshold</option><option value="day">On a fixed day</option>
          </select>
        </div>
        {b.closureMode === 'day'
          ? <Slider label="Start" value={b.closureDay} min={0} max={90} step={1} fmt={day} onChange={(v) => u({ closureDay: v })} help={HELP.closureDay} hint={anyClosure ? sliderHint(r?.closureDay, b.closureDay, day, 'later', 'earlier', n) : null} />
          : <>
              <div className="field">
                <label><span>Metric</span><Help h={HELP.trigMetric} /></label>
                <select value={b.trigMetric} onChange={(e) => u({ trigMetric: e.target.value as B['trigMetric'] })}>
                  <option value="infectious">Infectious share of population</option><option value="hospital">Hospital load / capacity</option><option value="cumulative">Cumulative infected share</option>
                </select>
              </div>
              <Slider label="Threshold" value={b.trigAbove} min={0.001} max={b.trigMetric === 'infectious' ? 0.05 : 1} step={b.trigMetric === 'infectious' ? 0.001 : 0.01} fmt={pct1} onChange={(v) => u({ trigAbove: v })} help={HELP.trigThreshold} hint={anyClosure && r?.trigMetric === b.trigMetric ? sliderHint(r?.trigAbove, b.trigAbove, pct1, 'triggers later', 'triggers earlier', n) : null} />
            </>}
        <Slider label="Duration (0 = until end)" value={b.closureDuration} min={0} max={120} step={5} fmt={dur} onChange={(v) => u({ closureDuration: v })} help={HELP.duration} hint={anyClosure ? sliderHint(r?.closureDuration, b.closureDuration, dur, 'longer', 'shorter', n) : null} />
      </div>

      <div className="group">
        <h3 style={{ marginTop: 10 }}>Live engine</h3>
        <div className="field">
          <label><span>Backend</span><Help h={HELP.backend} /></label>
          <select value={p.backend} onChange={(e) => p.setBackend(e.target.value)}>{p.backends.map((n) => <option key={n}>{n}</option>)}</select>
        </div>
        <Slider label="Agents (capped)" value={p.liveAgents} min={10000} max={p.maxAgents} step={10000} fmt={(v) => `${(v / 1000).toFixed(0)}k`} onChange={p.setLiveAgents} help={HELP.agents} />
        <Toggle on={p.chains} set={p.setChains} help={HELP.chains}>Record infection chains</Toggle>
      </div>

      <div className="note diff">{diff ?? 'Same as baseline (no interventions).'}</div>
      <div className="row actions" style={{ marginTop: 8 }}>
        <button className="btn primary" disabled={disabled || !!p.liveHint} onClick={p.onLive}>Run live</button><Help h={HELP.runLive} />
        <button className="btn" disabled={disabled || !p.canReplay} onClick={p.onReplay} title={p.replayHint}>Replay</button><Help h={HELP.replay} />
        <button className="btn" onClick={p.onExport}>Export JSON</button><Help h={HELP.exportJson} />
      </div>
      {p.liveHint && <div className="note" style={{ marginTop: 6 }}>Run live needs this city's agent data (not in the repo). Build it with: <code>{p.liveHint}</code> (Replay and the Lab work without it).</div>}
      {!p.canReplay && !disabled && <div className="note" style={{ marginTop: 6 }}>{p.replayHint}</div>}
    </div>
  )
}
