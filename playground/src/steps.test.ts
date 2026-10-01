import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { simulate } from './core'
import { afterStage, changes } from './steps'

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

describe('the files after a stage', () => {
  // @behavior PGE-067
  it('lists the files it ran on, apart from those a folder configuration took', () => {
    const file = (path: string, text = '') => ({ path, folder: false, modified: 0, text })
    const { outcomes } = simulate(`${CONFIG}unit = { root = ["*"] }\n`, 'series', [
      file('/downloads/A/Show 07.mkv'),
      file('/downloads/B/auto-renamer.toml', '[pipeline.video]\nstages = [{ number = { into = "episode" } }, { format = "Other {episode}" }]\n'),
      file('/downloads/B/Show 08.mkv'),
    ])

    const { ran, taken } = afterStage(outcomes, 'video', 1, null)

    expect(ran.map(({ origin, changes }) => [origin, changes])).toEqual([['A/Show 07.mkv', [{ key: 'name', from: 'Show 07', to: 'Show 7' }]]])
    expect(taken).toEqual(['B/Show 08.mkv'])
    expect(afterStage(outcomes, 'video', 1, 'B').ran.map(({ origin }) => origin)).toEqual(['B/Show 08.mkv'])
  })
})
