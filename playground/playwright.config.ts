import { defineConfig } from '@playwright/test'

// The built page, served as GitHub Pages would serve it, in the Google Chrome the machine already has.
export default defineConfig({
  testDir: 'e2e',
  testMatch: '*.e2e.ts',
  forbidOnly: !!process.env.CI,
  use: {
    baseURL: 'http://localhost:4173',
    channel: 'chrome',
    locale: 'en-US',
    viewport: { width: 1400, height: 1000 },
  },
  webServer: {
    command: 'pnpm preview --port 4173 --strictPort',
    url: 'http://localhost:4173',
    reuseExistingServer: !process.env.CI,
  },
})
