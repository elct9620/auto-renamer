import { readFileSync } from 'node:fs'

import { initSync } from './core'

// Tests run the real core, loaded from the module `pnpm wasm` built.
initSync({ module: readFileSync(new URL('./wasm/auto_renamer_wasm_bg.wasm', import.meta.url)) })
