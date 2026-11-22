import { useEffect, useState } from 'react'

/** Result of reading one file from results/ through /api/lab/file. */
export type Fetched<T> = { s: 'loading' } | { s: 'missing' } | { s: 'empty' } | { s: 'error'; m: string } | { s: 'ok'; d: T }

const cache = new Map<string, Promise<Fetched<string>>>()
function raw(path: string): Promise<Fetched<string>> {
  let p = cache.get(path)
  if (!p) {
    p = fetch(`/api/lab/file/${path}`)
      .then(async (r): Promise<Fetched<string>> => {
        if (r.status === 404) return { s: 'missing' }
        if (!r.ok) return { s: 'error', m: `${r.status} ${(await r.text()).slice(0, 120)}` }
        const t = await r.text()
        return t.trim() === '' ? { s: 'empty' } : { s: 'ok', d: t }
      })
      .catch((e): Fetched<string> => ({ s: 'error', m: String(e?.message ?? e) }))
    cache.set(path, p)
  }
  return p
}

function useRaw(path: string): Fetched<string> {
  const [v, set] = useState<Fetched<string>>({ s: 'loading' })
  useEffect(() => { let on = true; raw(path).then((r) => on && set(r)); return () => { on = false } }, [path])
  return v
}
export function useText(path: string): Fetched<string> { return useRaw(path) }
export function useJson<T>(path: string): Fetched<T> {
  const r = useRaw(path)
  if (r.s !== 'ok') return r
  try { return { s: 'ok', d: JSON.parse(r.d) as T } } catch (e) { return { s: 'error', m: `invalid JSON: ${String((e as Error).message)}` } }
}
/** Several JSON files at once, keyed by the given names. */
export function useJsons<T>(paths: Record<string, string>): Record<string, Fetched<T>> {
  const [v, set] = useState<Record<string, Fetched<T>>>({})
  const key = JSON.stringify(paths)
  useEffect(() => {
    let on = true
    Object.entries(paths).forEach(([k, p]) => raw(p).then((r) => {
      let out: Fetched<T>
      if (r.s !== 'ok') out = r
      else { try { out = { s: 'ok', d: JSON.parse(r.d) as T } } catch (e) { out = { s: 'error', m: `invalid JSON: ${String((e as Error).message)}` } } }
      if (on) set((o) => ({ ...o, [k]: out }))
    }))
    return () => { on = false }
    // eslint-disable-next-line react-hooks/exhaustive-deps
  }, [key])
  return new Proxy(v, { get: (t, k: string) => t[k] ?? { s: 'loading' } }) as Record<string, Fetched<T>>
}
export const okOf = <T,>(f: Fetched<T> | undefined): T | null => (f && f.s === 'ok' ? f.d : null)

// ---- file names ----
export const M1 = 'Apple M1'
export const XEON = 'Xeon 2.0 GHz (Kaggle, 4 cores)'
export const T4 = 'Tesla T4 (Kaggle)'
const X = 'intel_r_xeon_r_cpu_2_00ghz'
export const macBench = (name: string) => `bench/${name}_apple_m1.json`
export const kgBench = (name: string) => `kaggle/bench/${name}_${X}.json`

// ---- types of the result files ----
export interface Machine { os: string; arch: string; cpu: string; logical_cores: number; memory_gb: number }
export interface BPoint { agents: number; threads: number; steps: number; wall_s: { median: number; min: number; max: number }; agents_per_sec: number; contacts_per_sec: number; speedup: number; efficiency: number }
export interface Bench { backend: string; machine: Machine; days: number; warmup: number; reps: number; seed: number; points: BPoint[]; amdahl: { agents: number; serial_fraction: number; max_speedup: number }[] }
export const pt = (b: Bench | null, agents: number, threads = 1): BPoint | undefined => b?.points.find((p) => p.agents === agents && p.threads === threads)
export const settingsOf = (b: Bench | null, hw: string, extra = ''): string =>
  b ? `${hw} · ${b.days} days (${b.points[0]?.steps ?? '?'} steps) · ${b.reps} reps (+${b.warmup} warm-up), median · seed ${b.seed}${extra ? ' · ' + extra : ''}` : hw
