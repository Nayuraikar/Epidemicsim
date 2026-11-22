import { useState } from 'react'
import { IlpTab, MpiTab, PerformanceTab, SimdTab, TlpTab } from './CpuTabs'
import Nav from '../components/Nav'
import { GpuTab, PtxTab, WarpTab } from './GpuTabs'
import { MemoryTab } from './MemTabs'
import { ModelTab } from './ModelTabs'
import { SummaryTab } from './Summary'

const TABS: { id: string; label: string; title?: string; el: () => React.ReactElement }[] = [
  { id: 'perf', label: 'Performance', el: PerformanceTab }, { id: 'ilp', label: 'ILP', el: IlpTab }, { id: 'tlp', label: 'TLP', el: TlpTab },
  { id: 'simd', label: 'SIMD', el: SimdTab }, { id: 'gpu', label: 'GPU', el: GpuTab }, { id: 'warps', label: 'Warps', el: WarpTab },
  { id: 'ptx', label: 'PTX', el: PtxTab }, { id: 'mem', label: 'Memory', el: MemoryTab }, { id: 'mpi', label: 'MPI', el: MpiTab }, { id: 'model', label: 'Model', el: ModelTab }, { id: 'sum', label: 'Summary', el: SummaryTab },
]
export default function Lab() {
  const [tab, setTab] = useState(() => { const q = new URLSearchParams(location.hash.split('?')[1] ?? '').get('tab'); if (q) return q; try { return localStorage.getItem('lab-tab') || 'perf' } catch { return 'perf' } })
  const pick = (id: string) => { setTab(id); try { localStorage.setItem('lab-tab', id) } catch { /* ignore */ } }
  const Cur = (TABS.find((t) => t.id === tab) ?? TABS[0]).el
  const idx = Math.max(0, TABS.findIndex((t) => t.id === tab))
  const num = (i: number) => String(i + 1).padStart(2, '0')
  return (
    <div className="lab">
      <header className="topbar">
        <div className="brand">epidemicsim<small>PAGP lab</small></div>
        <span className="spacer" /><Nav active="lab" />
      </header>
      <div className="lab-layout">
        <nav className="lab-side" role="tablist" aria-label="Lab section">
          {TABS.map((t, i) => <button key={t.id} role="tab" aria-selected={t.id === tab} className={t.id === tab ? 'on' : ''} onClick={() => pick(t.id)}><i>{num(i)}</i>{t.label}</button>)}
        </nav>
        <main className="lab-main" key={tab}>
          <div className="lab-head"><i>{num(idx)}</i><h2>{TABS[idx].title ?? TABS[idx].label}</h2></div>
          <Cur />
        </main>
      </div>
    </div>
  )
}
