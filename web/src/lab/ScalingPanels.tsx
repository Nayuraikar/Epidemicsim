import { Bar, BarChart, ComposedChart, ErrorBar, Line } from 'recharts'
import { C } from '../colors'
import { macBench, okOf, useJson, useJsons, withCi, type Bench, type Machine } from './data'
import { Chart, Explain, f3, Finding, grid, legend, Need, Panel, PAL, Table, tip, Xax, Yax } from './ui'

interface WPoint { threads: number; agents: number; wall_s: { median: number; mean: number; ci95: number; n: number }; weak_efficiency: number; scaled_speedup: number }
interface Weak { backend: string; machine: Machine; per_thread_agents: number; days: number; steps: number; warmup: number; reps: number; seed: number; thermal_before?: string | null; thermal_after?: string | null; points: WPoint[]; gustafson?: { serial_fraction: number } }
interface MPoint { procs: number; tasks: number; wall_s: { median: number; mean: number; ci95: number; n: number }; weak_efficiency: number; scaled_speedup: number; comm_fraction: number }
interface WeakMpi { machine: string; backend: string; agents: number; days: number; tasks_per_rank: number; reps: number; note: string; points: MPoint[]; gustafson_serial_fraction?: number }

const ONE_M = 1_000_000
const WEAK = [
  { key: 'rayon', label: 'rayon guided', file: 'scaling/weak_rayon_guided_c64_apple_m1.json', strong: macBench('rayon_guided_c64') },
  { key: 'openmp', label: 'OpenMP guided', file: 'scaling/weak_openmp_guided_c64_apple_m1.json', strong: macBench('openmp_guided_c64') },
]
const gus = (t: number, a: number) => t - a * (t - 1)

