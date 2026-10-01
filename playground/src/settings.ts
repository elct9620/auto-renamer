import type { Value } from './config'

export type SettingKind = 'path' | 'boolean' | 'pipelines' | 'vars' | 'unit'

/** Where a setting is written: a watch, the default every watch shares, or a folder configuration. */
export type Scope = 'watch' | 'default' | 'folder'

export interface Setting {
  key: string
  kind: SettingKind
  scopes: Scope[]
  required?: boolean
  example: Value
}

const SHARED: Scope[] = ['watch', 'default']

/** The keys the CLI reads for a watch, the default and a folder configuration, as the page's forms offer them.
 * The core does not describe them: they only feed the simulation, and the download is checked by the CLI's parser.
 * The `batch_*` keys wait for a redesign and are not offered; a configuration that holds them keeps them. */
export const SETTINGS: Setting[] = [
  { key: 'source', kind: 'path', scopes: ['watch'], required: true, example: '/downloads' },
  { key: 'target', kind: 'path', scopes: ['watch'], example: '/video' },
  { key: 'unit', kind: 'unit', scopes: SHARED, example: { root: ['*/*'] } },
  { key: 'pipelines', kind: 'pipelines', scopes: SHARED, example: ['video'] },
  { key: 'vars', kind: 'vars', scopes: [...SHARED, 'folder'], example: { show: 'Alpha' } },
  { key: 'dry_run', kind: 'boolean', scopes: SHARED, example: true },
]

export function settingsOf(scope: Scope): Setting[] {
  return SETTINGS.filter((setting) => setting.scopes.includes(scope))
}
