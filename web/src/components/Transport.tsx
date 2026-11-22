interface Props {
  step: number; maxStep: number; playing: boolean; speed: number; live: boolean; follow: boolean
  onSeek: (s: number) => void; onPlay: () => void; onSpeed: (v: number) => void; onLive: () => void; onStop: (() => void) | null
}
export const SPEEDS = [0.5, 1, 2, 5, 10, 20]

export function Transport(p: Props) {
  const day = Math.floor(p.step / 4)
  return (
    <div className="card transport fade-in">
      <button className="btn primary" onClick={p.onPlay} disabled={p.maxStep < 0} aria-label={p.playing ? 'Pause' : 'Play'}>{p.playing ? '⏸ Pause' : '▶ Play'}</button>
      <span className="day"><b>Day {day}</b></span>
      <input type="range" min={0} max={Math.max(p.maxStep, 0)} value={Math.min(p.step, Math.max(p.maxStep, 0))} onChange={(e) => p.onSeek(+e.target.value)} aria-label="Time" />
      <label className="row">Speed
        <select className="btn" value={p.speed} onChange={(e) => p.onSpeed(+e.target.value)}>{SPEEDS.map((s) => <option key={s} value={s}>{s} d/s</option>)}</select>
      </label>
      {p.live && !p.follow && <button className="btn" onClick={p.onLive}>Jump to live</button>}
      {p.onStop && <button className="btn danger" onClick={p.onStop}>Stop</button>}
    </div>
  )
}
