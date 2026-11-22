import { useEffect, useState } from 'react'
import Nav from './components/Nav'
import { api } from './api'
import { kgBench, okOf, useJson, type Bench } from './lab/data'
import type { CitiesResp } from './types'

type MetalRows = { rows: { agents: number; speedup_vs_best_cpu: number; best_cpu: string; steps: number }[]; note: string }
const dash = '–'

function Figure({ label, value, unit, cap }: { label: string; value: string; unit?: string; cap: string }) {
  return (
    <div className="figure">
      <span className="label">{label}</span>
      <div className="num">{value}{unit && <small style={{ fontSize: '.5em' }}>{unit}</small>}</div>
      <div className="cap">{cap}</div>
    </div>
  )
}

export default function Landing() {
  const metal = useJson<MetalRows>('gpu/metal_vs_cpu.json')
  const cuda = useJson<Bench>(kgBench('cuda')), kseq = useJson<Bench>(kgBench('sequential'))
  const [cities, setCities] = useState<CitiesResp | null>(null)
  useEffect(() => { api.cities().then(setCities).catch(() => setCities(null)) }, [])

  const mrows = okOf(metal)?.rows ?? []
  const mbest = mrows.reduce<(typeof mrows)[number] | null>((b, r) => (!b || r.speedup_vs_best_cpu > b.speedup_vs_best_cpu ? r : b), null)
  const c = okOf(cuda), s = okOf(kseq)
  const cp = c?.points.find((p) => p.agents === 1_000_000), sp = s?.points.find((p) => p.agents === 1_000_000)
  const built = cities?.cities.filter((x) => x.built) ?? []
  const agents = built.length ? Math.min(...built.map((x) => x.agents ?? 0)) : 0
  const mil = (n: number) => (n >= 1e6 ? `${(n / 1e6).toFixed(n % 1e6 ? 1 : 0)}M` : `${(n / 1e3).toFixed(0)}k`)

  return (
    <div className="landing">
      <header className="topbar"><div className="brand">epidemicsim<small>agent-based epidemics, parallel backends</small></div><span className="spacer" /><Nav active="home" /></header>
      <section className="hero">
        <h1>Simulate an epidemic across a whole city, then see how fast each kind of parallelism can do it.</h1>
        <p>One agent-based model (1M people, 4 steps a day) run by sequential, multithreaded, SIMD, Metal, CUDA and MPI backends that all produce identical results.</p>
        <div className="cta">
          <a className="btn primary big" href="#/demo">One-click demo</a>
          <span className="note">Replays the pre-baked Mumbai combined-interventions run.</span>
        </div>
      </section>
      <section className="figures" aria-label="Headline figures">
        <Figure label="Best Metal speedup" value={mbest ? mbest.speedup_vs_best_cpu.toFixed(2) : dash} unit="×"
          cap={mbest ? `M1 GPU vs ${mbest.best_cpu.replace('_', ' ')}, ${mil(mbest.agents)} agents, ${mbest.steps} steps, median of 5` : 'results/gpu/metal_vs_cpu.json not found'} />
        <Figure label="CUDA speedup, T4" value={cp && sp ? (sp.wall_s.median / cp.wall_s.median).toFixed(1) : dash} unit="×"
          cap={cp && sp && c ? `vs Xeon sequential, 1M agents, ${c.days} days, median of ${c.reps}` : 'results/kaggle/bench not found'} />
        <Figure label="Backends registered" value={cities ? String(cities.backends.length) : dash}
          cap="golden tests compare each to sequential: per-step stats and run hash; see Lab, Summary" />
        <Figure label="Cities built" value={cities ? String(built.length) : dash}
          cap={built.length ? `${built.map((x) => x.name).join(', ')} · ${mil(agents)} agents each (min)` : 'server API not reachable'} />
      </section>
      <div className="sections">
        <section className="lsec">
          <span className="label">01 Studio</span>
          <h2>Studio</h2>
          <p>Pick a city, build an intervention scenario (masks, vaccination, isolation, closures, lockdown), run it live or replay it, and compare runs on maps and SEIR curves.</p>
          <a href="#/studio">Open the Studio →</a>
        </section>
        <section className="lsec">
          <span className="label">02 Lab</span>
          <h2>PAGP Lab</h2>
          <p>Measured results for eight parallel-computing topics: speedup and Amdahl, ILP, threads and schedules, SIMD, GPUs, warps, PTX, and MPI, plus a backend x hardware matrix.</p>
          <a href="#/lab">Open the Lab →</a>
        </section>
      </div>
    </div>
  )
}
