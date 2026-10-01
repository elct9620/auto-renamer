import react from '@vitejs/plugin-react'
import { defineConfig } from 'vite'

// Served from a project page on GitHub Pages, so every asset is referenced relative to the page.
export default defineConfig({
  base: './',
  plugins: [react()],
})
