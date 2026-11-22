import type { ScenarioSpec } from './types'
import { C } from './colors'

function download(name: string, blob: Blob) {
  const a = document.createElement('a')
  a.href = URL.createObjectURL(blob)
  a.download = name
  a.click()
  setTimeout(() => URL.revokeObjectURL(a.href), 1000)
}

export function exportScenarioJson(sc: ScenarioSpec) {
  download(`${sc.name || 'scenario'}.json`, new Blob([JSON.stringify(sc, null, 2) + '\n'], { type: 'application/json' }))
}

const loadImg = (src: string) => new Promise<HTMLImageElement>((res, rej) => { const i = new Image(); i.onload = () => res(i); i.onerror = rej; i.src = src })

/** Snapshot one maplibre map canvas (grabbed inside a render so the drawing buffer is valid). */
export function mapSnapshot(map: { once: (e: 'render', f: () => void) => void; triggerRepaint: () => void; getCanvas: () => HTMLCanvasElement }): Promise<HTMLCanvasElement> {
  return new Promise((res) => {
    map.once('render', () => {
      const src = map.getCanvas()
      const c = document.createElement('canvas')
      c.width = src.width; c.height = src.height
      c.getContext('2d')!.drawImage(src, 0, 0)
      res(c)
    })
    map.triggerRepaint()
  })
}

/** Compose title + maps + chart SVGs of the comparison view into one PNG (no extra libraries). */
export async function exportComparisonPng(root: HTMLElement, maps: HTMLCanvasElement[], title: string) {
  const scale = 2
  const rootBox = root.getBoundingClientRect()
  const W = Math.round(rootBox.width), pad = 16, head = 44
  const H = Math.round(rootBox.height) + head
  const cv = document.createElement('canvas')
  cv.width = (W + pad * 2) * scale; cv.height = (H + pad * 2) * scale
  const ctx = cv.getContext('2d')!
  ctx.scale(scale, scale)
  ctx.fillStyle = C.bg; ctx.fillRect(0, 0, W + pad * 2, H + pad * 2)
  ctx.fillStyle = C.ink; ctx.font = '600 18px system-ui, sans-serif'; ctx.fillText(title, pad, pad + 22)
  const ox = pad - rootBox.left, oy = pad + head - rootBox.top
  // card backgrounds
  root.querySelectorAll<HTMLElement>('.card').forEach((el) => {
    const b = el.getBoundingClientRect()
    ctx.fillStyle = '#fff'; ctx.beginPath(); ctx.roundRect(b.left + ox, b.top + oy, b.width, b.height, 14); ctx.fill()
  })
  // all HTML text (metric cards, counts, legends, headers); SVG text is inside the serialised charts
  ctx.textBaseline = 'top'
  const walker = document.createTreeWalker(root, NodeFilter.SHOW_TEXT)
  for (let n = walker.nextNode(); n; n = walker.nextNode()) {
    const el = n.parentElement
    const t = (n.textContent ?? '').replace(/\s+/g, ' ').trim()
    if (!el || !t || el.closest('svg, select, option, button, .overlay')) continue
    const r = document.createRange()
    r.selectNodeContents(n)
    const b = r.getBoundingClientRect()
    if (!b.width) continue
    const cs = getComputedStyle(el)
    ctx.fillStyle = cs.color; ctx.font = `${cs.fontWeight} ${cs.fontSize} system-ui, sans-serif`
    ctx.fillText(t, b.left + ox, b.top + oy + 1)
  }
  // maps
  const mapEls = root.querySelectorAll<HTMLElement>('[data-export-map]')
  mapEls.forEach((el, i) => { const b = el.getBoundingClientRect(); if (maps[i]) ctx.drawImage(maps[i], b.left + ox, b.top + oy, b.width, b.height) })
  // charts (SVG with resolved colours)
  for (const svg of Array.from(root.querySelectorAll<SVGSVGElement>('svg.recharts-surface'))) {
    const b = svg.getBoundingClientRect()
    const clone = svg.cloneNode(true) as SVGSVGElement
    clone.setAttribute('xmlns', 'http://www.w3.org/2000/svg')
    clone.setAttribute('width', String(b.width)); clone.setAttribute('height', String(b.height))
    clone.querySelectorAll('text').forEach((t) => { t.setAttribute('font-family', 'system-ui, sans-serif'); t.setAttribute('font-size', '11'); if (!t.getAttribute('fill')) t.setAttribute('fill', C.muted) })
    const img = await loadImg('data:image/svg+xml;charset=utf-8,' + encodeURIComponent(new XMLSerializer().serializeToString(clone)))
    ctx.drawImage(img, b.left + ox, b.top + oy, b.width, b.height)
  }
  cv.toBlob((blob) => blob && download('comparison.png', blob), 'image/png')
}
