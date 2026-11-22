import { useMemo, useState } from 'react'
import { Bar, BarChart, ComposedChart, Line, LineChart } from 'recharts'
import { C } from '../colors'
import { kgBench, macBench, M1, XEON, okOf, pt, settingsOf, useJson, useJsons, useText, type Bench, type BPoint, type Machine } from './data'
import { big, Chart, Collapsible, Explain, f3, Finding, grid, legend, Need, NotRun, PAL, Panel, Pre, sec, Select, Table, tip, x2, Xax, Yax } from './ui'

type Jsons = Record<string, ReturnType<typeof useJson<Bench>>>
const VARIANTS = [
  { key: 'rayon_static_c0', label: 'rayon static', hw: M1, file: macBench('rayon_static_c0') },
  { key: 'rayon_dynamic_c1024', label: 'rayon dynamic', hw: M1, file: macBench('rayon_dynamic_c1024') },
  { key: 'rayon_guided_c64', label: 'rayon guided', hw: M1, file: macBench('rayon_guided_c64') },
  { key: 'openmp_static_c0', label: 'OpenMP static', hw: M1, file: macBench('openmp_static_c0') },
  { key: 'openmp_dynamic_c1024', label: 'OpenMP dynamic', hw: M1, file: macBench('openmp_dynamic_c1024') },
  { key: 'openmp_guided_c64', label: 'OpenMP guided', hw: M1, file: macBench('openmp_guided_c64') },
  { key: 'kg_rayon_guided_c64', label: 'rayon guided (Xeon)', hw: XEON, file: kgBench('rayon_guided_c64') },
]
const ONE_M = 1_000_000

