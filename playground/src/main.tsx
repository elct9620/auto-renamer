import { StrictMode } from 'react'
import { createRoot } from 'react-dom/client'

import App from './App.tsx'
import { TooltipProvider } from './components/ui/tooltip'
import { init } from './core'
import './index.css'
import { startI18n } from './i18n'

// shadcn/ui themes by a `dark` class, so the page follows the system's scheme through it.
const dark = window.matchMedia('(prefers-color-scheme: dark)')
const followScheme = () => document.documentElement.classList.toggle('dark', dark.matches)
followScheme()
dark.addEventListener('change', followScheme)

startI18n(navigator.languages)

// The core answers synchronously once its module is loaded, so the page waits for it before drawing.
await init()

createRoot(document.getElementById('root')!).render(
  <StrictMode>
    <TooltipProvider>
      <App />
    </TooltipProvider>
  </StrictMode>,
)
