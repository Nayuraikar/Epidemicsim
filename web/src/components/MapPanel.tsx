import { useEffect, useRef } from 'react'
import * as maplibregl from 'maplibre-gl'
import 'maplibre-gl/dist/maplibre-gl.css'
import workerUrl from 'maplibre-gl/dist/maplibre-gl-worker.mjs?url'
import { MapboxOverlay } from '@deck.gl/mapbox'
import { PolygonLayer, ScatterplotLayer } from '@deck.gl/layers'
import type { Run } from '../run'
import type { Frame, Grid } from '../types'
import { C, rgb } from '../colors'
import { mapSnapshot } from '../export'

const STYLE = 'https://basemaps.cartocdn.com/gl/positron-gl-style/style.json' // Carto Positron
// Vite rewrites the bundle location, so point MapLibre at the emitted worker explicitly.
maplibregl.setWorkerUrl(workerUrl)
const maps = new WeakMap<HTMLElement, maplibregl.Map>()
const all = new Set<maplibregl.Map>()
let syncing = false

export async function snapshotMaps(root: HTMLElement): Promise<HTMLCanvasElement[]> {
  const out: HTMLCanvasElement[] = []
  for (const el of Array.from(root.querySelectorAll<HTMLElement>('[data-export-map]'))) {
    const m = maps.get(el)
    const base = m ? await mapSnapshot(m) : document.createElement('canvas')
    const deck = el.querySelector<HTMLCanvasElement>('canvas:not(.maplibregl-canvas)')
    if (deck && base.width) base.getContext('2d')!.drawImage(deck, 0, 0, base.width, base.height)
    out.push(base)
  }
  return out
}

interface Cell { n: number; poly: [number, number][] }
function cellsOf(grid: Grid, f: Frame | undefined, cache: Map<number, [number, number][]>): Cell[] {
  if (!f) return []
  const [x0, y0, x1, y1] = grid.bbox
  const dx = (x1 - x0) / grid.cols, dy = (y1 - y0) / grid.rows
  const out: Cell[] = []
  for (let i = 0; i < f.density.length; i += 2) {
    const c = f.density[i]
    let poly = cache.get(c)
    if (!poly) {
      const cx = x0 + (c % grid.cols) * dx, cy = y1 - (Math.floor(c / grid.cols) + 1) * dy
      poly = [[cx, cy], [cx + dx, cy], [cx + dx, cy + dy], [cx, cy + dy]]
      cache.set(c, poly)
    }
    out.push({ n: f.density[i + 1], poly })
  }
  return out
}

const AMBER = rgb(C.E), RED = rgb(C.I)
function heat(t: number): [number, number, number, number] {
  const k = Math.pow(Math.min(1, t), 0.6)
  return [AMBER[0] + (RED[0] - AMBER[0]) * k, AMBER[1] + (RED[1] - AMBER[1]) * k, AMBER[2] + (RED[2] - AMBER[2]) * k, 40 + 190 * k]
}

interface Props {
  run: Run | null
  frame: Frame | undefined
  bbox: [number, number, number, number] | null
  onPick: (run: Run, cell: number) => void
}

export function MapPanel({ run, frame, bbox, onPick }: Props) {
  const box = useRef<HTMLDivElement>(null)
  const map = useRef<maplibregl.Map | null>(null)
  const overlay = useRef<MapboxOverlay | null>(null)
  const cache = useRef(new Map<number, [number, number][]>())
  const pick = useRef({ run, onPick })
  pick.current = { run, onPick }

  useEffect(() => {
    const el = box.current!
    const m = new maplibregl.Map({ container: el, style: STYLE, center: [78, 20], zoom: 4, attributionControl: { compact: true } })
    const o = new MapboxOverlay({ interleaved: false, layers: [], glOptions: { preserveDrawingBuffer: true } } as ConstructorParameters<typeof MapboxOverlay>[0])
    m.addControl(o as unknown as maplibregl.IControl)
    m.addControl(new maplibregl.NavigationControl({ showCompass: false }), 'top-right')
    maps.set(el, m); all.add(m); map.current = m; overlay.current = o
    m.on('move', (e) => {
      if (syncing || !(e as { originalEvent?: unknown }).originalEvent) return
      syncing = true
      const cam = { center: m.getCenter(), zoom: m.getZoom(), bearing: m.getBearing(), pitch: m.getPitch() }
      all.forEach((other) => { if (other !== m) other.jumpTo(cam) })
      syncing = false
    })
    m.on('click', (e) => {
      const { run: r, onPick: cb } = pick.current
      const g = r?.meta?.grid
      if (!r || !g) return
      const [x0, y0, x1, y1] = g.bbox
      const cx = Math.floor(((e.lngLat.lng - x0) / (x1 - x0)) * g.cols), cy = Math.floor(((y1 - e.lngLat.lat) / (y1 - y0)) * g.rows)
      if (cx >= 0 && cy >= 0 && cx < g.cols && cy < g.rows) cb(r, cy * g.cols + cx)
    })
    return () => { all.delete(m); m.remove(); map.current = null }
  }, [])

  // Fly to the city whenever it (or its bbox) changes.
  useEffect(() => {
    const m = map.current
    if (!m || !bbox) return
    const cam = m.cameraForBounds([[bbox[0], bbox[1]], [bbox[2], bbox[3]]], { padding: 24 })
    if (cam) m.flyTo({ ...cam, duration: 1800, essential: true })
  }, [bbox?.join(',')])

  useEffect(() => {
    const g = run?.meta?.grid
    const layers = []
    if (g && frame) {
      const max = Math.max(run!.maxCell, 1)
      layers.push(new PolygonLayer<Cell>({
        id: 'cells', data: cellsOf(g, frame, cache.current), getPolygon: (d) => d.poly,
        getFillColor: (d) => heat(d.n / max), stroked: false, filled: true, pickable: false,
        updateTriggers: { getFillColor: [frame.step, max] },
      }))
    }
    const ins = run?.inspected
    if (ins) {
      const pts = [
        { p: ins.home, c: rgb(C.S), r: 9 },
        ...(ins.work ? [{ p: ins.work, c: rgb(C.runs[0]), r: 9 }] : []),
        ...ins.chain.map((h) => ({ p: h.place, c: rgb(C.I), r: 11 })),
      ]
      layers.push(new ScatterplotLayer({
        id: 'agent', data: pts, getPosition: (d) => [d.p.lon, d.p.lat], getFillColor: (d) => [...d.c, 220] as [number, number, number, number],
        getRadius: (d) => d.r, radiusUnits: 'pixels', stroked: true, getLineColor: [255, 255, 255, 255], lineWidthMinPixels: 2,
      }))
    }
    overlay.current?.setProps({ layers })
  }, [run, frame, run?.inspected])

  return <div ref={box} className="map" data-export-map />
}