// ------------------------------------------------------------------ Performance
export function PerformanceTab() {
  const f = useJsons<Bench>({
    seq: macBench('sequential'), kseq: kgBench('sequential'), metal: macBench('metal'), cuda: kgBench('cuda'),
    ...Object.fromEntries(VARIANTS.map((v) => [v.key, v.file])),
  }) as Jsons
  const [sel, setSel] = useState('rayon_guided_c64')
  const series = [
    { k: 'seq', l: 'sequential (M1)', b: okOf(f.seq) }, { k: 'kseq', l: 'sequential (Xeon)', b: okOf(f.kseq) },
    { k: 'metal', l: 'Metal (M1)', b: okOf(f.metal) }, { k: 'cuda', l: 'CUDA (T4)', b: okOf(f.cuda) },
  ].filter((s) => s.b)
  const sizes = [...new Set(series.flatMap((s) => s.b!.points.map((p) => p.agents)))].sort((a, b) => a - b)
  const rowsOf = (key: 'agents_per_sec' | 'contacts_per_sec') => sizes.map((n) => ({ n, ...Object.fromEntries(series.map((s) => [s.k, pt(s.b, n)?.[key]])) }))
  const vs = VARIANTS.map((v, i) => ({ ...v, b: okOf(f[v.key]), color: PAL[i % PAL.length] })).filter((v) => v.b)
  const threads = [...new Set(vs.flatMap((v) => v.b!.points.filter((p) => p.agents === ONE_M).map((p) => p.threads)))].sort((a, b) => a - b)
  const spRows = threads.map((t) => ({ t, ideal: t, ...Object.fromEntries(vs.map((v) => [v.key, pt(v.b, ONE_M, t)?.speedup])) }))
  const effRows = threads.map((t) => ({ t, ...Object.fromEntries(vs.map((v) => [v.key, pt(v.b, ONE_M, t)?.efficiency])) }))
  const cur = vs.find((v) => v.key === sel) ?? vs[0]
  const am = cur?.b?.amdahl.find((a) => a.agents === ONE_M)
  const fit = am ? Array.from({ length: Math.max(...threads, 8) }, (_, i) => i + 1).map((n) => ({ t: n, fit: 1 / (am.serial_fraction + (1 - am.serial_fraction) / n), measured: pt(cur!.b, ONE_M, n)?.speedup, ideal: n })) : []
  const bestAt = (n: number) => series.map((s) => ({ l: s.l, v: pt(s.b, n)?.agents_per_sec ?? 0 })).sort((a, b) => b.v - a.v)[0]
  const top = bestAt(ONE_M)
  const mx = vs.map((v) => ({ l: v.label, s: Math.max(...v.b!.points.filter((p) => p.agents === ONE_M).map((p) => p.speedup)) })).sort((a, b) => b.s - a.s)[0]
  return (
    <>
      <Explain>Speedup is time on 1 thread divided by time on N threads; efficiency is speedup divided by N. Amdahl's law fits the measured speedups with a serial fraction s: speedup(N) = 1 / (s + (1 - s) / N).</Explain>
      <Explain>Throughput is agents (and agent-location visits, "contacts") advanced per second at each population size, for the sequential reference and the GPU backends.</Explain>
      <Need what="Performance benchmarks" files={[f.seq]}>
        {top && mx && <Finding>At 1M agents the highest measured throughput is {top.l} at {big(top.v)} agents/s. Best CPU thread speedup is {mx.l}: {x2(mx.s)} over its own 1 thread.{am ? ` ${cur!.label}: serial fraction ${f3(am.serial_fraction)}, Amdahl ceiling ${x2(am.max_speedup)}.` : ''}</Finding>}
        <div className="lab-grid2">
          {(['agents_per_sec', 'contacts_per_sec'] as const).map((key) => (
            <Panel key={key} title={`${key === 'agents_per_sec' ? 'Agents' : 'Contacts'} per second vs population`} settings={series.map((s) => `${s.l}: ${s.b!.machine.cpu}, ${s.b!.days} days (${s.b!.points[0]?.steps} steps), ${s.b!.reps} reps, median`).join(' | ') + ' · 1 thread for CPU rows'}>
              <Chart><LineChart data={rowsOf(key)} margin={{ left: 0, right: 12, top: 6 }}>{grid}<Xax dataKey="n" type="number" scale="log" domain={['dataMin', 'dataMax']} ticks={sizes} tickFormatter={big} /><Yax tickFormatter={big} />{tip}{legend}
                {series.map((s, i) => <Line key={s.k} dataKey={s.k} name={s.l} stroke={PAL[i]} strokeWidth={2} connectNulls isAnimationActive={false} />)}</LineChart></Chart>
            </Panel>
          ))}
        </div>
        <div className="lab-grid2">
          <Panel title="Speedup vs threads (1M agents)" settings={vs[0] && settingsOf(vs[0].b!, 'M1 variants; Xeon rayon only has 1 and 4 threads', '1M agents')}>
            <Chart><ComposedChart data={spRows}>{grid}<Xax dataKey="t" label={{ value: 'threads', position: 'insideBottom', offset: -2, fontSize: 11 }} /><Yax />{tip}{legend}
              <Line dataKey="ideal" name="ideal" stroke={C.D} strokeDasharray="4 4" dot={false} isAnimationActive={false} />
              {vs.map((v) => <Line key={v.key} dataKey={v.key} name={v.label} stroke={v.color} strokeWidth={2} connectNulls isAnimationActive={false} />)}</ComposedChart></Chart>
          </Panel>
          <Panel title="Parallel efficiency vs threads (1M agents)" settings="same runs as the speedup chart">
            <Chart><LineChart data={effRows}>{grid}<Xax dataKey="t" /><Yax domain={[0, 1.05]} />{tip}{legend}
              {vs.map((v) => <Line key={v.key} dataKey={v.key} name={v.label} stroke={v.color} strokeWidth={2} connectNulls isAnimationActive={false} />)}</LineChart></Chart>
          </Panel>
        </div>
        {cur && (
          <Panel title="Amdahl fit" settings={settingsOf(cur.b!, cur.hw, '1M agents')}>
            <Select label="Variant " value={cur.key} onChange={setSel} options={vs.map((v) => ({ v: v.key, l: v.label }))} />
            {am ? (
              <>
                <Chart h={220}><ComposedChart data={fit}>{grid}<Xax dataKey="t" /><Yax />{tip}{legend}
                  <Line dataKey="ideal" name="ideal" stroke={C.D} strokeDasharray="4 4" dot={false} isAnimationActive={false} />
                  <Line dataKey="fit" name={`Amdahl fit (s = ${f3(am.serial_fraction)})`} stroke={C.E} strokeWidth={2} dot={false} isAnimationActive={false} />
                  <Line dataKey="measured" name="measured" stroke={C.S} strokeWidth={2} connectNulls isAnimationActive={false} /></ComposedChart></Chart>
                <Table head={['variant', 'serial fraction s', 'max speedup (1/s)']} rows={vs.map((v) => { const a = v.b!.amdahl.find((x) => x.agents === ONE_M); return [v.label, f3(a?.serial_fraction), x2(a?.max_speedup)] })} />
              </>
            ) : <NotRun what="Amdahl fit for this variant (needs a thread sweep)" />}
          </Panel>
        )}
      </Need>
    </>
  )
}

