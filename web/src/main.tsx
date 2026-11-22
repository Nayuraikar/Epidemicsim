import { StrictMode, useEffect, useState } from 'react'
import { createRoot } from 'react-dom/client'
import './index.css'
import App from './App.tsx'
import Lab from './lab/Lab.tsx'
import Landing from './Landing.tsx'

const route = () => (location.hash.replace(/^#\/?/, '').split('?')[0] || 'home')
function Root() {
  const [r, setR] = useState(route())
  useEffect(() => { const h = () => setR(route()); window.addEventListener('hashchange', h); return () => window.removeEventListener('hashchange', h) }, [])
  if (r === 'lab') return <Lab />
  if (r === 'studio') return <App key="studio" />
  if (r === 'demo') return <App key="demo" demo />
  return <Landing />
}
createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <Root />
  </StrictMode>,
)
