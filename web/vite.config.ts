import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Dev: run `epidemicsim serve` (port 8080) and proxy API + WebSocket to it.
export default defineConfig({
  plugins: [react()],
  server: {
    proxy: {
      '/api': 'http://127.0.0.1:8080',
      '/ws': { target: 'ws://127.0.0.1:8080', ws: true },
    },
  },
})
