import type { Value } from './config'

export type SettingKind = 'path' | 'boolean' | 'routes' | 'vars' | 'unit' | 'text' | 'texts' | 'integer'

/** Where a setting is written: a watch, the default every watch shares, a folder configuration, or a target. */
export type Scope = 'watch' | 'default' | 'folder' | 'target'

export interface Setting {
  key: string
  kind: SettingKind
  scopes: Scope[]
  required?: boolean
  example: Value
}

const SHARED: Scope[] = ['watch', 'default']

/** The keys the CLI reads for a watch, the default, a folder configuration and a target, as the page's forms
 * offer them. The core does not describe them: they only feed the simulation, and the download is checked by
 * the CLI's parser. */
export const SETTINGS: Setting[] = [
  { key: 'source', kind: 'path', scopes: ['watch'], required: true, example: '/downloads' },
  { key: 'unit', kind: 'unit', scopes: SHARED, example: { root: ['*/*'] } },
  { key: 'routes', kind: 'routes', scopes: SHARED, example: [{ pipeline: 'video' }] },
  { key: 'group', kind: 'texts', scopes: SHARED, example: ['show', 'episode'] },
  { key: 'vars', kind: 'vars', scopes: [...SHARED, 'folder'], example: { show: 'Alpha' } },
  { key: 'quiet', kind: 'text', scopes: SHARED, example: '5m' },
  { key: 'max_wait', kind: 'text', scopes: SHARED, example: '30m' },
  { key: 'max_files', kind: 'integer', scopes: [...SHARED, 'folder'], example: 1000 },
  { key: 'dry_run', kind: 'boolean', scopes: SHARED, example: true },
  { key: 'path', kind: 'path', scopes: ['target'], required: true, example: '/video' },
]

export function settingsOf(scope: Scope): Setting[] {
  return SETTINGS.filter((setting) => setting.scopes.includes(scope))
}
