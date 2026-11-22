import { useState, type ComponentProps, type ReactNode } from 'react'
import { CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { C } from '../colors'
import type { Fetched } from './data'

/** Series colours: scenario identity + SEIR constants from the Studio (theme.css). */
export const PAL = [C.S, C.I, C.R, C.E, C.runs[1], C.runs[2], C.runs[0], C.D]
const MONO = 'ui-monospace, "SF Mono", Menlo, monospace'
export const axis = { stroke: C.muted, tick: { fill: C.muted, fontSize: 11, fontFamily: MONO }, tickLine: false }
export const grid = <CartesianGrid stroke={C.line} strokeDasharray="1 3" />
/** Tooltip values: 1 decimal place everywhere (numbers and [lo, hi] ranges), unit appended when given. */
export const r1 = (v: unknown) => (typeof v === 'number' && isFinite(v) ? (Math.round(v * 10) / 10).toFixed(1) : String(v ?? ''))
const tipFmt = (unit = '') => (v: unknown) => (Array.isArray(v) ? `${r1(v[0])} – ${r1(v[1])}${unit}` : `${r1(v)}${unit}`)
const tipStyle = { borderRadius: 2, border: `1px solid ${C.ink}`, boxShadow: 'none', background: '#fff', fontSize: 12, fontFamily: MONO }
export const tipU = (unit = '') => <Tooltip contentStyle={tipStyle} formatter={tipFmt(unit)} labelFormatter={(l) => (typeof l === 'number' ? r1(l).replace(/\.0$/, '') : l)} />
export const tip = tipU()
/** Legend always sits below the plot (and below any x-axis label). */
export const legend = <Legend verticalAlign="bottom" align="center" height={32} wrapperStyle={{ fontSize: 11, paddingTop: 8, fontFamily: MONO, color: C.muted }} iconType="plainline" />
/** x-axis label that stays inside the axis band, clear of the legend. */
export const xlabel = (value: string) => ({ value, position: 'insideBottom' as const, offset: -4, fontSize: 11, fill: C.muted, fontFamily: MONO })
export const Xax = (p: ComponentProps<typeof XAxis>) => <XAxis {...axis} {...(p.label ? { height: 44 } : {})} {...p} />
export const Yax = (p: ComponentProps<typeof YAxis>) => <YAxis {...axis} {...p} />
export const Chart = ({ children, h = 280 }: { children: React.ReactElement; h?: number }) => <ResponsiveContainer width="100%" height={h}>{children}</ResponsiveContainer>

export const f3 = (v: number | null | undefined) => (v == null || !isFinite(v) ? 'n/a' : v >= 100 ? v.toFixed(0) : v >= 10 ? v.toFixed(1) : v >= 1 ? v.toFixed(2) : v >= 0.01 ? v.toFixed(3) : v.toPrecision(2))
export const sec = (v: number | null | undefined) => (v == null ? 'n/a' : `${f3(v)} s`)
export const x2 = (v: number | null | undefined) => (v == null || !isFinite(v) ? 'n/a' : `${v.toFixed(2)}x`)
export const big = (v: number) => (v >= 1e9 ? `${(v / 1e9).toFixed(2)}G` : v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v >= 1e3 ? `${(v / 1e3).toFixed(0)}k` : String(v))

/** Card shown when a results file is missing/empty/unreadable. */
export function NotRun({ what, f }: { what: string; f?: Fetched<unknown> | Fetched<unknown>[] }) {
  const list = Array.isArray(f) ? f : f ? [f] : []
  const loading = list.some((x) => x.s === 'loading')
  const err = list.find((x) => x.s === 'error') as { m: string } | undefined
  return (
    <div className="lab-notrun">
      <b>{loading ? 'Loading…' : err ? 'Could not read' : 'Not run'}</b>
      <span>{loading ? what : err ? `${what}: ${err.m}` : `${what} is missing or empty in results/. Run the benchmark that writes it, then reload.`}</span>
    </div>
  )
}
/** Renders children only when every fetched file is ok; else a NotRun card. */
export function Need({ what, files, children }: { what: string; files: Fetched<unknown>[]; children: ReactNode }) {
  return files.every((f) => f.s === 'ok') ? <>{children}</> : <NotRun what={what} f={files.find((f) => f.s !== 'ok')} />
}

export const Explain = ({ children }: { children: ReactNode }) => <p className="lab-explain">{children}</p>
export const Finding = ({ children }: { children: ReactNode }) => <div className="lab-finding"><b>Finding</b><span>{children}</span></div>
export const Settings = ({ children }: { children: ReactNode }) => <div className="lab-settings">RUN SETTINGS · {children}</div>
export function Panel({ title, settings, children }: { title: string; settings?: ReactNode; children: ReactNode }) {
  return <section className="lab-panel"><h3>{title}</h3>{children}{settings && <Settings>{settings}</Settings>}</section>
}
export function Table({ head, rows }: { head: string[]; rows: ReactNode[][] }) {
  return (
    <div className="lab-tablewrap"><table className="lab-table">
      <thead><tr>{head.map((h) => <th key={h}>{h}</th>)}</tr></thead>
      <tbody>{rows.map((r, i) => <tr key={i}>{r.map((c, j) => <td key={j}>{c}</td>)}</tr>)}</tbody>
    </table></div>
  )
}
export function Select<T extends string>({ value, onChange, options, label }: { value: T; onChange: (v: T) => void; options: { v: T; l: string }[]; label?: string }) {
  return <label className="lab-select">{label}<select value={value} onChange={(e) => onChange(e.target.value as T)}>{options.map((o) => <option key={o.v} value={o.v}>{o.l}</option>)}</select></label>
}
export function Collapsible({ title, children }: { title: string; children: ReactNode }) {
  const [open, setOpen] = useState(false)
  return <div className="lab-collapse"><button className="btn" onClick={() => setOpen(!open)}>{open ? 'Hide' : 'Show'} {title}</button>{open && children}</div>
}
export const Pre = ({ text, max = 420 }: { text: string; max?: number }) => <pre className="lab-pre" style={{ maxHeight: max }}>{text}</pre>
