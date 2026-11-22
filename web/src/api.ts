import type { BandsResp, CitiesResp, Comparison, Frame, RunMeta, ScenarioSpec } from './types'

async function get<T>(url: string): Promise<T> {
  const r = await fetch(url)
  if (!r.ok) throw new Error(`${r.status} ${(await r.text()).slice(0, 200)}`)
  return r.json() as Promise<T>
}
export const api = {
  cities: () => get<CitiesResp>('/api/cities'),
  scenarios: () => get<{ scenarios: ScenarioSpec[] }>('/api/scenarios').then((r) => r.scenarios),
  comparison: (city: string) => get<Comparison>(`/api/cities/${city}/comparison`),
  bands: (city: string, sc: string) => get<BandsResp>(`/api/cities/${city}/scenarios/${sc}/bands`),
  frames: (city: string, sc: string) => get<{ meta: RunMeta; frames: Frame[] }>(`/api/cities/${city}/scenarios/${sc}/frames`),
}
