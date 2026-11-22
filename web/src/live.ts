import type { Frame, Inspected, RunMeta, ScenarioSpec } from './types'

export interface StartOpts { city: string; scenario: ScenarioSpec; agents: number; days: number; seed: number; backend: string; chains: boolean; speed: number }
export interface Handlers {
  onStatus: (m: string) => void
  onReady: (m: RunMeta) => void
  onFrame: (f: Frame) => void
  onAgent: (a: Inspected) => void
  onDone: (stopped: boolean) => void
  onError: (m: string) => void
}

/** One live run over /ws/live. */
export class LiveConn {
  private ws: WebSocket
  private finished = false
  constructor(opts: StartOpts, h: Handlers) {
    const proto = location.protocol === 'https:' ? 'wss' : 'ws'
    this.ws = new WebSocket(`${proto}://${location.host}/ws/live`)
    this.ws.onopen = () => this.ws.send(JSON.stringify({ type: 'start', ...opts }))
    this.ws.onerror = () => { if (!this.finished) { this.finished = true; h.onError('Cannot reach the live engine (is `epidemicsim serve` running?)') } }
    this.ws.onclose = () => { if (!this.finished) { this.finished = true; h.onError('Live connection closed') } }
    this.ws.onmessage = (ev) => {
      const o = JSON.parse(ev.data as string)
      switch (o.type) {
        case 'status': h.onStatus(o.message); break
        case 'ready': h.onReady(o as RunMeta); break
        case 'frame': h.onFrame(o as Frame); break
        case 'agent': h.onAgent(o.agent as Inspected); break
        case 'done': this.finished = true; h.onDone(o.stopped); break
        case 'error': h.onError(o.message); break
      }
    }
  }
  private send(o: object) { if (this.ws.readyState === WebSocket.OPEN) this.ws.send(JSON.stringify(o)) }
  pause() { this.send({ type: 'pause' }) }
  resume() { this.send({ type: 'resume' }) }
  stop() { this.send({ type: 'stop' }) }
  speed(stepsPerSec: number) { this.send({ type: 'speed', steps_per_sec: stepsPerSec }) }
  inspect(cell: number) { this.send({ type: 'inspect', cell }) }
  close() { this.finished = true; this.ws.close() }
}
