// The page's own texts in each language it speaks. Configuration keys, stage and parameter names, and
// what the core reports stay as the CLI writes them.

import i18n from 'i18next'
import { initReactI18next } from 'react-i18next'

import en from './locales/en.json'
import zhTW from './locales/zh-TW.json'

export const LANGUAGES = { en: 'English', 'zh-TW': '繁體中文' } as const
export type Language = keyof typeof LANGUAGES

export const texts: Record<Language, typeof en> = { en, 'zh-TW': zhTW }

/** The language for a browser's preferences: Traditional Chinese for any Chinese, English otherwise. */
export function languageOf(preferred: readonly string[]): Language {
  return preferred[0]?.toLowerCase().startsWith('zh') ? 'zh-TW' : 'en'
}

export function startI18n(preferred: readonly string[]): void {
  i18n.on('languageChanged', (language) => {
    document.documentElement.lang = language
  })
  i18n.use(initReactI18next).init({
    resources: Object.fromEntries(Object.entries(texts).map(([language, translation]) => [language, { translation }])),
    lng: languageOf(preferred),
    fallbackLng: 'en',
    interpolation: { escapeValue: false },
  })
}
