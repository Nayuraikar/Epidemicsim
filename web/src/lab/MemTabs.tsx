import { Bar, BarChart, ComposedChart, ErrorBar, Line, Scatter } from 'recharts'
import { C } from '../colors'
import { macBench, okOf, pt, significance, sigText, tm, useJson, withCi, type Bench, type Machine } from './data'
import { Chart, Explain, f3, Finding, grid, legend, Need, PAL, Panel, Table, tip, x2, Xax, Yax, xlabel } from './ui'

interface Stat { median: number; mean: number; ci95: number; max: number }
interface Roof { name: string; threads: number; peak: number; unit: string; value: Stat; detail: string }
interface KPoint { kernel: string; variant: string; elements: number; working_set_mb: number; ns_per_element: Stat; bytes_per_element: number; int_ops_per_element: number; flops_per_element: number; arithmetic_intensity: number; achieved_gb_s: number; achieved_gops: number }
interface Roofline { machine: Machine; reps: number; thermal_before?: string | null; thermal_after?: string | null; notes: string[]; roofs: Roof[]; kernels: KPoint[]; gpu: string | null }
interface LRow { layout: string; order: string; pass_s: Stat; ns_per_agent: number; bytes_per_agent: number }
interface Layout { machine: Machine; agents: number; locations: number; passes_per_sample: number; warmup: number; reps: number; threads: number; thermal_before?: string | null; thermal_after?: string | null; checksums_match: boolean; note: string; rows: LRow[] }

const ORDERS = [{ k: 'generated', l: 'as generated' }, { k: 'sorted_by_location', l: 'sorted by location' }, { k: 'shuffled', l: 'shuffled' }]

