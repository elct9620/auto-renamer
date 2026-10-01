import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { setIn } from './config'
import { check, read, render } from './core'
import { type Scope, settingsOf } from './settings'

const GLOBAL = `
[pipeline.video]
stages = ["move"]

[watch.series]
source = "/downloads"
`

const FOLDER = `
[pipeline.video]
stages = ["move"]
`

const PLACES: Record<Scope, { kind: 'global' | 'folder'; text: string; path: string[] }> = {
  watch: { kind: 'global', text: GLOBAL, path: ['watch', 'series'] },
  default: { kind: 'global', text: GLOBAL, path: ['default'] },
  folder: { kind: 'folder', text: FOLDER, path: [] },
}

describe('settings', () => {
  // @behavior PGE-029
  it.each(Object.keys(PLACES) as Scope[])('offers for a %s only settings the CLI reads', (scope) => {
    const { kind, text, path } = PLACES[scope]

    for (const setting of settingsOf(scope)) {
      const config = setIn(read(text), [...path, setting.key], setting.example)
      expect(() => check(kind, render(config)), setting.key).not.toThrow()
    }
  })
})
