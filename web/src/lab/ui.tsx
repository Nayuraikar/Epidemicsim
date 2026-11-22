import { useState, type ComponentProps, type ReactNode } from 'react'
import { CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import { C } from '../colors'
import type { Fetched } from './data'

/** Series colours: scenario identity + SEIR constants from the Studio (theme.css). */
export const PAL = [C.S, C.I, C.R, C.E, C.runs[1], C.runs[2], C.runs[0], C.D]
export const axis = { stroke: C.line, tick: { fill: C.muted, fontSize: 11 } }
export const grid = <CartesianGrid stroke={C.line} strokeDasharray="3 3" />
export const tip = <Tooltip contentStyle={{ borderRadius: 10, border: `1px solid ${C.line}`, fontSize: 12 }} />
export const legend = <Legend wrapperStyle={{ fontSize: 12 }} />
export const Xax = (p: ComponentProps<typeof XAxis>) => <XAxis {...axis} {...p} />
export const Yax = (p: ComponentProps<typeof YAxis>) => <YAxis {...axis} {...p} />
export const Chart = ({ children, h = 260 }: { children: React.ReactElement; h?: number }) => <ResponsiveContainer width="100%" height={h}>{children}</ResponsiveContainer>

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
    <div className="card lab-notrun">
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
export const Finding = ({ children }: { children: ReactNode }) => <div className="lab-finding"><b>Key finding</b><span>{children}</span></div>
export const Settings = ({ children }: { children: ReactNode }) => <div className="lab-settings">Run settings: {children}</div>
export function Panel({ title, settings, children }: { title: string; settings?: ReactNode; children: ReactNode }) {
  return <section className="card lab-panel"><h3>{title}</h3>{children}{settings && <Settings>{settings}</Settings>}</section>
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
