import { useMemo, useState } from 'react'
import { Bar, BarChart, LineChart, Line } from 'recharts'
import { kgBench, M1, T4, XEON, okOf, pt, settingsOf, useJson, useJsons, useText, type Bench } from './data'
import { big, Chart, Explain, f3, Finding, grid, legend, Need, NotRun, PAL, Panel, Pre, sec, Select, Table, tip, x2, Xax, Yax } from './ui'

interface MRow { agents: number; storage: string; threadgroup: number; steps: number; total_s: number; kernel_s: number; cpu_lambda_s: number; readback_s: number; dispatch_overhead_s: number; matches_sequential: boolean }
interface MBreak { device: string; warmup: number; reps: number; days: number; rows: MRow[] }
interface CRow { agents: number; variant: string; layout: string; count: string; update: string; partition_every: number; prevalence: number; block: number; steps: number; total_s: number; kernel_s: number; kernel_update_s: number | null; cpu_lambda_s: number; h2d_s: number; readback_s: number; dispatch_overhead_s: number; matches_sequential: boolean | null }
interface CBreak { device: string; warmup: number; reps: number; days: number; rows: CRow[] }
interface MvC { note: string; rows: { agents: number; metal_s: number; best_cpu: string; best_cpu_s: number; cpu_s: Record<string, number>; speedup_vs_best_cpu: number; steps: number }[] }
const ONE_M = 1_000_000

