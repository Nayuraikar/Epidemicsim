import { useCallback, useEffect, useMemo, useRef, useState } from 'react'
import { api } from './api'
import { C } from './colors'
import { exportComparisonPng, exportScenarioJson } from './export'
import { LiveConn } from './live'
import { baselineFromComparison, dailyFromBands, dailyFromFrames, metricsFromEnsemble, metricsFromFrames, type MetricVals } from './metrics'
import { pushFrame, type Run } from './run'
import { EMPTY, fromScenario, sameBuilder, toScenario, type Builder } from './scenario'
import type { CitiesResp, CityInfo, Comparison, ScenarioSpec } from './types'
import { ScenarioBuilder } from './components/Builder'
import { HospitalChart, RtChart, SeirChart, type Series } from './components/Charts'
import { Hotspots } from './components/Hotspots'
import { Inspector } from './components/Inspector'
import { MapPanel, snapshotMaps } from './components/MapPanel'
import Nav from './components/Nav'
import { MetricCards, type CardData } from './components/MetricCards'
import { Transport } from './components/Transport'
import { GuideContext, Help } from './components/Help'
import { HELP } from './helptext'

const MAX_RUNS = 3
let nextId = 1

export default function App({ demo = false }: { demo?: boolean }) {
  const [cities, setCities] = useState<CitiesResp | null>(null)
  const [presets, setPresets] = useState<ScenarioSpec[]>([])
  const [loadErr, setLoadErr] = useState('')
  const [city, setCity] = useState('')
  const [comparisons, setComparisons] = useState<Record<string, Comparison | null>>({})
  const [b, setB] = useState<Builder>(EMPTY)
  const [preset, setPreset] = useState('baseline')
  const [liveAgents, setLiveAgents] = useState(100000)
  const [backend, setBackend] = useState('rayon')
  const [chains, setChains] = useState(true)
  const runsRef = useRef<Run[]>([])
  const [, bump] = useState(0)
  const [step, setStep] = useState(0)
  const [playing, setPlaying] = useState(true)
  const [speed, setSpeed] = useState(5)
  const [follow, setFollow] = useState(true)
  const [busy, setBusy] = useState(false)
  const [focus, setFocus] = useState(0)
  const [guide, setGuide] = useState(false)
  const exportRef = useRef<HTMLDivElement>(null)
  const refresh = useCallback(() => bump((v) => v + 1), [])
  const runs = runsRef.current

  // ---- initial load ----
  useEffect(() => {
    Promise.all([api.cities(), api.scenarios()])
      .then(([c, s]) => {
        setCities(c); setPresets(s); setLiveAgents(c.default_live_agents)
        setBackend(c.backends.includes('rayon') ? 'rayon' : c.backends[0])
        const first = c.cities.find((x) => x.built) ?? c.cities[0]
        if (first) setCity(first.name)
        const base = s.find((x) => x.name === 'baseline')
        if (base) setB(fromScenario(base))
      })
      .catch((e) => setLoadErr(String(e.message ?? e)))
  }, [])

  const info: CityInfo | undefined = cities?.cities.find((c) => c.name === city)

  const clearRuns = useCallback(() => {
    runsRef.current.forEach((r) => r.conn?.close())
    runsRef.current = []
    setStep(0); setFocus(0); refresh()
  }, [refresh])

  const pickCity = (name: string) => { if (name === city) return; clearRuns(); setCity(name) }

  useEffect(() => {
    if (!city || !info?.built || city in comparisons) return
    api.comparison(city).then((c) => setComparisons((m) => ({ ...m, [city]: c }))).catch(() => setComparisons((m) => ({ ...m, [city]: null })))
  }, [city, info?.built, comparisons])

  // ---- builder <-> presets ----
  const presetSpec = presets.find((p) => p.name === preset)
  const presetB = presetSpec ? fromScenario(presetSpec) : null
  const baseSpec = presets.find((x) => x.name === 'baseline')
  const baseB = baseSpec ? fromScenario(baseSpec) : EMPTY
  const modified = !!presetSpec && !sameBuilder(b, fromScenario(presetSpec))
  const onPreset = (name: string) => { const p = presets.find((x) => x.name === name); if (p) { setPreset(name); setB(fromScenario(p)) } }
  const baked = info?.baked_scenarios ?? []
  const canReplay = !modified && baked.includes(preset)
  const replayHint = modified ? 'Replay only plays pre-baked presets: reset to a preset, or use Run live for custom settings.' : `No baked frames for “${preset}” in ${city}. Run: epidemicsim bake --city ${city} --scenarios ${preset}`
  const currentSpec = (): ScenarioSpec => toScenario(b, modified ? 'custom' : preset, modified ? 'Custom scenario from the studio' : presetSpec?.description ?? '')

  // ---- runs ----
  const addRun = (r: Run) => {
    while (runsRef.current.length >= MAX_RUNS) runsRef.current.shift()?.conn?.close()
    runsRef.current.push(r)
    setStep(0); setFollow(true); setPlaying(true); setFocus(runsRef.current.length - 1)
    refresh()
  }
  const blank = (mode: Run['mode'], label: string, sc: ScenarioSpec, presetName: string | null): Run => ({
    id: nextId++, city, label, mode, scenario: sc, preset: presetName, status: 'loading', message: mode === 'live' ? 'Connecting…' : 'Loading frames…',
    meta: null, frames: [], maxCell: 0, bands: null, ensemble: null, ensembleRuns: 0, conn: null, inspected: null, inspectNote: '',
  })

  const startReplay = () => {
    const r = blank('replay', `${preset} · replay`, currentSpec(), preset)
    addRun(r)
    const cmp = comparisons[city]
    const row = cmp?.scenarios.find((s) => s.scenario === preset)
    Promise.all([api.frames(city, preset), api.bands(city, preset).catch(() => null)])
      .then(([f, bands]) => {
        r.meta = f.meta; r.frames = []
        f.frames.forEach((x) => pushFrame(r, x))
        r.bands = bands; r.ensemble = row?.metrics ?? null; r.ensembleRuns = cmp?.runs ?? 0
        r.status = 'done'; refresh()
      })
      .catch((e) => { r.status = 'error'; r.message = String(e.message ?? e); refresh() })
  }

  const startLive = () => {
    const sc = currentSpec()
    const r = blank('live', `${sc.name} · live`, sc, modified ? null : preset)
    addRun(r)
    r.conn = new LiveConn({ city, scenario: sc, agents: liveAgents, days: 120, seed: 42, backend, chains, speed: speed * 4 }, {
      onStatus: (m) => { r.message = m; refresh() },
      onReady: (m) => { r.meta = m; r.status = 'running'; r.message = ''; refresh() },
      onFrame: (f) => { pushFrame(r, f) },
      onAgent: (a) => { r.inspected = a; r.inspectNote = ''; refresh() },
      onDone: (stopped) => { r.status = 'done'; r.message = stopped ? 'stopped' : ''; r.conn = null; refresh() },
      onError: (m) => {
        if (r.status === 'running' && r.meta) { r.inspectNote = m } else { r.status = 'error'; r.message = m }
        refresh()
      },
    })
  }

  // ---- one-click demo (#/demo): pick a built city with baked frames, replay combo/baseline automatically ----
  const [demoPhase, setDemoPhase] = useState(demo ? 0 : 9)
  const demoTarget = (() => {
    const cs = cities?.cities.filter((c) => c.built && c.baked_scenarios.length) ?? []
    const c = cs.find((x) => x.name === 'mumbai') ?? cs[0]
    return c ? { city: c.name, preset: ['combo', 'baseline'].find((p) => c.baked_scenarios.includes(p)) ?? c.baked_scenarios[0] } : null
  })()
  useEffect(() => {
    if (!cities || !presets.length || !city || demoPhase > 1) return
    if (!demoTarget) { setDemoPhase(9); return }
    if (demoPhase === 0) {
      if (city !== demoTarget.city) { setCity(demoTarget.city); return }
      onPreset(demoTarget.preset); setDemoPhase(1); return
    }
    if (city === demoTarget.city && preset === demoTarget.preset && city in comparisons) { setDemoPhase(2); startReplay() }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [demo, demoPhase, cities, presets, city, preset, comparisons])

  const removeRun = (id: number) => { const i = runs.findIndex((x) => x.id === id); if (i >= 0) { runs[i].conn?.close(); runs.splice(i, 1); setFocus(0); refresh() } }

  // ---- playback: rAF loop; live runs push frames, we flush ~30 fps and (when following) track the live edge ----
  const liveRuns = runs.filter((r) => r.conn)
  const maxStep = Math.max(-1, ...runs.map((r) => r.frames.length - 1))
  const stateRef = useRef({ playing, speed, follow, maxStep, hasLive: liveRuns.length > 0 })
  stateRef.current = { playing, speed, follow, maxStep, hasLive: liveRuns.length > 0 }
  useEffect(() => {
    let raf = 0, last = performance.now(), acc = 0
    const loop = (t: number) => {
      const s = stateRef.current
      const dt = (t - last) / 1000
      last = t
      acc += dt
      if (s.hasLive || runsRef.current.some((r) => r.status === 'loading')) {
        if (acc > 0.033) { acc = 0; if (s.follow && s.playing) setStep(s.maxStep); refresh() }
      } else if (s.playing && s.maxStep > 0) {
        setStep((p) => { const n = p + s.speed * 4 * dt; if (n >= s.maxStep) { setPlaying(false); return s.maxStep } return n })
      }
      raf = requestAnimationFrame(loop)
    }
    raf = requestAnimationFrame(loop)
    return () => cancelAnimationFrame(raf)
  }, [refresh])

  const togglePlay = () => {
    const next = !playing
    setPlaying(next)
    liveRuns.forEach((r) => (next ? r.conn?.resume() : r.conn?.pause()))
    if (next && !follow && step >= maxStep && maxStep > 0) setStep(0)
  }
  const changeSpeed = (v: number) => { setSpeed(v); liveRuns.forEach((r) => r.conn?.speed(v * 4)) }
  const stopAll = () => liveRuns.forEach((r) => r.conn?.stop())

  const pick = (r: Run, cell: number) => {
    if (!r.conn) { r.inspectNote = 'The inspector needs a live run (replay frames contain no agent data).'; refresh(); return }
    r.conn.inspect(cell)
  }

  // ---- derived chart/metric data ----
  const rows = runs.map((r, i) => ({ r, i, color: C.runs[i % 3] }))
  const chartData = useMemoCharts(rows.map((x) => x.r), runs.map((r) => r.frames.length + (r.bands ? 1e6 : 0)).join(','))
  const yMax = Math.max(1, ...chartData.map((d) => Math.max(...d.map((p) => Math.max(+p.S, +p.R, +p.E, +p.I, +p.D)))))
  const series: Series[] = rows.map((x, j) => ({
    label: x.r.label, color: x.color, data: chartData[j], banded: !!x.r.bands,
    capacity: x.r.bands && comparisons[x.r.city] ? comparisons[x.r.city]!.hospital_capacity : x.r.meta?.capacity ?? 0,
  }))
  const cmp = comparisons[city] ?? null
  const cards: CardData[] = rows.map((x) => {
    const r = x.r
    const useEns = !!r.ensemble
    const vals: MetricVals = useEns ? metricsFromEnsemble(r.ensemble!) : metricsFromFrames(r.frames, r.meta?.agents ?? 1, r.meta?.capacity ?? 0)
    let base: MetricVals | null = null
    if (useEns) base = baselineFromComparison(cmp)
    else {
      const b0 = runs.find((o) => o !== r && o.scenario.name === 'baseline' && !o.ensemble && o.meta && r.meta && o.meta.agents === r.meta.agents)
      if (b0 && b0.meta) base = metricsFromFrames(b0.frames, b0.meta.agents, b0.meta.capacity)
    }
    const isBase = r.scenario.name === 'baseline'
    return { label: r.label, color: x.color, vals, base, isBase, source: useEns ? `mean of ${r.ensembleRuns} runs` : r.mode === 'live' ? `single live run · ${r.meta ? (r.meta.agents / 1000).toFixed(0) + 'k agents' : ''}` : 'single baked run' }
  })

  const doExportPng = async () => {
    if (!exportRef.current) return
    setBusy(true)
    try { await exportComparisonPng(exportRef.current, await snapshotMaps(exportRef.current), `epidemicsim · ${city} · ${runs.map((r) => r.label).join(' vs ')}`) } finally { setBusy(false) }
  }

  const anyBuilt = !!info?.built
  const focusRun = runs[Math.min(focus, runs.length - 1)] ?? null

  if (loadErr) return <div className="app"><div className="card error-box" style={{ gridColumn: '1/-1' }}>Cannot reach the server API: {loadErr}. Start it with <code>epidemicsim serve</code>.</div></div>
  if (!cities) return <div className="app"><div className="card empty" style={{ gridColumn: '1/-1' }}><div className="spinner" />Loading…</div></div>

  return (
    <GuideContext.Provider value={guide}>
    <div className="app">
      <header className="topbar">
        <div className="brand">epidemicsim<small>studio</small></div>
        <Nav active="studio" />
        <div className="chips" role="tablist" aria-label="City">
          {cities.cities.map((c) => (
            <button key={c.name} role="tab" aria-selected={c.name === city} className={`chip ${c.name === city ? 'on' : ''} ${c.built ? '' : 'off'}`} onClick={() => pickCity(c.name)}>
              {c.name[0].toUpperCase() + c.name.slice(1)}{!c.built && <small>not built</small>}
            </button>
          ))}
        </div>
        <span className="spacer" />
        <button className="btn" disabled={!runs.length || busy} onClick={doExportPng}>{busy ? 'Exporting…' : 'Export comparison PNG'}</button><Help h={HELP.exportPng} once />
        <button className={`btn ${guide ? 'primary' : ''}`} aria-pressed={guide} onClick={() => setGuide((g) => !g)}>Guide</button>
      </header>

      <aside className="sidebar">
        <ScenarioBuilder b={b} ref0={presetB} base={baseB} set={setB} presets={presets.map((p) => p.name)} preset={preset} modified={modified} onPreset={onPreset}
          disabled={!anyBuilt} liveHint={anyBuilt && info && !info.has_agents ? info.build_command : ''} canReplay={canReplay} replayHint={replayHint} liveAgents={liveAgents} maxAgents={cities.max_live_agents} setLiveAgents={setLiveAgents}
          backend={backend} backends={cities.backends} setBackend={setBackend} chains={chains} setChains={setChains}
          onLive={startLive} onReplay={startReplay} onExport={() => exportScenarioJson(currentSpec())} />
      </aside>

      <main className="main" ref={exportRef}>
        {!anyBuilt && info && (
          <div className="card empty fade-in">
            <h2>{info.name[0].toUpperCase() + info.name.slice(1)} is not built yet</h2>
            <p>No data exists for this city yet. Build it with this command, then reload:</p>
            <pre className="lab-pre">{info.build_command}</pre>
          </div>
        )}
        {anyBuilt && (
          <>
            <div className="maps" style={{ ['--n' as string]: Math.max(runs.length, 1) }}>
              {runs.length === 0 && (
                <div className="card mapcard fade-in">
                  <div className="head"><b>{info?.name} · {info?.agents?.toLocaleString()} agents</b><Help h={HELP.heatmap} /><span className="note">idle</span></div>
                  <MapPanel run={null} frame={undefined} bbox={info?.bbox ?? null} onPick={pick} />
                  <div className="counts note">Choose a scenario and press <b>Run live</b>{baked.length ? <> or <b>Replay</b> ({baked.join(', ')} are pre-baked)</> : ''}.</div>
                </div>
              )}
              {rows.map(({ r, color }) => {
                const frame = r.frames[Math.min(Math.floor(step), r.frames.length - 1)]
                const c = frame?.counts
                return (
                  <div className="card mapcard fade-in" key={r.id} onClick={() => setFocus(runs.indexOf(r))} style={focusRun === r && runs.length > 1 ? { outline: `2px solid ${color}` } : undefined}>
                    <div className="head">
                      <b data-export-text style={{ color }}><span className="dot" style={{ background: color }} />{r.label}</b><Help h={HELP.heatmap} once={rows[0].r !== r} />
                      <button className="btn" onClick={(e) => { e.stopPropagation(); removeRun(r.id) }} aria-label="Remove run">✕</button>
                    </div>
                    <MapPanel run={r} frame={frame} bbox={info?.bbox ?? null} onPick={pick} />
                    {r.status === 'loading' && <div className="overlay"><div><div className="spinner" />{r.message}</div></div>}
                    {r.status === 'error' && <div className="overlay error"><div><b>Run failed</b><div>{r.message}</div></div></div>}
                    <div className="counts">
                      {c ? <>
                        <span><i className="dot" style={{ background: C.S }} />S {c[0].toLocaleString()}</span>
                        <span><i className="dot" style={{ background: C.E }} />E {c[1].toLocaleString()}</span>
                        <span><i className="dot" style={{ background: C.I }} />I {(c[2] + c[3]).toLocaleString()}</span>
                        <span><i className="dot" style={{ background: C.R }} />R {c[5].toLocaleString()}</span>
                        <span><i className="dot" style={{ background: C.D }} />D {c[6].toLocaleString()}</span>
                        <Help h={HELP.counts} once={rows[0].r !== r} />
                      </> : <span className="note">waiting for data…</span>}
                      {r.meta && r.meta.subsample > 1 && <span className="note">· {(r.meta.agents / 1000).toFixed(0)}k-agent sample of {(r.meta.city_agents / 1e6).toFixed(1)}M</span>}
                    </div>
                  </div>
                )
              })}
            </div>

            {runs.length > 0 && (
              <>
                <Transport step={step} maxStep={maxStep} playing={playing} speed={speed} live={liveRuns.length > 0} follow={follow}
                  onSeek={(s) => { setStep(s); setFollow(false) }} onPlay={togglePlay} onSpeed={changeSpeed}
                  onLive={() => { setFollow(true); setStep(maxStep) }} onStop={liveRuns.length ? stopAll : null} />
                <MetricCards cards={cards} />
                <div className="charts" style={{ ['--n' as string]: runs.length }}>
                  {rows.map((x, j) => (
                    <div className="card fade-in" key={x.r.id}>
                      <div className="chart-title"><b data-export-text style={{ color: x.color }}>{x.r.label}: SEIR{x.r.bands ? ' · 5–95% band' : ''}</b>{x.r.bands && <Help h={HELP.band} once={j > 0} />}</div>
                      <SeirChart data={chartData[j]} yMax={yMax} banded={!!x.r.bands} />
                    </div>
                  ))}
                </div>
                <div className="charts two">
                  <div className="card fade-in"><div className="chart-title"><b data-export-text>Hospital load vs capacity</b><Help h={HELP.hospital} /></div><HospitalChart series={series} /></div>
                  <div className="card fade-in"><div className="chart-title"><b data-export-text>R<sub>t</sub> (daily mean)</b><Help h={HELP.rt} /></div><RtChart series={series} /></div>
                </div>
                <div className="charts two">
                  <div className="card fade-in">
                    <div className="chart-title"><b data-export-text>Hotspots{focusRun ? ` · ${focusRun.label}` : ''}</b><Help h={HELP.hotspots} /><span className="note">top zones by new infections</span></div>
                    {focusRun && focusRun.meta ? <Hotspots run={focusRun} version={focusRun.frames.length} /> : <div className="empty">Waiting for data…</div>}
                  </div>
                  <Inspector run={focusRun} />
                </div>
              </>
            )}
          </>
        )}
      </main>
    </div>
    </GuideContext.Provider>
  )
}

/** Daily chart points per run; recomputed only when the run set or frame counts change. */
function useMemoCharts(rs: Run[], key: string) {
  return useMemo(() => rs.map((r) => (r.bands ? dailyFromBands(r.bands) : dailyFromFrames(r.frames))),
    // eslint-disable-next-line react-hooks/exhaustive-deps
    [key, rs.map((r) => r.id).join(',')])
}
