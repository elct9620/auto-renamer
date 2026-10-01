import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import App from './App.tsx'
import { init } from './core'
import './index.css'
import './styles.css'

// The core answers synchronously once its module is loaded, so the page waits for it before drawing.
await init()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <App />
  </StrictMode>,
)
