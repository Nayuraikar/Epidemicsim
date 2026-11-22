import type { LiveConn } from './live'
import type { BandsResp, Dist, Frame, Inspected, RunMeta, ScenarioSpec } from './types'

export interface Run {
  id: number
  city: string
  label: string
  mode: 'replay' | 'live'
  scenario: ScenarioSpec
  /** Preset name when this is an unmodified preset (so ensemble results exist for it). */
  preset: string | null
  status: 'loading' | 'running' | 'paused' | 'done' | 'error'
  message: string
  meta: RunMeta | null
  /** Mutated in place while a live run streams; App re-renders via a version counter. */
  frames: Frame[]
  maxCell: number
  bands: BandsResp | null
  ensemble: Record<string, Dist> | null
  ensembleRuns: number
  conn: LiveConn | null
  inspected: Inspected | null
  inspectNote: string
}

export function pushFrame(r: Run, f: Frame) {
  r.frames.push(f)
  for (let i = 1; i < f.density.length; i += 2) if (f.density[i] > r.maxCell) r.maxCell = f.density[i]
}
