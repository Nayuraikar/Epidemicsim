import { C } from '../colors'
import { kgBench, macBench, M1, T4, XEON, okOf, pt, useJson, useJsons, useText, type Bench } from './data'
import { big, Explain, Finding, Panel, sec, Table, x2 } from './ui'

type Golden = { st: 'PASS' | 'FAIL' | 'reference' | 'unknown'; src: string }
const PROG = 'docs/PROGRESS.md (Phase 4/5)'
const tone = (g: Golden) => (g.st === 'PASS' ? C.R : g.st === 'FAIL' ? C.I : C.muted)
interface Chk { matches_sequential: boolean | null }

export function SummaryTab() {
  const b = useJsons<Bench>({
    seq: macBench('sequential'), ilp: macBench('ilp_optimised'), ray: macBench('rayon_guided_c64'), omp: macBench('openmp_guided_c64'), neon: macBench('simd_neon'), metal: macBench('metal'), mpi: macBench('mpi'),
    kseq: kgBench('sequential'), kray: kgBench('rayon_guided_c64'), kavx: kgBench('simd_avx2'), cuda: kgBench('cuda'),
  })
  const gt = useText('kaggle/logs/test_golden_table.log'), cg = useText('kaggle/logs/test_cuda_golden.log'), ag = useText('kaggle/logs/test_avx2_golden.log')
  const sc = useJson<{ identical: boolean; sequential: { agents: number; days: number; seed: number; hash: string }; cuda: { hash: string } }>('kaggle/synth_compare.json')
  const mb = useJson<{ rows: Chk[] }>('gpu/metal_breakdown.json'), cb = useJson<{ rows: Chk[] }>('kaggle/gpu/cuda_breakdown.json'), cvv = useJson<{ rows: Chk[] }>('kaggle/gpu/cuda_variants.json'), cw = useJson<{ rows: Chk[] }>('kaggle/gpu/cuda_warp.json')
  const mv = useJson<{ rows: { agents: number }[] }>('gpu/metal_vs_cpu.json')

  // golden table rows parsed from the Kaggle log
  const rows = gt.s === 'ok' ? [...gt.d.matchAll(/^(\S.*?)\s{2,}(PASS|FAIL)\s+(PASS|FAIL)\s*$/gm)].map((m) => ({ name: m[1].trim(), ok: m[2] === 'PASS' && m[3] === 'PASS' })) : []
  const kg = (name: string): Golden => { const r = rows.find((x) => x.name === name); return r ? { st: r.ok ? 'PASS' : 'FAIL', src: 'kaggle/logs/test_golden_table.log' } : { st: 'unknown', src: 'row not in log' } }
  const logOk = (t: ReturnType<typeof useText>): Golden => (t.s === 'ok' ? { st: /test result: ok\./.test(t.d) ? 'PASS' : 'FAIL', src: 'test log' } : { st: 'unknown', src: 'log missing' })
  const flags = (c: Chk[] | undefined) => (c ? { n: c.length, ok: c.filter((r) => r.matches_sequential === true).length, no: c.filter((r) => r.matches_sequential === false).length } : null)
  const cudaFlags = [okOf(cb), okOf(cvv), okOf(cw)].map((x) => flags(x?.rows)).filter((x) => x)
  const cudaAll = cudaFlags.length ? { n: cudaFlags.reduce((a, x) => a + x!.n, 0), ok: cudaFlags.reduce((a, x) => a + x!.ok, 0), no: cudaFlags.reduce((a, x) => a + x!.no, 0) } : null
  const cudaGold: Golden = logOk(cg).st === 'PASS' && cudaAll?.no === 0 ? { st: 'PASS', src: 'kaggle/logs/test_cuda_golden.log + matches_sequential flags' } : logOk(cg)

  type Cell = { s: number | null; base: number | null; label?: string; g: Golden; note?: string }
  const t1 = (x: Bench | null, th = 1, n = 1_000_000) => pt(x, n, th)?.wall_s.median ?? null
  const mseq = t1(okOf(b.seq)), kseq = t1(okOf(b.kseq))
  const NA: Cell = { s: null, base: null, g: { st: 'unknown', src: '' } }
  const matrix: { name: string; m1: Cell; xeon: Cell; gpu: Cell }[] = [
    { name: 'Sequential (reference)', m1: { s: mseq, base: mseq, g: { st: 'reference', src: 'is the reference' } }, xeon: { s: kseq, base: kseq, g: { st: 'reference', src: 'is the reference' } }, gpu: NA },
    { name: 'ILP optimised kernels', m1: { s: t1(okOf(b.ilp)), base: mseq, g: { st: 'PASS', src: PROG } }, xeon: { ...NA, label: 'ilp optimised', s: null, g: kg('ilp optimised') }, gpu: NA },
    { name: 'rayon guided (M1 8 threads, Xeon 4)', m1: { s: t1(okOf(b.ray), 8), base: mseq, g: { st: 'PASS', src: PROG } }, xeon: { s: t1(okOf(b.kray), 4), base: kseq, g: kg('rayon guided (4t)') }, gpu: NA },
    { name: 'OpenMP guided (8 threads)', m1: { s: t1(okOf(b.omp), 8), base: mseq, g: { st: 'PASS', src: PROG } }, xeon: { ...NA, g: kg('openmp guided (4t)'), label: 'golden only, not benchmarked' }, gpu: NA },
    { name: 'SIMD (NEON on M1, AVX2 on Xeon)', m1: { s: t1(okOf(b.neon)), base: mseq, g: { st: 'PASS', src: PROG } }, xeon: { s: t1(okOf(b.kavx)), base: kseq, g: logOk(ag) }, gpu: NA },
    { name: 'Metal (M1 GPU)', m1: { s: t1(okOf(b.metal)), base: mseq, g: { st: 'PASS', src: `${PROG}; 100k and 1M checked, 10M not` } }, xeon: NA, gpu: NA },
    { name: 'CUDA (T4 GPU)', m1: NA, xeon: NA, gpu: { s: t1(okOf(b.cuda)), base: kseq, g: cudaGold, note: 'speedup vs Xeon sequential' } },
  ]
  const mpi = okOf(b.mpi)
  const mp = pt(mpi, 50_000, 8)

  const cell = (c: Cell) => c.s == null && c.g.st === 'unknown' && !c.label ? <span className="note">not run</span> : (
    <div className="lab-cell">
      {c.s != null ? <><b>{sec(c.s)}</b>{c.base != null && <span>{x2(c.base / c.s)} vs sequential{c.note ? ` (${c.note})` : ''}</span>}</> : <span className="note">{c.label ?? 'not benchmarked'}</span>}
      <em style={{ color: tone(c.g) }} title={c.g.src}>golden: {c.g.st}</em>
    </div>
  )
  const metalRows = okOf(mb)?.rows ?? []
  const mflags = flags(metalRows)
  const verif: [string, string, string, string, string][] = [
    ['rayon / OpenMP / ILP vs sequential', M1, 'golden table: 3k agents, 40 days, seed 42, with and without vaccination', 'PASS', PROG],
    ['SIMD NEON vs sequential', M1, 'golden table, same settings', 'PASS', PROG],
    ['rayon / OpenMP / ILP vs sequential', XEON, 'golden table (same settings); NEON row panicked on x86 after these rows (test now skips NEON on non-aarch64; not re-run on Kaggle)', rows.length ? `${rows.filter((r) => r.ok).length} of ${rows.length} rows PASS` : 'log missing', 'kaggle/logs/test_golden_table.log'],
    ['SIMD AVX2 vs sequential', XEON, 'avx2_matches_on_x86_64', logOk(ag).st, 'kaggle/logs/test_avx2_golden.log'],
    ['Metal vs sequential (100k, 1M)', M1, `${mflags ? `${mflags.ok} of ${mflags.n} breakdown runs matched` : 'no breakdown file'}; per-step stats identical`, mflags && mflags.no === 0 && mflags.ok > 0 ? 'PASS' : 'unknown', 'results/gpu/metal_breakdown.json; ' + PROG],
    ['Metal vs sequential (10M)', M1, 'sequential reference too slow at 10M', 'not checked against sequential', 'results/gpu/metal_vs_cpu.json (10M row)' + (okOf(mv)?.rows.some((r) => r.agents === 10_000_000) ? '' : ' (row absent)')],
    ['CUDA variants vs sequential', T4, `golden test cuda_variants_match; ${cudaAll ? `${cudaAll.ok} of ${cudaAll.n} benchmark runs flagged matches_sequential` : ''}`, cudaGold.st, 'kaggle/logs/test_cuda_golden.log; kaggle/gpu/*.json'],
    ['CUDA vs sequential run output', T4, okOf(sc) ? `${big(okOf(sc)!.sequential.agents)} agents, ${okOf(sc)!.sequential.days} days, seed ${okOf(sc)!.sequential.seed}; hash ${okOf(sc)!.sequential.hash} vs ${okOf(sc)!.cuda.hash}` : 'no file', okOf(sc) ? (okOf(sc)!.identical ? 'PASS' : 'FAIL') : 'unknown', 'kaggle/synth_compare.json'],
    ['MPI vs sequential', M1, 'static and dynamic, 3-4 ranks, byte-identical outputs', 'PASS', PROG],
  ]
  const pass = verif.filter((v) => v[3] === 'PASS' || v[3].includes('PASS')).length
  return (
    <>
      <Explain>Every backend on every machine we have numbers for, at 1M agents and 80 steps (20 days) unless noted. Speedup is against the sequential run on the same hardware.</Explain>
      <Explain>"Golden" means the backend reproduces the sequential reference's per-step statistics and run hash exactly. Pass/fail comes from the saved test logs (Kaggle) or from docs/PROGRESS.md (Mac, whose logs were not saved).</Explain>
      <Finding>{verif.length} verification rows: {pass} PASS, {verif.filter((v) => v[3].startsWith('not checked')).length} not checked against sequential (Metal 10M). {okOf(b.metal) && okOf(b.cuda) ? `Fastest 1M run: CUDA on T4 ${sec(t1(okOf(b.cuda)))} (${x2((kseq ?? NaN) / (t1(okOf(b.cuda)) ?? NaN))} vs Xeon sequential); Metal on M1 ${sec(t1(okOf(b.metal)))} (${x2((mseq ?? NaN) / (t1(okOf(b.metal)) ?? NaN))} vs M1 sequential).` : ''} Kaggle and Mac are different machines, so cross-machine times are not a like-for-like comparison.</Finding>
      <Panel title="Backend x hardware" settings={`1M agents · 20 days (80 steps) · 5 reps (+1 warm-up) · median wall time · seed 42 · ${M1} 8 cores / ${XEON} / ${T4}`}>
        <Table head={['Backend', M1, XEON, T4]} rows={[
          ...matrix.map((r) => [r.name, cell(r.m1), cell(r.xeon), cell(r.gpu)]),
          ['MPI (different workload)', mpi && mp ? <div className="lab-cell"><b>{sec(mp.wall_s.median)}</b><span>{x2(mp.speedup)} vs 1 process; 8 processes, 50k agents, {mpi.days} days, 8 scenarios x 6 runs (not comparable to rows above)</span><em style={{ color: C.R }}>golden: PASS</em></div> : <span className="note">not run</span>, <span className="note">not run</span>, <span className="note">not run</span>],
        ]} />
      </Panel>
      <Panel title="Cross-backend verification" settings="each row cites its source file">
        <Table head={['Check', 'Hardware', 'Scope', 'Result', 'Source']} rows={verif.map((v) => [v[0], v[1], v[2], <b key="r" style={{ color: v[3].includes('PASS') ? C.R : v[3].includes('FAIL') ? C.I : C.E }}>{v[3]}</b>, <span key="s" className="note">{v[4]}</span>])} />
      </Panel>
      <p className="note">Speedups use sequential medians of {sec(mseq)} (M1) and {sec(kseq)} (Xeon) at 1M agents.</p>
    </>
  )
}
