import { createContext, useCallback, useContext, useEffect, useId, useLayoutEffect, useRef, useState } from 'react'
import { createPortal } from 'react-dom'

export interface HelpText { what: string; effect: string; try: string }

/** When true, every <Help> shows its text inline instead of behind the "?" (Studio "Guide" toggle). */
export const GuideContext = createContext(false)

const GAP = 6
const MARGIN = 8

function Body({ h }: { h: HelpText }) {
  return (
    <>
      <span className="help-line"><i>WHAT</i>{h.what}</span>
      <span className="help-line"><i>EFFECT</i>{h.effect}</span>
      <span className="help-line"><i>TRY</i>{h.try}</span>
    </>
  )
}

/**
 * "?" button placed after a label. The panel opens on hover, keyboard focus and tap; it closes on
 * Escape, outside click or leaving. It is portalled to <body> with fixed positioning, flipped above
 * the button near the bottom edge and shifted inside the viewport, so no scroll container clips it.
 * `once`: in Guide mode keep this one as a tooltip (used for repeated copies such as per-run cards).
 */
export function Help({ h, once = false }: { h: HelpText; once?: boolean }) {
  const guide = useContext(GuideContext) && !once
  const id = useId()
  const btn = useRef<HTMLButtonElement>(null)
  const pop = useRef<HTMLSpanElement>(null)
  const pinned = useRef(false)
  const timer = useRef<number | undefined>(undefined)
  const [open, setOpen] = useState(false)
  const [pos, setPos] = useState<{ left: number; top: number } | null>(null)

  const close = useCallback(() => { pinned.current = false; window.clearTimeout(timer.current); setOpen(false) }, [])
  const show = () => { window.clearTimeout(timer.current); setOpen(true) }
  const leave = () => { window.clearTimeout(timer.current); timer.current = window.setTimeout(() => { if (!pinned.current) setOpen(false) }, 120) }

  const place = useCallback(() => {
    const b = btn.current, p = pop.current
    if (!b || !p) return
    const r = b.getBoundingClientRect()
    const vw = document.documentElement.clientWidth, vh = window.innerHeight
    const pw = p.offsetWidth, ph = p.offsetHeight
    const left = Math.min(Math.max(r.left + r.width / 2 - pw / 2, MARGIN), Math.max(MARGIN, vw - pw - MARGIN))
    let top = r.bottom + GAP
    if (top + ph > vh - MARGIN) top = r.top - ph - GAP
    if (top < MARGIN) top = Math.max(MARGIN, vh - ph - MARGIN)
    setPos((o) => (o && o.left === left && o.top === top ? o : { left, top }))
  }, [])

  useLayoutEffect(() => { if (open) place(); else setPos(null) }, [open, place])

  useEffect(() => {
    if (!open) return
    const onKey = (e: KeyboardEvent) => { if (e.key === 'Escape') close() }
    const onDown = (e: Event) => { const t = e.target as Node; if (!btn.current?.contains(t) && !pop.current?.contains(t)) close() }
    document.addEventListener('keydown', onKey)
    document.addEventListener('pointerdown', onDown)
    window.addEventListener('resize', place)
    window.addEventListener('scroll', place, true)
    return () => {
      document.removeEventListener('keydown', onKey); document.removeEventListener('pointerdown', onDown)
      window.removeEventListener('resize', place); window.removeEventListener('scroll', place, true)
    }
  }, [open, close, place])

  useEffect(() => () => window.clearTimeout(timer.current), [])

  return (
    <>
      <button ref={btn} type="button" className="help-btn" aria-label="Help" aria-expanded={open && !guide}
        aria-describedby={guide || open ? id : undefined}
        onMouseEnter={show} onMouseLeave={leave} onFocus={show}
        onBlur={(e) => { if (!pinned.current && !pop.current?.contains(e.relatedTarget as Node)) setOpen(false) }}
        onClick={() => { if (pinned.current) close(); else { pinned.current = true; setOpen(true) } }}>?</button>
      {guide
        ? <span id={id} className="help-inline"><Body h={h} /></span>
        : open && createPortal(
          <span ref={pop} id={id} role="tooltip" className="help-pop" onMouseEnter={show} onMouseLeave={leave}
            style={{ left: pos?.left ?? 0, top: pos?.top ?? 0, visibility: pos ? 'visible' : 'hidden' }}>
            <Body h={h} />
          </span>, document.body)}
    </>
  )
}