// ------------------------------------------------------------------ ILP
type Fn = { name: string; lines: string[] }
function parseAsm(t: string): Fn[] {
  const out: Fn[] = []
  let cur: Fn | null = null
  for (const ln of t.split('\n')) {
    const m = /^(?:[\w$.]+::)*([A-Za-z_][\w$.]*):\s*$/.exec(ln)
    if (m && ln.includes('::')) { cur = { name: m[1], lines: [] }; out.push(cur); continue }
    if (cur && !/^\s*(\.cfi|\.p2align|\.globl|\.loh|Lfunc_|Ltmp)/.test(ln)) cur.lines.push(ln)
  }
  return out
}
const KINDS: { id: string; re: RegExp; label: string; color: string }[] = [
  { id: 'mul', re: /\b(umull|umaddl|umulh|smull)\b/, label: 'unrolled 64-bit multiplies (Philox rounds inline)', color: '#fef3c7' },
  { id: 'sel', re: /\b(csel|cset|fcsel|csinc|csinv)\b/, label: 'selects (branches turned into data)', color: '#dcfce7' },
  { id: 'div', re: /\budiv\b/, label: 'integer divide', color: '#e0e7ff' },
  { id: 'call', re: /\bbl\s+/, label: 'calls (naive: location_of / philox per agent)', color: '#fee2e2' },
  { id: 'br', re: /\b(b\.\w+|cbz|cbnz|tbz|tbnz)\b/, label: 'conditional branches', color: '#f3f4f6' },
]
const kindOf = (l: string) => KINDS.find((k) => k.re.test(l))
const count = (lines: string[]) => Object.fromEntries(KINDS.map((k) => [k.id, lines.filter((l) => k.re.test(l)).length]))
const isInsn = (l: string) => /^\s+[a-z]/.test(l) && !/^\s+\./.test(l)