/** Weak scaling for rayon and OpenMP with the Gustafson fit next to Amdahl (Performance tab). */
export function WeakCpuPanel() {
  const f = useJsons<Weak>({ rayon: WEAK[0].file, openmp: WEAK[1].file })
  const s = useJsons<Bench>({ rayon: WEAK[0].strong, openmp: WEAK[1].strong })
  const ws = WEAK.map((w, i) => ({ ...w, d: okOf(f[w.key]), strong: okOf(s[w.key]), color: PAL[i] })).filter((w) => w.d)
  const threads = [...new Set(ws.flatMap((w) => w.d!.points.map((p) => p.threads)))].sort((a, b) => a - b)
  const eff = threads.map((t) => ({ t, ideal: 1, ...Object.fromEntries(ws.flatMap((w) => { const p = w.d!.points.find((x) => x.threads === t); return [[w.key, p?.weak_efficiency]] })) }))
  const fit = threads.map((t) => ({ t, ideal: t, ...Object.fromEntries(ws.flatMap((w) => [[w.key, w.d!.points.find((x) => x.threads === t)?.scaled_speedup], [`${w.key}_fit`, w.d!.gustafson ? gus(t, w.d!.gustafson.serial_fraction) : undefined]])) }))
  const last = (w: (typeof ws)[number]) => w.d!.points[w.d!.points.length - 1]
  const am = (w: (typeof ws)[number]) => w.strong?.amdahl.find((a) => a.agents === ONE_M)?.serial_fraction
  return (
    <Need what="Weak-scaling runs (results/scaling/weak_*.json)" files={[f.rayon, f.openmp]}>
      <Explain>Strong scaling keeps the problem fixed and adds threads (Amdahl's law). Weak scaling keeps the work per thread fixed (here {ws[0] ? (ws[0].d!.per_thread_agents / 1000).toFixed(0) : '?'}k agents per thread) and grows the problem with the thread count; ideal is constant time. Gustafson's law, scaled speedup S(N) = N - alpha (N - 1), is fitted to those runs; alpha is the serial fraction of the scaled workload.</Explain>
      {ws.length > 0 && <Finding>At {last(ws[0]).threads} threads and {(last(ws[0]).agents / 1e6).toFixed(1)}M agents weak-scaling efficiency is {ws.map((w) => `${f3(last(w).weak_efficiency)} (${w.label})`).join(' and ')}. Gustafson alpha {ws.map((w) => `${f3(w.d!.gustafson?.serial_fraction)} (${w.label})`).join(', ')} versus the Amdahl serial fraction from the strong-scaling runs {ws.map((w) => `${f3(am(w))} (${w.label})`).join(', ')}: the two fits are of different experiments and the cores are 4 performance + 4 efficiency, so neither is a pure serial fraction.</Finding>}
      <div className="lab-grid2">
        <Panel title="Weak scaling: efficiency vs threads" settings={ws[0] && `${ws[0].d!.machine.cpu} · ${ws[0].d!.per_thread_agents.toLocaleString()} agents/thread · ${ws[0].d!.days} days (${ws[0].d!.steps} steps) · ${ws[0].d!.reps} reps (+${ws[0].d!.warmup} warm-up) · seed ${ws[0].d!.seed} · efficiency = time(1 thread) / time(N threads) on means`}>
          <Chart><ComposedChart data={eff}>{grid}<Xax dataKey="t" /><Yax domain={[0, 1.1]} />{tip}{legend}
            <Line dataKey="ideal" name="ideal" stroke={C.D} strokeDasharray="4 4" dot={false} isAnimationActive={false} />
            {ws.map((w) => <Line key={w.key} dataKey={w.key} name={w.label} stroke={w.color} strokeWidth={2} isAnimationActive={false} />)}</ComposedChart></Chart>
        </Panel>
        <Panel title="Scaled speedup and Gustafson fit" settings="same runs; dashed = S(N) = N - alpha (N - 1) with the fitted alpha">
          <Chart><ComposedChart data={fit}>{grid}<Xax dataKey="t" /><Yax />{tip}{legend}
            <Line dataKey="ideal" name="ideal" stroke={C.D} strokeDasharray="4 4" dot={false} isAnimationActive={false} />
            {ws.map((w) => <Line key={w.key} dataKey={w.key} name={`${w.label} (measured)`} stroke={w.color} strokeWidth={2} isAnimationActive={false} />)}
            {ws.map((w) => <Line key={`${w.key}_fit`} dataKey={`${w.key}_fit`} name={`${w.label} Gustafson`} stroke={w.color} strokeDasharray="5 3" dot={false} isAnimationActive={false} />)}</ComposedChart></Chart>
        </Panel>
      </div>
      <Panel title="Weak-scaling runs" settings={ws.map((w) => `${w.label}: thermal before: ${w.d!.thermal_before ?? 'n/a'}; after: ${w.d!.thermal_after ?? 'n/a'}`).join(' | ')}>
        <Table head={['variant', 'threads', 'agents', 'time s (mean ± 95% CI)', 'efficiency', 'scaled speedup']} rows={ws.flatMap((w) => w.d!.points.map((p) => [w.label, p.threads, p.agents.toLocaleString(), withCi(p.wall_s), f3(p.weak_efficiency), f3(p.scaled_speedup)]))} />
      </Panel>
    </Need>
  )
}

/** Weak scaling for MPI (work per rank fixed) with Gustafson next to the strong-scaling Amdahl fit. */
export function WeakMpiPanel() {
  const w = useJson<WeakMpi>('scaling/weak_mpi_apple_m1.json')
  const strong = useJson<Bench>(macBench('mpi'))
  const d = okOf(w), sb = okOf(strong)
  const rows = d ? d.points.map((p) => ({ n: p.procs, ideal: 1, efficiency: p.weak_efficiency })) : []
  const am = sb?.amdahl[0]?.serial_fraction
  return (
    <Need what="MPI weak scaling (scaling/weak_mpi_apple_m1.json)" files={[w]}>
      {d && (
        <>
          <Explain>Weak scaling for MPI: every rank always gets {d.tasks_per_rank} scenario runs (static mode), so the total workload grows with the process count. Ideal wall time stays flat.</Explain>
          <Finding>At {d.points[d.points.length - 1].procs} processes ({d.points[d.points.length - 1].tasks} runs) weak-scaling efficiency is {f3(d.points[d.points.length - 1].weak_efficiency)}. Gustafson alpha {f3(d.gustafson_serial_fraction)} versus Amdahl serial fraction {f3(am)} from the strong-scaling static runs; this machine has 4 performance + 4 efficiency cores, so ranks beyond 4 run on slower cores.</Finding>
          <div className="lab-grid2">
            <Panel title="MPI weak scaling: efficiency vs processes" settings={`${d.machine} · ${d.backend} backend · ${d.agents.toLocaleString()} agents · ${d.days} days · ${d.tasks_per_rank} runs per rank · ${d.reps} reps (+1 warm-up), mean`}>
              <Chart><ComposedChart data={rows}>{grid}<Xax dataKey="n" /><Yax domain={[0, 1.1]} />{tip}{legend}<Line dataKey="ideal" name="ideal" stroke={C.D} strokeDasharray="4 4" dot={false} isAnimationActive={false} /><Line dataKey="efficiency" name="static" stroke={C.S} strokeWidth={2} isAnimationActive={false} /></ComposedChart></Chart>
            </Panel>
            <Panel title="Wall time vs processes (95% CI)" settings="same runs; error bars = 95% CI of the mean">
              <Chart><BarChart data={d.points.map((p) => ({ n: p.procs, time: p.wall_s.mean, err: p.wall_s.ci95 }))}>{grid}<Xax dataKey="n" /><Yax unit=" s" />{tip}<Bar dataKey="time" name="wall time" fill={C.S} isAnimationActive={false}><ErrorBar dataKey="err" stroke={C.D} /></Bar></BarChart></Chart>
            </Panel>
          </div>
          <Panel title="Details" settings={d.note}>
            <Table head={['procs', 'runs', 'time s (mean ± 95% CI)', 'efficiency', 'scaled speedup', 'comm %']} rows={d.points.map((p) => [p.procs, p.tasks, withCi(p.wall_s), f3(p.weak_efficiency), f3(p.scaled_speedup), f3(p.comm_fraction * 100)])} />
          </Panel>
        </>
      )}
    </Need>
  )
}
