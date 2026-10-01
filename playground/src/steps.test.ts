import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { simulate } from './core'
import { changes } from './steps'

const CONFIG = `[pipeline.video]
stages = [{ number = { into = "episode" } }, { format = "Show {episode}" }]

[watch.series]
source = "/downloads"
pipelines = ["video"]
`

describe('a timeline', () => {
  // @behavior PGE-065
  it('marks only what each step changed', () => {
    const { outcomes } = simulate(CONFIG, 'series', [{ path: '/downloads/Show 07.mkv', folder: false, modified: 0, text: '' }])
    const [claim, number, format] = outcomes[0].steps

    expect(changes(undefined, claim)).toEqual([])
    expect(changes(claim, number)).toEqual([{ key: 'episode', to: '7' }])
    expect(changes(number, format)).toEqual([{ key: 'name', from: 'Show 07', to: 'Show 7' }])
  })
})