function Roofline() {
  const r = useJson<Roofline>('roofline/mac.json')
  const sc = useJson<Bench>(macBench('simd_scalar')), ne = useJson<Bench>(macBench('simd_neon'))
  const d = okOf(r)
  const roof = (name: string, t: number) => d?.roofs.find((x) => x.name.startsWith(name) && x.threads === t)
  const bw1 = roof('memory', 1), bw8 = roof('memory', 8), fp1 = roof('peak FP32', 1), fp8 = roof('peak FP32', 8), in1 = roof('peak scalar', 1), in8 = roof('peak scalar', 8)
  const xs = Array.from({ length: 41 }, (_, i) => 0.05 * Math.pow(10, (i * 3.4) / 40))
  const line = (bw?: Roof, peak?: Roof) => (x: number) => (bw && peak ? Math.min(peak.peak, bw.peak * x) : undefined)
  const curve = xs.map((x) => ({ x, int1: line(bw1, in1)(x), int8: line(bw8, in8)(x), fp1: line(bw1, fp1)(x), fp8: line(bw8, fp8)(x) }))
  const kinds = d ? [...new Set(d.kernels.map((k) => `${k.kernel} ${k.variant}`))] : []
  const pts = (big: boolean) => d ? d.kernels.filter((k) => (k.elements > 2_000_000) === big).map((k) => ({ x: k.arithmetic_intensity, y: k.achieved_gops, name: `${k.kernel} ${k.variant} (${k.elements >> 20}M)` })) : []
  const k = (kernel: string, variant: string, elements = 1 << 20) => d?.kernels.find((x) => x.kernel === kernel && x.variant === variant && x.elements === elements)
  const kt = k('timer_recovery', 'scalar'), ktn = k('timer_recovery', 'neon')
  const sb = okOf(sc), nb = okOf(ne)
  const ps = pt(sb, 1_000_000), pn = pt(nb, 1_000_000)
  const stepS = ps ? tm(ps.wall_s)! / ps.steps : undefined
  const kernelSp = kt && ktn ? kt.ns_per_element.median / ktn.ns_per_element.median : undefined
  const share = kt && stepS ? (kt.ns_per_element.mean * 1e6 * 1e-9) / stepS : undefined // upper bound: every agent infected
  const bound = share != null && kernelSp ? 1 / (1 - share + share / kernelSp) : undefined
  const whole = ps && pn ? tm(ps.wall_s)! / tm(pn.wall_s)! : undefined
  const verdict = bound == null || whole == null ? null : Math.abs(bound - whole) <= 0.03 ? 'consistent' : bound > whole ? 'bound-above' : 'bound-below'
  return (
    <Need what="Roofline (roofline/mac.json)" files={[r]}>
      {d && (
        <>
          <Explain>A roofline plots achieved operations per second against arithmetic intensity (operations per byte moved). Under the sloped line a kernel is limited by memory bandwidth; under the flat line, by the machine's compute peak. Both agent kernels here are integer kernels (RNG and counters, no floating-point), so they are compared with the integer roof; the FP32 FMA roof is drawn for reference only.</Explain>
          <Finding>Measured on {d.machine.cpu}: triad bandwidth {bw1 ? f3(bw1.peak) : 'n/a'} GB/s (1 thread) and {bw8 ? f3(bw8.peak) : 'n/a'} GB/s (8 threads); NEON FP32 FMA peak {fp1 ? f3(fp1.peak) : 'n/a'} GFLOP/s (1 thread) and {fp8 ? f3(fp8.peak) : 'n/a'} (8); scalar integer mul+add peak {in1 ? f3(in1.peak) : 'n/a'} Gop/s (1 thread). Kernel arithmetic intensity: timer {kt ? f3(kt.arithmetic_intensity) : 'n/a'} ops/B, Philox stream {k('philox_stream', 'scalar') ? f3(k('philox_stream', 'scalar')!.arithmetic_intensity) : 'n/a'} ops/B.{kernelSp && whole && share != null && bound != null ? ` NEON vs scalar: timer kernel ${x2(kernelSp)} alone, whole run ${x2(whole)}. The timer kernel is at most ${(share * 100).toFixed(1)}% of a scalar step at 1M agents (upper bound: every agent infected), which by Amdahl's law caps the whole-run gain at ${x2(bound)}${verdict === 'consistent' ? ', matching the measured whole-run speedup: the small share of the kernel explains the small end-to-end gain.' : verdict === 'bound-above' ? `, above the measured ${x2(whole)}, so this share alone does not fully explain the gap (not explained by this data).` : `, below the measured ${x2(whole)}, so the kernel share does not explain it (the data does not support this explanation).`}` : ''}</Finding>
          <Panel title="Roofline (log-log)" settings={`${d.machine.cpu} · kernels single-threaded, ${d.reps} reps (+1 warm-up), median; roofs = best repetition · ops/bytes per element are static tallies of the kernel source · thermal before: ${d.thermal_before ?? 'n/a'}; after: ${d.thermal_after ?? 'n/a'}`}>
            <Chart h={340}><ComposedChart data={curve} margin={{ left: 4, right: 12, top: 6 }}>{grid}
              <Xax dataKey="x" type="number" scale="log" domain={[0.05, 120]} ticks={[0.1, 1, 10, 100]} label={xlabel('arithmetic intensity (ops / byte)')} />
              <Yax type="number" scale="log" domain={[0.1, 1000]} ticks={[0.1, 1, 10, 100, 1000]} label={{ value: 'Gop/s (integer) · GFLOP/s (FP32)', angle: -90, position: 'insideLeft', fontSize: 11 }} />
              {tip}{legend}
              <Line dataKey="int1" name="roof: memory + integer peak, 1 thread" stroke={C.S} strokeWidth={2} dot={false} isAnimationActive={false} />
              <Line dataKey="int8" name="roof: memory + integer peak, 8 threads" stroke={C.S} strokeDasharray="5 3" dot={false} isAnimationActive={false} />
              <Line dataKey="fp1" name="roof: memory + FP32 FMA peak, 1 thread" stroke={C.E} strokeWidth={1.5} dot={false} isAnimationActive={false} />
              <Line dataKey="fp8" name="roof: memory + FP32 FMA peak, 8 threads" stroke={C.E} strokeDasharray="5 3" dot={false} isAnimationActive={false} />
              <Scatter data={pts(false)} name="kernels, 1M elements (cache-resident)" fill={C.I} isAnimationActive={false} />
              <Scatter data={pts(true)} name="kernels, 32M elements (DRAM-resident)" fill={C.R} shape="diamond" isAnimationActive={false} />
            </ComposedChart></Chart>
            <p className="note">GPU roofline: not measured (placeholder; needs a GPU run).</p>
          </Panel>
          <Panel title="Roofs and kernel points" settings={d.notes.join(' ')}>
            <Table head={['roof', 'threads', 'peak', 'mean ± 95% CI']} rows={d.roofs.map((x) => [x.name, x.threads, `${f3(x.peak)} ${x.unit}`, `${f3(x.value.mean)} ± ${f3(x.value.ci95)}`])} />
            <Table head={['kernel', 'elements', 'bytes/elt', 'ops/elt', 'ops/byte', 'ns/elt (mean ± CI)', 'GB/s', 'Gop/s']} rows={d.kernels.map((x) => [`${x.kernel} ${x.variant}`, `${x.elements >> 20} M`, x.bytes_per_element, x.int_ops_per_element, f3(x.arithmetic_intensity), `${f3(x.ns_per_element.mean)} ± ${f3(x.ns_per_element.ci95)}`, f3(x.achieved_gb_s), f3(x.achieved_gops)])} />
            <p className="note">{kinds.length} kernel/variant series. Working sets (MB): {d.kernels.filter((x) => x.variant === 'scalar').map((x) => `${x.kernel} ${x.elements >> 20}M: ${f3(x.working_set_mb)}`).join('; ')}.</p>
          </Panel>
        </>
      )}
    </Need>
  )
}

