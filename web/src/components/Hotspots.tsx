import { useMemo } from 'react'
import { Area, AreaChart, CartesianGrid, Legend, ResponsiveContainer, Tooltip, XAxis, YAxis } from 'recharts'
import type { Run } from '../run'
import { C } from '../colors'

// Zones are neutral-hued (not the compartment colours) so they never read as S/E/I/R/D.
const ZCOL = ['#334155', '#7c3aed', '#0891b2', '#be185d', '#a16207', '#4d7c0f', '#0f766e', '#9333ea', '#64748b']

/** New infections per day, stacked by the zone where the infection happened. */
export function Hotspots({ run, version }: { run: Run; version: number }) {
  const { rows, top, kinds } = useMemo(() => {
    const zones = run.meta?.grid.zones ?? []
    const nd = Math.ceil(run.frames.length / 4)
    const per = Array.from({ length: nd }, () => new Array(zones.length).fill(0) as number[])
    const kinds = [0, 0, 0, 0]
    for (const f of run.frames) {
      for (let i = 0; i < f.hot.length; i += 2) per[f.day][f.hot[i]] += f.hot[i + 1]
      f.kinds.forEach((v, j) => (kinds[j] += v))
    }
    const total = zones.map((_, z) => per.reduce((s, d) => s + d[z], 0))
    const top = zones.map((n, z) => ({ n, z, t: total[z] })).sort((a, b) => b.t - a.t).slice(0, 5)
    const rows = per.map((d, day) => {
      const r: Record<string, number> = { day }
      top.forEach(({ n, z }) => (r[n] = d[z]))
      r.Other = d.reduce((s, v, z) => s + (top.some((t) => t.z === z) ? 0 : v), 0)
      return r
    })
    return { rows, top, kinds }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [run, version, run.frames.length])
  const tot = kinds.reduce((a, b) => a + b, 0) || 1
  const names = ['homes', 'work/school', 'markets', 'hospitals']
  return (
    <div>
      <ResponsiveContainer width="100%" height={220}>
        <AreaChart data={rows} margin={{ top: 4, right: 8, left: -8, bottom: 0 }}>
          <CartesianGrid stroke={C.line} strokeDasharray="3 3" />
          <XAxis dataKey="day" stroke={C.line} tick={{ fill: C.muted, fontSize: 11 }} />
          <YAxis stroke={C.line} tick={{ fill: C.muted, fontSize: 11 }} />
          <Tooltip formatter={(v) => (typeof v === 'number' ? (Math.round(v * 10) / 10).toFixed(1) : String(v))} labelFormatter={(d) => `day ${d}`} />
          {top.map(({ n }, i) => <Area key={n} dataKey={n} stackId="z" stroke={ZCOL[i]} fill={ZCOL[i]} fillOpacity={0.55} isAnimationActive={false} />)}
          <Area dataKey="Other" stackId="z" stroke="#cbd5e1" fill="#e2e8f0" isAnimationActive={false} />
          <Legend />
        </AreaChart>
      </ResponsiveContainer>
      <div className="note">Where infections happened: {kinds.map((v, j) => `${names[j]} ${Math.round((100 * v) / tot)}%`).join(' · ')}</div>
    </div>
  )
}
