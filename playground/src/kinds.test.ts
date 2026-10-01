import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { KIND_ICONS, stageIcon } from './components/kinds'
import { stages } from './core'

describe('icons', () => {
  // @behavior PGE-047
  it('gives every stage the core declares an icon', () => {
    expect(stages().filter(({ name }) => stageIcon(name) === KIND_ICONS.stage).map(({ name }) => name)).toEqual([])
  })
})
