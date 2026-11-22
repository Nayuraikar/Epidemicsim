/** Colours are defined once in theme.css; read here (resolved) for SVG/canvas/deck use. */
function v(name: string, fallback: string): string {
  if (typeof document === 'undefined') return fallback
  return getComputedStyle(document.documentElement).getPropertyValue(name).trim() || fallback
}
export const C = {
  S: v('--s', '#2563eb'), E: v('--e', '#f59e0b'), I: v('--i', '#dc2626'), R: v('--r', '#16a34a'), D: v('--d', '#6b7280'),
  runs: [v('--run-1', '#334155'), v('--run-2', '#7c3aed'), v('--run-3', '#0891b2')],
  ink: v('--ink', '#111827'), muted: v('--muted', '#6b7280'), line: v('--line', '#e5e7eb'), bg: v('--bg', '#f6f7f9'),
}
export function rgb(hex: string): [number, number, number] {
  const n = parseInt(hex.slice(1), 16)
  return [(n >> 16) & 255, (n >> 8) & 255, n & 255]
}
