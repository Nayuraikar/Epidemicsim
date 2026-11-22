const LINKS = [{ id: 'home', label: 'Home', href: '#/' }, { id: 'studio', label: 'Studio', href: '#/studio' }, { id: 'lab', label: 'Lab', href: '#/lab' }]
/** Top navigation; the current page is highlighted. `demo` counts as Studio. */
export default function Nav({ active }: { active: 'home' | 'studio' | 'lab' }) {
  return (
    <nav className="nav" aria-label="Pages">
      {LINKS.map((l) => <a key={l.id} href={l.href} className={l.id === active ? 'on' : ''} aria-current={l.id === active ? 'page' : undefined}>{l.label}</a>)}
    </nav>
  )
}
