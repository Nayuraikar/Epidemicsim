import { useState } from 'react'
import { IlpTab, MpiTab, PerformanceTab, SimdTab, TlpTab } from './CpuTabs'
import { GpuTab, PtxTab, WarpTab } from './GpuTabs'
import { MemoryTab } from './MemTabs'
import { ModelTab } from './ModelTabs'
import { SummaryTab } from './Summary'

const TABS: { id: string; label: string; el: () => React.ReactElement }[] = [
  { id: 'perf', label: 'Performance', el: PerformanceTab }, { id: 'ilp', label: 'ILP', el: IlpTab }, { id: 'tlp', label: 'TLP', el: TlpTab },
  { id: 'simd', label: 'SIMD', el: SimdTab }, { id: 'gpu', label: 'GPU', el: GpuTab }, { id: 'warps', label: 'Warps', el: WarpTab },
  { id: 'ptx', label: 'PTX', el: PtxTab }, { id: 'mem', label: 'Memory', el: MemoryTab }, { id: 'mpi', label: 'MPI', el: MpiTab }, { id: 'model', label: 'Model', el: ModelTab }, { id: 'sum', label: 'Summary', el: SummaryTab },
]
export default function Lab() {
  const [tab, setTab] = useState(() => { try { return localStorage.getItem('lab-tab') || 'perf' } catch { return 'perf' } })
  const pick = (id: string) => { setTab(id); try { localStorage.setItem('lab-tab', id) } catch { /* ignore */ } }
  const Cur = (TABS.find((t) => t.id === tab) ?? TABS[0]).el
  return (
    <div className="lab">
      <header className="topbar">
        <div className="brand">epidemicsim<small>PAGP lab</small></div>
        <nav className="chips" role="tablist" aria-label="Lab section">
          {TABS.map((t) => <button key={t.id} role="tab" aria-selected={t.id === tab} className={`chip ${t.id === tab ? 'on' : ''}`} onClick={() => pick(t.id)}>{t.label}</button>)}
        </nav>
        <span className="spacer" />
        <a className="btn" href="#/">Home</a><a className="btn" href="#/studio">Studio</a>
      </header>
      <main className="lab-main" key={tab}><Cur /></main>
    </div>
  )
}