function AsmPane({ fn }: { fn: Fn | undefined }) {
  if (!fn) return <div className="lab-asm note">function not found</div>
  return (
    <div className="lab-asm"><div className="lab-asm-in">{fn.lines.map((l, i) => { const k = kindOf(l); return <div key={i} style={k ? { background: k.color } : undefined}>{l || ' '}</div> })}</div></div>
  )
}
export function IlpTab() {
  const f = useJsons<Bench>({ n: macBench('ilp_naive'), o: macBench('ilp_optimised') })
  const naive = useText('ilp/naive.asm'), opt = useText('ilp/optimised.asm'), an = useText('ilp/analysis.md')
  const nb = okOf(f.n), ob = okOf(f.o)
  const nf = useMemo(() => (naive.s === 'ok' ? parseAsm(naive.d) : []), [naive])
  const of = useMemo(() => (opt.s === 'ok' ? parseAsm(opt.d) : []), [opt])
  const [pair, setPair] = useState<'place' | 'update'>('place')
  const [flush, setFlush] = useState(false)
  const nfn = nf.find((x) => x.name === `${pair}_naive`)
  const ofn = of.find((x) => x.name === (flush ? 'flush_batch' : `${pair}_optimised`))
  const sizes = [100_000, ONE_M]
  const rows = sizes.map((n) => ({ n: big(n), naive: pt(nb, n)?.wall_s.median, optimised: pt(ob, n)?.wall_s.median }))
  const cN = count((nfn?.lines ?? []).filter(isInsn)), cO = count((ofn?.lines ?? []).filter(isInsn))
  return (
    <>
      <Explain>Instruction-level parallelism: the "optimised" kernels unroll four agents at a time, hoist per-step constants out of the per-agent loop and replace branches with selects, so the CPU can overlap independent work.</Explain>
      <Explain>Below: the same simulation with the naive vs optimised kernels (one thread), then the generated aarch64 assembly side by side. Highlighted lines are the categories in the legend; counts are static, not executed.</Explain>
      <Need what="ILP benchmark (ilp_naive / ilp_optimised)" files={[f.n, f.o]}>
        <Finding>Optimised kernels run {sizes.map((n) => `${x2((pt(nb, n)?.wall_s.median ?? NaN) / (pt(ob, n)?.wall_s.median ?? NaN))} faster at ${big(n)}`).join(' and ')} ({sec(pt(nb, ONE_M)?.wall_s.median)} to {sec(pt(ob, ONE_M)?.wall_s.median)} at 1M). The assembly view shows why the code is much larger: {nfn?.name} has {nfn ? nfn.lines.filter(isInsn).length : 'n/a'} instructions, {ofn?.name} has {ofn ? ofn.lines.filter(isInsn).length : 'n/a'}.</Finding>
        <Panel title="Naive vs optimised wall time" settings={settingsOf(nb, M1, '1 thread, sequential schedule')}>
          <Chart h={220}><BarChart data={rows}>{grid}<Xax dataKey="n" /><Yax unit=" s" />{tip}{legend}<Bar dataKey="naive" fill={C.I} isAnimationActive={false} /><Bar dataKey="optimised" fill={C.R} isAnimationActive={false} /></BarChart></Chart>
        </Panel>
      </Need>
      <Panel title="Assembly: naive vs optimised" settings="Apple M1 aarch64 --release, cargo-show-asm: results/ilp/{naive,optimised}.asm">
        {naive.s !== 'ok' || opt.s !== 'ok' ? <NotRun what="Assembly files" f={naive.s !== 'ok' ? naive : opt} /> : (
          <>
            <div className="lab-controls">
              <Select label="Kernel " value={pair} onChange={setPair} options={[{ v: 'place', l: 'place (agent placement)' }, { v: 'update', l: 'update (state transitions)' }]} />
              <label className="lab-select"><input type="checkbox" checked={flush} onChange={(e) => setFlush(e.target.checked)} /> show flush_batch (Philox x4) instead of optimised {pair}</label>
            </div>
            <div className="lab-legend">{KINDS.map((k) => <span key={k.id}><i style={{ background: k.color }} />{k.label}: naive {cN[k.id]} / optimised {cO[k.id]}</span>)}</div>
            <div className="lab-sbs"><div><b>{nfn?.name ?? 'naive'}</b><AsmPane fn={nfn} /></div><div><b>{ofn?.name ?? 'optimised'}</b><AsmPane fn={ofn} /></div></div>
          </>
        )}
      </Panel>
      {an.s === 'ok' && <Collapsible title="written analysis (results/ilp/analysis.md)"><Pre text={an.d} max={400} /></Collapsible>}
    </>
  )
}

