import { Area, CartesianGrid, ComposedChart, Legend, Line, ReferenceLine, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import type { Pt } from '../types'
import { C } from '../colors'

const SEIR: [string, string, string][] = [['S', C.S, 'Susceptible'], ['E', C.E, 'Exposed'], ['I', C.I, 'Infectious (A+I)'], ['R', C.R, 'Recovered'], ['D', C.D, 'Dead']]
const k = (v: number) => (v >= 1e6 ? `${(v / 1e6).toFixed(1)}M` : v >= 1e4 ? `${Math.round(v / 1e3)}k` : v >= 1e3 ? `${(v / 1e3).toFixed(1)}k` : String(Math.round(v)))
const axis = { stroke: C.line, tick: { fill: C.muted, fontSize: 11 } }

/** SEIR(D) curves with 5-95% bands (bands only when the data came from an ensemble). */
export function SeirChart({ data, yMax, banded }: { data: Pt[]; yMax: number; banded: boolean }) {
  return (
    <ResponsiveContainer width="100%" height={240}>
      <ComposedChart data={data} margin={{ top: 4, right: 8, left: -8, bottom: 0 }}>
        <CartesianGrid stroke={C.line} strokeDasharray="3 3" />
        <XAxis dataKey="day" {...axis} />
        <YAxis {...axis} tickFormatter={k} domain={[0, yMax]} allowDataOverflow />
        <Tooltip formatter={(v) => (Array.isArray(v) ? `${k(+v[0])} – ${k(+v[1])}` : k(+(v ?? 0)))} labelFormatter={(d) => `day ${d}`} />
        {banded && SEIR.map(([key, col]) => <Area key={key + 'b'} dataKey={`${key}_b`} stroke="none" fill={col} fillOpacity={0.16} isAnimationActive={false} legendType="none" tooltipType="none" />)}
        {SEIR.map(([key, col, name]) => <Line key={key} dataKey={key} name={name} stroke={col} strokeWidth={2} dot={false} isAnimationActive={false} />)}
      </ComposedChart>
    </ResponsiveContainer>
  )
}

export interface Series { label: string; color: string; data: Pt[]; capacity: number; banded: boolean }

/** Hospital load of every run on one axis with the capacity line(s). */
export function HospitalChart({ series }: { series: Series[] }) {
  const days = Math.max(...series.map((s) => s.data.length), 1)
  const rows = Array.from({ length: days }, (_, i) => {
    const row: Record<string, number | [number, number]> = { day: i }
    series.forEach((s, j) => { const p = s.data[i]; if (p) { row[`h${j}`] = p.hosp as number; if (p.hosp_b) row[`hb${j}`] = p.hosp_b as [number, number] } })
    return row
  })
  const caps = [...new Set(series.map((s) => Math.round(s.capacity)))]
  return (
    <ResponsiveContainer width="100%" height={240}>
      <ComposedChart data={rows} margin={{ top: 4, right: 8, left: -8, bottom: 0 }}>
        <CartesianGrid stroke={C.line} strokeDasharray="3 3" />
        <XAxis dataKey="day" {...axis} />
        <YAxis {...axis} tickFormatter={k} />
        <Tooltip formatter={(v) => (Array.isArray(v) ? `${k(+v[0])} – ${k(+v[1])}` : k(+(v ?? 0)))} labelFormatter={(d) => `day ${d}`} />
        {caps.map((c) => <ReferenceLine key={c} y={c} stroke={C.I} strokeDasharray="6 4" label={{ value: `capacity ${k(c)}`, fill: C.I, fontSize: 11, position: 'insideTopRight' }} />)}
        {series.map((s, j) => s.banded && <Area key={'b' + j} dataKey={`hb${j}`} stroke="none" fill={s.color} fillOpacity={0.14} isAnimationActive={false} legendType="none" tooltipType="none" />)}
        {series.map((s, j) => <Line key={j} dataKey={`h${j}`} name={s.label} stroke={s.color} strokeWidth={2} dot={false} isAnimationActive={false} />)}
        <Legend />
      </ComposedChart>
    </ResponsiveContainer>
  )
}

/** Effective reproduction number R_t (daily mean) with the R_t = 1 threshold. */
export function RtChart({ series }: { series: Series[] }) {
  const days = Math.max(...series.map((s) => s.data.length), 1)
  const rows = Array.from({ length: days }, (_, i) => {
    const row: Record<string, number | [number, number]> = { day: i }
    series.forEach((s, j) => { const p = s.data[i]; if (p) { row[`r${j}`] = Math.round((p.rt as number) * 100) / 100; if (p.rt_b) row[`rb${j}`] = p.rt_b as [number, number] } })
    return row
  })
  return (
    <ResponsiveContainer width="100%" height={240}>
      <ComposedChart data={rows} margin={{ top: 4, right: 8, left: -8, bottom: 0 }}>
        <CartesianGrid stroke={C.line} strokeDasharray="3 3" />
        <XAxis dataKey="day" {...axis} />
        <YAxis {...axis} domain={[0, 4]} allowDataOverflow />
        <Tooltip labelFormatter={(d) => `day ${d}`} />
        <ReferenceLine y={1} stroke={C.muted} strokeDasharray="6 4" label={{ value: 'R = 1', fill: C.muted, fontSize: 11, position: 'insideTopRight' }} />
        {series.map((s, j) => s.banded && <Area key={'b' + j} dataKey={`rb${j}`} stroke="none" fill={s.color} fillOpacity={0.14} isAnimationActive={false} legendType="none" tooltipType="none" />)}
        {series.map((s, j) => <Line key={j} dataKey={`r${j}`} name={s.label} stroke={s.color} strokeWidth={2} dot={false} isAnimationActive={false} />)}
        <Legend />
      </ComposedChart>
    </ResponsiveContainer>
  )
}
