import { describe, expect, it } from 'vitest'

import { languageOf, texts } from './i18n'

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
})