// ------------------------------------------------------------------ TLP
interface Imb { machine: { cpu: string }; city: string; agents: number; days: number; steps: number; reps: number; points: { backend: string; schedule: string; threads: number; wall_s: number; total_time_imbalance: number; phases: { phase: string; per_thread: { thread: number; work_s: number }[] }[] }[] }
interface Method { method: string; correct_in_all_runs: boolean; runs: number; wrong_runs: number; max_lost_increments: number; time_ms: { median: number } }
interface Race { agents: number; threads: number; chunk: number; schedule: string; runs: number; scenarios: { name: string; description: string; infectious_agents: number; backends: { backend: string; methods: Method[]; fastest_correct: string }[] }[] }
const SCHEDS = ['static', 'dynamic', 'guided']
export function TlpTab() {
  const f = useJsons<Bench>({ ...Object.fromEntries(VARIANTS.slice(0, 6).map((v) => [v.key, v.file])) })
  const im = useJsons<Imb>({ mum: 'tlp/imbalance_mumbai.json', che: 'tlp/imbalance_chennai.json' })
  const race = useJson<Race>('tlp/race_demo.json')
  const [cfg, setCfg] = useState('rayon|guided')
  const bs = VARIANTS.slice(0, 6).map((v, i) => ({ ...v, b: okOf(f[v.key]), color: PAL[i] })).filter((v) => v.b)
  const threads = [1, 2, 4, 8]
  const time = threads.map((t) => ({ t, ...Object.fromEntries(bs.map((v) => [v.key, pt(v.b, ONE_M, t)?.wall_s.median])) }))
  const sp = threads.map((t) => ({ t, ...Object.fromEntries(bs.map((v) => [v.key, pt(v.b, ONE_M, t)?.speedup])) }))
  const best8 = bs.map((v) => ({ l: v.label, s: pt(v.b, ONE_M, 8)?.speedup ?? 0 })).sort((a, b) => b.s - a.s)[0]
  const mum = okOf(im.mum), che = okOf(im.che)
  const [be, sc] = cfg.split('|')
  const imbRows = SCHEDS.flatMap((s) => ['rayon', 'openmp'].map((b) => ({ k: `${b} ${s}`, Mumbai: mum?.points.find((p) => p.backend === b && p.schedule === s && p.threads === 8)?.total_time_imbalance, Chennai: che?.points.find((p) => p.backend === b && p.schedule === s && p.threads === 8)?.total_time_imbalance })))
  const perThread = (d: Imb | null) => {
    const p = d?.points.find((x) => x.backend === be && x.schedule === sc && x.threads === 8)
    return p ? Array.from({ length: 8 }, (_, i) => ({ thread: `T${i}`, ...Object.fromEntries(p.phases.map((ph) => [ph.phase, ph.per_thread.find((t) => t.thread === i)?.work_s])) })) : []
  }
  const phases = mum?.points[0]?.phases.map((p) => p.phase) ?? []
  const maxImb = Math.max(...imbRows.flatMap((r) => [r.Mumbai ?? 0, r.Chennai ?? 0]))
  const rd = okOf(race)
  return (
    <>
      <Explain>Thread-level parallelism splits agents over cores. The schedule decides how: static gives each thread one fixed block, dynamic hands out small chunks on demand, guided hands out shrinking chunks.</Explain>
      <Explain>Load imbalance is the busiest thread's work time divided by the mean (1.00 = perfect). The race demo counts per-location infectious agents with and without synchronisation to show what a data race does to the answer.</Explain>
      <Need what="Thread-scaling benchmarks" files={[f.rayon_guided_c64]}>
        {best8 && <Finding>At 1M agents and 8 threads the best schedule is {best8.l} with {x2(best8.s)} over its own 1-thread run on {M1} (4 performance + 4 efficiency cores).</Finding>}
        <div className="lab-grid2">
          <Panel title="Wall time vs threads (1M agents)" settings={settingsOf(bs[0]?.b ?? null, M1, '1M agents, chunk: static 0 / dynamic 1024 / guided 64')}>
            <Chart><LineChart data={time}>{grid}<Xax dataKey="t" /><Yax unit=" s" />{tip}{legend}{bs.map((v) => <Line key={v.key} dataKey={v.key} name={v.label} stroke={v.color} strokeWidth={2} strokeDasharray={v.key.startsWith('openmp') ? '5 3' : undefined} isAnimationActive={false} />)}</LineChart></Chart>
          </Panel>
          <Panel title="Speedup vs own 1 thread" settings="same runs (OpenMP dashed); each backend's 1-thread time differs">
            <Chart><LineChart data={sp}>{grid}<Xax dataKey="t" /><Yax />{tip}{legend}{bs.map((v) => <Line key={v.key} dataKey={v.key} name={v.label} stroke={v.color} strokeWidth={2} strokeDasharray={v.key.startsWith('openmp') ? '5 3' : undefined} isAnimationActive={false} />)}</LineChart></Chart>
          </Panel>
        </div>
      </Need>
      <Need what="Load-imbalance runs (Mumbai and Chennai)" files={[im.mum, im.che]}>
        {mum && che && <Finding>Measured imbalance is small: the maximum is {f3(maxImb)} (max/mean per-thread work). Mumbai and Chennai show {Math.abs(imbRows.reduce((a, r) => a + (r.Mumbai ?? 0) - (r.Chennai ?? 0), 0) / imbRows.length) < 0.02 ? 'no meaningful difference' : 'a small difference'} on average (mean gap {f3(Math.abs(imbRows.reduce((a, r) => a + (r.Mumbai ?? 0) - (r.Chennai ?? 0), 0) / imbRows.length))}).</Finding>}
        <div className="lab-grid2">
          <Panel title="Imbalance at 8 threads: Mumbai vs Chennai" settings={mum && `${mum.machine.cpu} · ${big(mum.agents)} agents · ${mum.days} days (${mum.steps} steps) · ${mum.reps} reps (median wall) · both cities`}>
            <Chart><BarChart data={imbRows}>{grid}<Xax dataKey="k" interval={0} angle={-25} textAnchor="end" height={60} /><Yax domain={[1, 'auto']} />{tip}{legend}<Bar dataKey="Mumbai" fill={C.S} isAnimationActive={false} /><Bar dataKey="Chennai" fill={C.E} isAnimationActive={false} /></BarChart></Chart>
          </Panel>
          <Panel title="Per-thread work time (8 threads)" settings={mum && `${mum.machine.cpu} · ${big(mum.agents)} agents · ${mum.days} days · stacked by parallel phase`}>
            <Select label="Backend / schedule " value={cfg} onChange={setCfg} options={['rayon', 'openmp'].flatMap((b) => SCHEDS.map((s) => ({ v: `${b}|${s}`, l: `${b} ${s}` })))} />
            {[['Mumbai', mum], ['Chennai', che]].map(([n, d]) => (
              <div key={n as string}><b className="note">{n as string}</b>
                <Chart h={150}><BarChart data={perThread(d as Imb | null)}>{grid}<Xax dataKey="thread" /><Yax unit=" s" />{tip}{n === 'Mumbai' && legend}{phases.map((p, i) => <Bar key={p} dataKey={p} stackId="a" fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart></div>
            ))}
          </Panel>
        </div>
      </Need>
      <Need what="Race demo (race_demo.json)" files={[race]}>
        {rd && (
          <>
            {(() => { const all = rd.scenarios.flatMap((s) => s.backends.map((b) => ({ s: s.name, b: b.backend, u: b.methods.find((m) => m.method === 'unsynchronised')! }))); return <Finding>Unsynchronised counting gave a wrong answer in {all.map((a) => `${a.u.wrong_runs}/${a.u.runs} runs (${a.b}, ${a.s})`).join(', ')}. Atomics and per-thread buffers were correct in every run.</Finding> })()}
            {rd.scenarios.map((s) => (
              <Panel key={s.name} title={`Race demo: ${s.name}`} settings={`${M1} · ${big(rd.agents)} agents · ${rd.threads} threads · ${rd.schedule} chunk ${rd.chunk} · ${rd.runs} runs per method · ${big(s.infectious_agents)} infectious agents (${s.description})`}>
                <Table head={['backend', 'method', 'wrong runs', 'max lost increments', 'median ms', 'correct?']} rows={s.backends.flatMap((b) => b.methods.map((m) => [b.backend, m.method, `${m.wrong_runs} / ${m.runs}`, m.max_lost_increments, f3(m.time_ms.median), <b key="c" style={{ color: m.correct_in_all_runs ? C.R : C.I }}>{m.correct_in_all_runs ? 'correct' : 'WRONG'}{b.fastest_correct === m.method ? ' · fastest correct' : ''}</b>]))} />
              </Panel>
            ))}
          </>
        )}
      </Need>
    </>
  )
}

// ------------------------------------------------------------------ SIMD
interface Align { machine: Machine; elements: number; passes_per_sample: number; reps: number; note: string; rows: { kernel: string; variant: string; alignment: string; ns_per_element: number }[] }
export function SimdTab() {
  const f = useJsons<Bench>({ ms: macBench('simd_scalar'), mn: macBench('simd_neon'), ks: kgBench('simd_scalar'), ka: kgBench('simd_avx2') })
  const al = useJson<Align>('bench/simd_alignment_apple_m1.json')
  const sizes = [100_000, ONE_M]
  const pair = (a: Bench | null, b: Bench | null, la: string, lb: string, name: string) => ({ name, a, b, la, lb, rows: sizes.map((n) => ({ n: big(n), [la]: pt(a, n)?.wall_s.median, [lb]: pt(b, n)?.wall_s.median })), sp: sizes.map((n) => (pt(a, n) && pt(b, n) ? pt(a, n)!.wall_s.median / pt(b, n)!.wall_s.median : null)) })
  const ps = [pair(okOf(f.ms), okOf(f.mn), 'scalar', 'NEON', 'Apple M1: scalar vs NEON'), pair(okOf(f.ks), okOf(f.ka), 'scalar', 'AVX2', 'Kaggle Xeon: scalar vs AVX2')]
  const a = okOf(al)
  const kernels = a ? [...new Set(a.rows.map((r) => r.kernel))] : []
  const alRows = kernels.map((k) => ({ k, ...Object.fromEntries(a!.rows.filter((r) => r.kernel === k).map((r) => [`${r.variant} ${r.alignment.startsWith('aligned') ? 'aligned' : 'misaligned'}`, r.ns_per_element])) }))
  const alKeys = a ? [...new Set(a.rows.map((r) => `${r.variant} ${r.alignment.startsWith('aligned') ? 'aligned' : 'misaligned'}`))] : []
  const effect = a ? Math.max(...kernels.flatMap((k) => ['scalar', 'neon'].map((v) => { const x = a.rows.find((r) => r.kernel === k && r.variant === v && r.alignment.startsWith('aligned'))?.ns_per_element, y = a.rows.find((r) => r.kernel === k && r.variant === v && r.alignment.startsWith('mis'))?.ns_per_element; return x && y ? Math.abs(y / x - 1) : 0 }))) : 0
  return (
    <>
      <Explain>SIMD runs one instruction on several values at once: NEON (128-bit) on the M1, AVX2 (256-bit) on the Kaggle Xeon. Here it is used for the timer and RNG kernels inside the full simulation.</Explain>
      <Explain>The end-to-end gain is bounded by how much of a step those kernels are; the alignment test times the kernels alone with 64-byte-aligned vs offset arrays.</Explain>
      {ps.map((p) => (
        <Panel key={p.name} title={p.name} settings={p.a && p.b ? settingsOf(p.b, p.name.startsWith('Apple') ? M1 : XEON, '1 thread') : undefined}>
          {!p.a || !p.b ? <NotRun what={p.name} /> : (
            <>
              <Chart h={200}><BarChart data={p.rows}>{grid}<Xax dataKey="n" /><Yax unit=" s" />{tip}{legend}<Bar dataKey={p.la} fill={C.D} isAnimationActive={false} /><Bar dataKey={p.lb} fill={C.S} isAnimationActive={false} /></BarChart></Chart>
              <p className="note">Speedup of {p.lb} over scalar: {sizes.map((n, i) => `${x2(p.sp[i])} at ${big(n)}`).join(', ')}.</p>
            </>
          )}
        </Panel>
      ))}
      <Finding>{ps.map((p) => p.a && p.b ? `${p.lb}: ${x2(p.sp[1])} end-to-end at 1M` : `${p.lb}: not run`).join('; ')}. AVX2 was measured only on the Kaggle Xeon CPU, NEON only on the M1; they are different machines and not comparable with each other.{a ? ` Alignment changed kernel time by at most ${(effect * 100).toFixed(0)}%.` : ''}</Finding>
      <Need what="Alignment test (simd_alignment_apple_m1.json)" files={[al]}>
        {a && <Panel title="Alignment effect on the kernels alone (ns per element, lower is better)" settings={`${a.machine.cpu} · ${a.elements.toLocaleString()} elements · ${a.passes_per_sample} passes per sample · ${a.reps} reps · 1 thread`}>
          <Chart h={220}><BarChart data={alRows}>{grid}<Xax dataKey="k" /><Yax />{tip}{legend}{alKeys.map((k, i) => <Bar key={k} dataKey={k} fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart>
          <p className="note">{a.note}</p></Panel>}
      </Need>
    </>
  )
}

// ------------------------------------------------------------------ MPI
interface MpiPt { mode: string; procs: number; wall_s: { median: number }; speedup: number; efficiency: number; utilisation: number; comm_fraction: number; comm_s_total: number; idle_s_total: number; bytes_sent_total: number; compute_imbalance: number }
interface Mpi { machine: string; backend: string; agents: number; days: number; runs_per_scenario: number; scenarios: number; tasks: number; reps: number; note: string; points: MpiPt[] }
export function MpiTab() {
  const m = useJson<Mpi>('mpi/scaling.json')
  const bs = useJsons<Bench>({ s: macBench('mpi'), d: macBench('mpi_dynamic') })
  const d = okOf(m)
  const procs = d ? [...new Set(d.points.map((p) => p.procs))].sort((a, b) => a - b) : []
  const get = (mode: string, n: number) => d?.points.find((p) => p.mode === mode && p.procs === n)
  const sp = procs.map((n) => ({ n, ideal: n, static: get('static', n)?.speedup, dynamic: get('dynamic', n)?.speedup }))
  const comm = procs.map((n) => ({ n, static: get('static', n) ? get('static', n)!.comm_fraction * 100 : undefined, dynamic: get('dynamic', n) ? get('dynamic', n)!.comm_fraction * 100 : undefined }))
  const top = procs.length ? Math.max(...procs) : 0
  return (
    <>
      <Explain>MPI splits the Monte Carlo workload (many independent scenario runs) across processes. Static gives each rank a fixed share of runs; dynamic has rank 0 hand out runs on request (rank 0 does no simulation).</Explain>
      <Explain>Communication overhead is the share of wall time spent sending/receiving rather than simulating; speedup is relative to the 1-process run.</Explain>
      <Need what="MPI scaling (mpi/scaling.json)" files={[m]}>
        {d && (
          <>
            <Finding>At {top} processes static reaches {x2(get('static', top)?.speedup)} and dynamic {x2(get('dynamic', top)?.speedup)}. Communication was {f3((get('static', top)?.comm_fraction ?? 0) * 100)}% of run time for static and {f3((get('dynamic', top)?.comm_fraction ?? 0) * 100)}% for dynamic at {top} processes{okOf(bs.s)?.amdahl[0] ? `; Amdahl serial fraction (static) ${f3(okOf(bs.s)!.amdahl[0].serial_fraction)}` : ''}.</Finding>
            <div className="lab-grid2">
              <Panel title="Speedup vs processes" settings={`${d.machine} · ${d.backend} backend · ${big(d.agents)} agents · ${d.days} days · ${d.scenarios} scenarios x ${d.runs_per_scenario} runs · ${d.reps} reps (median)`}>
                <Chart><LineChart data={sp}>{grid}<Xax dataKey="n" /><Yax />{tip}{legend}<Line dataKey="ideal" stroke={C.D} strokeDasharray="4 4" dot={false} isAnimationActive={false} /><Line dataKey="static" stroke={C.S} strokeWidth={2} isAnimationActive={false} /><Line dataKey="dynamic" stroke={C.E} strokeWidth={2} isAnimationActive={false} /></LineChart></Chart>
              </Panel>
              <Panel title="Communication share of run time (%)" settings="same runs; dynamic includes the master waiting for requests">
                <Chart><BarChart data={comm}>{grid}<Xax dataKey="n" /><Yax unit="%" />{tip}{legend}<Bar dataKey="static" fill={C.S} isAnimationActive={false} /><Bar dataKey="dynamic" fill={C.E} isAnimationActive={false} /></BarChart></Chart>
              </Panel>
            </div>
            <Panel title="Details" settings={d.note}>
              <Table head={['mode', 'procs', 'median s', 'speedup', 'efficiency', 'utilisation', 'comm %', 'idle s (total)', 'bytes sent', 'compute imbalance']}
                rows={d.points.map((p) => [p.mode, p.procs, f3(p.wall_s.median), x2(p.speedup), f3(p.efficiency), f3(p.utilisation), f3(p.comm_fraction * 100), f3(p.idle_s_total), p.bytes_sent_total.toLocaleString(), f3(p.compute_imbalance)])} />
            </Panel>
          </>
        )}
      </Need>
    </>
  )
}
export type { BPoint }