// ------------------------------------------------------------------ GPU
export function GpuTab() {
  const mb = useJson<MBreak>('gpu/metal_breakdown.json'), mv = useJson<MvC>('gpu/metal_vs_cpu.json')
  const cb = useJson<CBreak>('kaggle/gpu/cuda_breakdown.json'), cv = useJson<CBreak>('kaggle/gpu/cuda_variants.json')
  const kb = useJsons<Bench>({ seq: kgBench('sequential'), ray: kgBench('rayon_guided_c64'), cuda: kgBench('cuda') })
  const [mS, setMS] = useState<'shared' | 'private'>('shared')
  const m = okOf(mb), v = okOf(mv), c = okOf(cb)
  const sizes = [100_000, ONE_M, 10_000_000]
  const cmp = sizes.map((n) => {
    const r = v?.rows.find((x) => x.agents === n)
    return { n: big(n), 'M1 CPU (best, 8t)': r?.best_cpu_s, 'Metal (M1)': r?.metal_s, 'Xeon CPU (rayon guided 4t)': pt(okOf(kb.ray), n, 4)?.wall_s.median, 'CUDA (T4)': pt(okOf(kb.cuda), n)?.wall_s.median }
  })
  const keys = ['M1 CPU (best, 8t)', 'Metal (M1)', 'Xeon CPU (rayon guided 4t)', 'CUDA (T4)']
  const metalStack = (m?.rows ?? []).filter((r) => r.storage === 'shared' && r.threadgroup === 64).map((r) => ({ n: `Metal ${big(r.agents)}`, kernel: r.kernel_s, 'dispatch overhead': r.dispatch_overhead_s, 'CPU lambda': r.cpu_lambda_s, readback: r.readback_s }))
  const cudaStack = (c?.rows ?? []).filter((r) => r.block === 256).map((r) => ({ n: `CUDA ${big(r.agents)}`, kernel: r.kernel_s, 'dispatch overhead': r.dispatch_overhead_s, 'CPU lambda': r.cpu_lambda_s, 'host-to-device': r.h2d_s, 'readback (device-to-host)': r.readback_s }))
  const stack = [...metalStack, ...cudaStack]
  const skeys = ['kernel', 'dispatch overhead', 'CPU lambda', 'host-to-device', 'readback', 'readback (device-to-host)']
  const share = (row: Record<string, number | string>) => { const t = skeys.reduce((a, k) => a + (typeof row[k] === 'number' ? (row[k] as number) : 0), 0); return t }
  const cuda1 = cudaStack.find((r) => r.n === 'CUDA 1M')
  const metal1 = metalStack.find((r) => r.n === 'Metal 1M')
  const blockRows = [100_000, ONE_M].map((n) => ({ n: big(n), ...Object.fromEntries((c?.rows ?? []).filter((r) => r.agents === n).map((r) => [`block ${r.block}`, r.total_s])) }))
  const blocks = [...new Set((c?.rows ?? []).map((r) => r.block))].sort((a, b) => a - b)
  const tgRows = [32, 64, 128, 256].map((tg) => ({ tg: String(tg), ...Object.fromEntries((m?.rows ?? []).filter((r) => r.threadgroup === tg).map((r) => [`${big(r.agents)} ${r.storage}`, r.total_s])) }))
  const tgKeys = [...new Set((m?.rows ?? []).map((r) => `${big(r.agents)} ${r.storage}`))]
  const cvr = okOf(cv)
  return (
    <>
      <Explain>A GPU runs thousands of agents at once, but each step also pays for launching kernels, copying data and reading results back. Metal (Apple M1, unified memory) and CUDA (Tesla T4, PCIe) have very different transfer costs.</Explain>
      <Explain>The first chart compares end-to-end time for 80 steps; the stacked bars split where one run's time went; the sweeps vary threadgroup (Metal) and block (CUDA) size.</Explain>
      <Need what="Metal vs CPU / Kaggle benches" files={[mv, kb.cuda]}>
        <Finding>At 1M agents: Metal {sec(v?.rows.find((r) => r.agents === ONE_M)?.metal_s)} vs best M1 CPU {sec(v?.rows.find((r) => r.agents === ONE_M)?.best_cpu_s)} ({x2(v?.rows.find((r) => r.agents === ONE_M)?.speedup_vs_best_cpu)}); CUDA on the T4 {sec(pt(okOf(kb.cuda), ONE_M)?.wall_s.median)} vs Xeon rayon guided 4t {sec(pt(okOf(kb.ray), ONE_M, 4)?.wall_s.median)} ({x2((pt(okOf(kb.ray), ONE_M, 4)?.wall_s.median ?? NaN) / (pt(okOf(kb.cuda), ONE_M)?.wall_s.median ?? NaN))}). At 100k Metal vs best M1 CPU is {x2(v?.rows.find((r) => r.agents === 100_000)?.speedup_vs_best_cpu)} ({'<'}1 means slower).</Finding>
        <Panel title="Wall time: CPU vs Metal vs CUDA" settings={`${M1}: ${v?.note ?? ''} | ${XEON} and ${T4}: ${settingsOf(okOf(kb.cuda), 'Kaggle', 'CUDA default variant soa_atomic_branch')}. 10M exists on Mac only (Metal 10M not checked against sequential).`}>
          <Chart><BarChart data={cmp}>{grid}<Xax dataKey="n" /><Yax unit=" s" scale="sqrt" />{tip}{legend}{keys.map((k, i) => <Bar key={k} dataKey={k} fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart>
        </Panel>
      </Need>
      <Need what="GPU time breakdowns" files={[mb, cb]}>
        <Finding>At 1M agents the kernels are {f3(((metal1?.kernel ?? 0) / share(metal1 ?? {})) * 100)}% of the attributed Metal time and {f3(((cuda1?.kernel ?? 0) / share(cuda1 ?? {})) * 100)}% of CUDA time; CUDA readback (device-to-host) is {f3(((cuda1?.['readback (device-to-host)'] ?? 0) / share(cuda1 ?? {})) * 100)}% and Metal dispatch overhead is {f3(((metal1?.['dispatch overhead'] ?? 0) / share(metal1 ?? {})) * 100)}%.</Finding>
        <Panel title="Where the time goes (seconds per 80-step run)" settings={`Metal: ${m?.device}, shared storage, threadgroup 64, ${m?.days} days, ${m?.reps} reps · CUDA: ${c?.device}, soa_atomic_branch block 256, ${c?.days} days, ${c?.reps} reps · one-off buffer setup excluded`}>
          <Chart h={280}><BarChart data={stack}>{grid}<Xax dataKey="n" /><Yax unit=" s" />{tip}{legend}{skeys.map((k, i) => <Bar key={k} dataKey={k} stackId="a" fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart>
        </Panel>
        <div className="lab-grid2">
          <Panel title="CUDA block-size sweep (total s)" settings={`${c?.device} · soa_atomic_branch · ${c?.days} days · ${c?.reps} reps`}>
            <Chart h={220}><BarChart data={blockRows}>{grid}<Xax dataKey="n" /><Yax unit=" s" />{tip}{legend}{blocks.map((b, i) => <Bar key={b} dataKey={`block ${b}`} fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart>
            <p className="note">Best at 1M: {(() => { const r = (c?.rows ?? []).filter((x) => x.agents === ONE_M).sort((a, b) => a.total_s - b.total_s); return r.length ? `block ${r[0].block} (${sec(r[0].total_s)}) vs worst block ${r[r.length - 1].block} (${sec(r[r.length - 1].total_s)})` : 'n/a' })()}.</p>
          </Panel>
          <Panel title="Metal threadgroup sweep (total s)" settings={`${m?.device} · ${m?.days} days · ${m?.reps} reps`}>
            <Select label="Storage filter " value={mS} onChange={setMS} options={[{ v: 'shared', l: 'shared' }, { v: 'private', l: 'private' }]} />
            <Chart h={190}><BarChart data={tgRows}>{grid}<Xax dataKey="tg" /><Yax unit=" s" />{tip}{legend}{tgKeys.filter((k) => k.endsWith(mS)).map((k, i) => <Bar key={k} dataKey={k} fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart>
            <p className="note">1M, shared vs private at threadgroup 64: {(() => { const a = m?.rows.find((r) => r.agents === ONE_M && r.threadgroup === 64 && r.storage === 'shared'), b = m?.rows.find((r) => r.agents === ONE_M && r.threadgroup === 64 && r.storage === 'private'); return a && b ? `${sec(a.total_s)} vs ${sec(b.total_s)}` : 'n/a' })()}.</p>
          </Panel>
        </div>
      </Need>
      {cvr && (
        <Panel title="CUDA variants at 1M (layout / counting / update strategy)" settings={`${cvr.device} · ${cvr.days} days · ${cvr.reps} reps · block 256`}>
          <Table head={['variant', 'total s', 'kernel s', 'matches sequential']} rows={cvr.rows.map((r) => [r.variant, f3(r.total_s), f3(r.kernel_s), r.matches_sequential == null ? 'n/a' : r.matches_sequential ? 'yes' : 'NO'])} />
        </Panel>
      )}
    </>
  )
}

// ------------------------------------------------------------------ Warps
export function WarpTab() {
  const w = useJson<CBreak>('kaggle/gpu/cuda_warp.json')
  const ncu = [useText('kaggle/ncu_branch_part0.csv'), useText('kaggle/ncu_pred_part0.csv'), useText('kaggle/ncu_branch_part4.csv')]
  const d = okOf(w)
  const prevs = d ? [...new Set(d.rows.map((r) => r.prevalence))].sort((a, b) => a - b) : []
  const variants = d ? [...new Set(d.rows.map((r) => r.variant))] : []
  const label = (v: string) => v.replace('soa_atomic_', '').replace('_b256', '').replace('part', 'partition every ')
  const rows = prevs.map((p) => ({ p: `${(p * 100).toFixed(2)}%`, ...Object.fromEntries(variants.map((v) => [label(v), d!.rows.find((r) => r.prevalence === p && r.variant === v)?.kernel_update_s ?? undefined])) }))
  const totals = prevs.map((p) => ({ p: `${(p * 100).toFixed(2)}%`, ...Object.fromEntries(variants.map((v) => [label(v), d!.rows.find((r) => r.prevalence === p && r.variant === v)?.total_s])) }))
  const best = prevs.map((p) => { const r = d!.rows.filter((x) => x.prevalence === p && x.kernel_update_s != null).sort((a, b) => a.kernel_update_s! - b.kernel_update_s!)[0]; return `${(p * 100).toFixed(2)}%: ${label(r.variant)} (${(r.kernel_update_s! * 1000).toFixed(1)} ms)` })
  const profErr = ncu.map((n) => (n.s === 'ok' ? (/==ERROR==[^\n]*/.exec(n.d)?.[0] ?? null) : n.s === 'empty' ? 'empty file' : 'file missing'))
  const profOk = ncu.some((n, i) => n.s === 'ok' && !profErr[i])
  return (
    <>
      <Explain>A warp is 32 GPU threads executing in lock-step. If threads of one warp take different sides of an if, both paths run one after the other (divergence). Predication computes both and selects; partitioning agents by health state keeps similar agents in the same warp.</Explain>
      <Explain>The sweep varies prevalence (the fraction of agents infectious) because divergence should grow with it. Times are the update kernel alone, from CUDA events.</Explain>
      <Need what="Warp-divergence benchmark (cuda_warp.json)" files={[w]}>
        {d && (
          <>
            <Finding>Fastest update kernel per prevalence: {best.join('; ')}. {(() => { const a = d.rows.filter((r) => r.variant === 'soa_atomic_branch_b256'), b = d.rows.filter((r) => r.variant === 'soa_atomic_pred_b256'); const ratio = prevs.map((p) => (b.find((r) => r.prevalence === p)!.kernel_update_s! / a.find((r) => r.prevalence === p)!.kernel_update_s!)); return `Predicated / branchy update time ranges ${Math.min(...ratio).toFixed(2)}x to ${Math.max(...ratio).toFixed(2)}x across prevalences.` })()}</Finding>
            <Panel title="Update-kernel time by prevalence (s)" settings={`${d.device} · ${big(d.rows[0].agents)} agents · ${d.days} days (${d.rows[0].steps} steps) · ${d.reps} reps (+${d.warmup} warm-up) · block ${d.rows[0].block} · partition = sort agents by health state every N steps`}>
              <Chart h={280}><BarChart data={rows}>{grid}<Xax dataKey="p" label={{ value: 'prevalence', position: 'insideBottom', offset: -2, fontSize: 11 }} /><Yax unit=" s" />{tip}{legend}{variants.map((v, i) => <Bar key={v} dataKey={label(v)} fill={PAL[i]} isAnimationActive={false} />)}</BarChart></Chart>
            </Panel>
            <Panel title="Total run time by prevalence (s)" settings="same runs; includes placement, transfers and readback">
              <Chart h={220}><LineChart data={totals}>{grid}<Xax dataKey="p" /><Yax unit=" s" />{tip}{legend}{variants.map((v, i) => <Line key={v} dataKey={label(v)} stroke={PAL[i]} strokeWidth={2} isAnimationActive={false} />)}</LineChart></Chart>
            </Panel>
          </>
        )}
      </Need>
      <Panel title="Branch efficiency (Nsight Compute)" settings={`${T4} · files kaggle/ncu_*.csv`}>
        {profOk ? (
          ncu.map((n, i) => n.s === 'ok' && !profErr[i] && <Pre key={i} text={n.d} max={200} />)
        ) : (
          <div className="card lab-notrun"><b>Profiler unavailable</b><span>Nsight Compute could not read GPU performance counters on this machine, so no branch-efficiency numbers exist. Message: {profErr.find(Boolean) ?? 'n/a'}</span></div>
        )}
      </Panel>
    </>
  )
}

// ------------------------------------------------------------------ PTX
interface PtxFile { sha256: string; bytes: number; kernels: Record<string, Record<string, number>>; totals: Record<string, number> }
const LEVELS = ['O0', 'O1', 'O2', 'O3', 'O3fm', 'G']
const LEVEL_NAME: Record<string, string> = { O0: '-O0', O1: '-O1', O2: '-O2', O3: '-O3', O3fm: '-O3 fast-math', G: '-G (debug)' }
type DRow = { a?: string; b?: string; t: 'same' | 'del' | 'add' | 'gap' }
function diffLines(a: string[], b: string[]): DRow[] | null {
  let s = 0
  while (s < a.length && s < b.length && a[s] === b[s]) s++
  let ea = a.length, eb = b.length
  while (ea > s && eb > s && a[ea - 1] === b[eb - 1]) { ea--; eb-- }
  const A = a.slice(s, ea), B = b.slice(s, eb)
  if (A.length * B.length > 30_000_000) return null
  const w = B.length + 1, dp = new Uint16Array((A.length + 1) * w)
  for (let i = A.length - 1; i >= 0; i--) for (let j = B.length - 1; j >= 0; j--) dp[i * w + j] = A[i] === B[j] ? dp[(i + 1) * w + j + 1] + 1 : Math.max(dp[(i + 1) * w + j], dp[i * w + j + 1])
  const out: DRow[] = a.slice(0, s).map((l) => ({ a: l, b: l, t: 'same' as const }))
  let i = 0, j = 0
  while (i < A.length || j < B.length) {
    if (i < A.length && j < B.length && A[i] === B[j]) { out.push({ a: A[i], b: B[j], t: 'same' }); i++; j++ }
    else if (j >= B.length || (i < A.length && dp[(i + 1) * w + j] >= dp[i * w + j + 1])) out.push({ a: A[i++], t: 'del' })
    else out.push({ b: B[j++], t: 'add' })
  }
  return out.concat(a.slice(ea).map((l) => ({ a: l, b: l, t: 'same' as const })))
}
const clean = (t: string, hide: boolean) => t.split('\n').filter((l) => !hide || !(/^\s*(\/\/|\.loc|\.file)/.test(l) || l.trim() === ''))

export function PtxTab() {
  const sm = useJson<{ arch: string; files: Record<string, PtxFile> }>('kaggle/ptx/summary.json')
  const [file, setFile] = useState('update'), [la, setLa] = useState('O3'), [lb, setLb] = useState('G'), [hide, setHide] = useState(true), [all, setAll] = useState(false)
  const ta = useText(`kaggle/ptx/${file}_${la}.ptx`), tb = useText(`kaggle/ptx/${file}_${lb}.ptx`)
  const s = okOf(sm)
  const nvcc = useText('kaggle/ptx/nvcc_info.txt')
  const ops = useMemo(() => (s ? [...new Set(LEVELS.flatMap((l) => Object.keys(s.files[`${file}_${l}`]?.totals ?? {})))].filter((k) => k !== 'total') : []), [s, file])
  const mix = s ? LEVELS.filter((l) => s.files[`${file}_${l}`]).map((l) => { const t = s.files[`${file}_${l}`].totals; const listed = ops.reduce((a, k) => a + (t[k] ?? 0), 0); return { l: LEVEL_NAME[l], ...Object.fromEntries(ops.map((k) => [k, t[k] ?? 0])), other: Math.max(0, t.total - listed) } } ) : []
  const groups = (f: string) => { const m: Record<string, string[]> = {}; LEVELS.forEach((l) => { const h = s?.files[`${f}_${l}`]?.sha256; if (h) (m[h] ??= []).push(LEVEL_NAME[l]) }); return Object.values(m).map((g) => g.join(' = ')).join('  |  ') }
  const diff = useMemo(() => (ta.s === 'ok' && tb.s === 'ok' ? diffLines(clean(ta.d, hide), clean(tb.d, hide)) : null), [ta, tb, hide])
  const same = s && s.files[`${file}_${la}`]?.sha256 === s.files[`${file}_${lb}`]?.sha256
  const rows = diff ? (all ? diff : diff.slice(0, 2500)) : []
  const changed = diff ? diff.filter((r) => r.t !== 'same').length : 0
  const lvlOpts = LEVELS.map((l) => ({ v: l, l: LEVEL_NAME[l] }))
  return (
    <>
      <Explain>PTX is NVIDIA's intermediate assembly. nvcc was run at each optimisation level on the three CUDA kernel files (placement, stats, update) and the instructions were counted per opcode.</Explain>
      <Explain>If two levels give the same SHA-256, their PTX is byte-identical, so any speed difference between them cannot come from the code generator.</Explain>
      <Need what="PTX summary (kaggle/ptx/summary.json)" files={[sm]}>
        {s && (
          <>
            <Finding>{['place', 'stats', 'update'].map((f) => `${f}: ${groups(f)}`).join(' · ')}. Identical hashes mean identical PTX: the optimisation flags had no effect on these kernels except -G, which {(() => { const t = (f: string, l: string) => s.files[`${f}_${l}`]?.totals.total; return `grows update from ${t('update', 'O3')} to ${t('update', 'G')} instructions` })()}.</Finding>
            <Panel title="Instruction mix per optimisation level" settings={`nvcc ${nvcc.s === 'ok' ? nvcc.d.trim() : ''} · target ${s.arch} · static instruction counts from the .ptx files (not executed counts)`}>
              <Select label="Kernel file " value={file} onChange={setFile} options={['place', 'stats', 'update'].map((f) => ({ v: f, l: f }))} />
              <Chart h={300}><BarChart data={mix}>{grid}<Xax dataKey="l" /><Yax />{tip}{legend}{[...ops, 'other'].map((k, i) => <Bar key={k} dataKey={k} stackId="a" fill={PAL[i % PAL.length]} fillOpacity={k === 'other' ? 0.4 : 1} isAnimationActive={false} />)}</BarChart></Chart>
              <Table head={['level', 'sha256 (16)', 'bytes', 'total instr', 'bra', 'setp', 'selp']} rows={LEVELS.filter((l) => s.files[`${file}_${l}`]).map((l) => { const x = s.files[`${file}_${l}`]; return [LEVEL_NAME[l], x.sha256, x.bytes.toLocaleString(), x.totals.total, x.totals.bra ?? 0, x.totals.setp ?? 0, x.totals.selp ?? 0] })} />
            </Panel>
            <Panel title="PTX diff" settings={`${file}_${la}.ptx vs ${file}_${lb}.ptx · ${s.arch}`}>
              <div className="lab-controls">
                <Select label="Left " value={la} onChange={setLa} options={lvlOpts} /><Select label="Right " value={lb} onChange={setLb} options={lvlOpts} />
                <label className="lab-select"><input type="checkbox" checked={hide} onChange={(e) => setHide(e.target.checked)} /> hide comments / .loc / blank lines</label>
              </div>
              {same && <div className="lab-finding"><b>Identical</b><span>Both files have the same SHA-256 ({s.files[`${file}_${la}`].sha256}): the PTX is byte-for-byte identical.</span></div>}
              {ta.s !== 'ok' || tb.s !== 'ok' ? <NotRun what="PTX files" f={ta.s !== 'ok' ? ta : tb} /> : diff == null ? <p className="note">Files too large to align in the browser; showing sizes only: {ta.d.split('\n').length} vs {tb.d.split('\n').length} lines.</p> : (
                <>
                  <p className="note">{changed} differing lines of {diff.length} aligned rows.{!all && diff.length > rows.length ? ` Showing first ${rows.length}.` : ''} {diff.length > 2500 && <button className="btn" onClick={() => setAll(!all)}>{all ? 'Show fewer' : 'Show all'}</button>}</p>
                  <div className="lab-diff">{rows.map((r, i) => <div key={i} className={`lab-drow ${r.t}`}><span>{r.a ?? ''}</span><span>{r.b ?? ''}</span></div>)}</div>
                </>
              )}
            </Panel>
          </>
        )}
      </Need>
    </>
  )
}

// ------------------------------------------------------------------ OpenACC
export function OpenAccTab() {
  const notes = useText('kaggle/openacc_notes.md'), bl = useText('kaggle/logs/openacc_build.log'), rl = useText('kaggle/logs/openacc_run.log'), steps = useText('kaggle/steps.tsv')
  const code = (n: string) => (steps.s === 'ok' ? steps.d.split('\n').map((l) => l.split('\t')).find((c) => c[0] === n)?.[1] : undefined)
  const bc = code('openacc_build'), rc = code('openacc_run')
  return (
    <>
      <Explain>OpenACC annotates C loops with pragmas so the compiler generates GPU code. It needs a compiler built with NVIDIA offload support (nvc or gcc with nvptx).</Explain>
      <Explain>This card shows the documented attempt on Kaggle: the build and run exit codes and the compiler's message. No OpenACC timings exist.</Explain>
      <Finding>OpenACC was attempted but not measured: build exit code {bc ?? 'n/a'}, run exit code {rc ?? 'n/a'} (127 = binary not found). {bl.s === 'ok' ? `The compiler said: “${bl.d.split('\n')[0].slice(0, 140)}”.` : ''}</Finding>
      <Panel title="OpenACC attempt (Kaggle)" settings={`${T4} host · gcc, source crates/kernels-c/openacc/acc_bench.c`}>
        {notes.s === 'ok' ? <Pre text={notes.d} max={200} /> : <NotRun what="openacc_notes.md" f={notes} />}
        {bl.s === 'ok' && <><b className="note">build log</b><Pre text={bl.d} max={200} /></>}
        {rl.s === 'ok' && <><b className="note">run log</b><Pre text={rl.d} max={120} /></>}
      </Panel>
    </>
  )
}