function LayoutSection() {
  const l = useJson<Layout>('layout/layout_apple_m1.json')
  const d = okOf(l)
  const row = (layout: string, order: string) => d?.rows.find((r) => r.layout === layout && r.order === order)
  const data = ORDERS.map((o) => ({ o: o.l, soa: row('soa', o.k)?.ns_per_agent, soa_e: row('soa', o.k) ? (row('soa', o.k)!.pass_s.ci95 * 1e9) / d!.agents : 0, aos: row('aos', o.k)?.ns_per_agent, aos_e: row('aos', o.k) ? (row('aos', o.k)!.pass_s.ci95 * 1e9) / d!.agents : 0 }))
  const verdicts = d ? [
    ...ORDERS.map((o) => ({ what: `AoS vs SoA, ${o.l}`, a: row('soa', o.k), b: row('aos', o.k) })),
    ...['soa', 'aos'].flatMap((lay) => [{ what: `${lay.toUpperCase()}: shuffled vs sorted by location`, a: row(lay, 'sorted_by_location'), b: row(lay, 'shuffled') }]),
  ].map((v) => ({ what: v.what, ratio: v.a && v.b ? v.b.ns_per_agent / v.a.ns_per_agent : NaN, sig: significance(v.a?.pass_s, v.b?.pass_s) })) : []
  return (
    <Need what="Memory-layout benchmark (layout/layout_apple_m1.json)" files={[l]}>
      {d && (
        <>
          <Explain>Structure-of-arrays (SoA) stores each agent field in its own array; array-of-structures (AoS) stores one 16-byte record per agent. The update kernel reads a random location's infection probability, so the order of agents also matters: sorted by location, neighbouring agents hit neighbouring locations; shuffled, they do not.</Explain>
          <Finding>{verdicts.map((v) => `${v.what}: ${isFinite(v.ratio) ? x2(v.ratio) : 'n/a'} time ratio (${sigText(v.sig)})`).join('; ')}. {d.checksums_match ? 'All six variants produced identical results.' : 'WARNING: result checksums differ between variants.'}</Finding>
          <Panel title="Update kernel: nanoseconds per agent (lower is better)" settings={`${d.machine.cpu} · ${(d.agents / 1e6).toFixed(1)}M agents, ${d.locations.toLocaleString()} locations · 1 thread · ${d.passes_per_sample} passes per sample · ${d.reps} reps (+${d.warmup} warm-up), median · error bars 95% CI of the mean · thermal before: ${d.thermal_before ?? 'n/a'}; after: ${d.thermal_after ?? 'n/a'}`}>
            <Chart h={240}><BarChart data={data}>{grid}<Xax dataKey="o" /><Yax unit=" ns" />{tip}{legend}
              <Bar dataKey="soa" name="SoA (15 B/agent)" fill={C.S} isAnimationActive={false}><ErrorBar dataKey="soa_e" stroke={C.D} /></Bar>
              <Bar dataKey="aos" name="AoS (16 B/agent)" fill={PAL[3]} isAnimationActive={false}><ErrorBar dataKey="aos_e" stroke={C.D} /></Bar></BarChart></Chart>
            <Table head={['layout', 'order', 'ms per pass (mean ± 95% CI)', 'ns / agent (median)']} rows={d.rows.map((r) => [r.layout, r.order, withCi(r.pass_s, (v) => (v * 1e3).toFixed(2)), f3(r.ns_per_agent)])} />
            <p className="note">{d.note}</p>
          </Panel>
        </>
      )}
    </Need>
  )
}

export function MemoryTab() {
  return (
    <>
      <h2 className="lab-h">Roofline (CPU)</h2>
      <Roofline />
      <h2 className="lab-h">Memory layout</h2>
      <LayoutSection />
    </>
  )
}
