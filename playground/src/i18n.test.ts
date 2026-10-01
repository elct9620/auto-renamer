import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import { stages } from './core'
import { type Language, languageOf, texts } from './i18n'

function keys(table: object, prefix = ''): string[] {
  return Object.entries(table).flatMap(([key, value]) =>
    typeof value === 'object' && value !== null ? keys(value, `${prefix}${key}.`) : [`${prefix}${key}`])
}

describe('languages', () => {
  // @behavior PGE-024
  it('speaks Traditional Chinese to a browser preferring Chinese', () => {
    expect(languageOf(['zh-CN', 'en'])).toBe('zh-TW')
  })

  // @behavior PGE-025
  it('speaks English to any other browser', () => {
    expect(languageOf(['ja', 'fr'])).toBe('en')
  })

  // @behavior PGE-026
  it('writes every text in both languages', () => {
    expect(keys(texts['zh-TW']).sort()).toEqual(keys(texts.en).sort())
  })

  // @behavior PGE-030
  it.each(['zh-TW', 'en'] as Language[])('describes in %s every stage, parameter and single value the core declares', (language) => {
    const described = new Set(keys(texts[language]))
    const expected = stages().flatMap((stage) => [
      `stages.${stage.name}.about`,
      ...stage.parameters.map((parameter) => `stages.${stage.name}.parameters.${parameter.name}`),
      ...(stage.value ? [`stages.${stage.name}.value`] : []),
    ])

    expect(expected.filter((key) => !described.has(key))).toEqual([])
  })
})
