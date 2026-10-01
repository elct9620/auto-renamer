import type { Value } from './config'

export type SettingKind = 'path' | 'duration' | 'count' | 'boolean' | 'pipelines' | 'vars' | 'unit'

/** Where a setting is written: a watch, the default every watch shares, or a folder configuration. */
export type Scope = 'watch' | 'default' | 'folder'

export interface Setting {
  key: string
  kind: SettingKind
  scopes: Scope[]
  required?: boolean
  /** Whether the simulation acts on it; the rest only take effect in the CLI. */
  simulated: boolean
  example: Value
}

const SHARED: Scope[] = ['watch', 'default']

/** The keys the CLI reads for a watch, the default and a folder configuration, as the page's forms offer them.
 * The core does not describe them: they only feed the simulation, and the download is checked by the CLI's parser. */
export const SETTINGS: Setting[] = [
  { key: 'source', kind: 'path', scopes: ['watch'], required: true, simulated: true, example: '/downloads' },
  { key: 'target', kind: 'path', scopes: ['watch'], simulated: true, example: '/video' },
  { key: 'unit', kind: 'unit', scopes: SHARED, simulated: true, example: { root: ['*/*'] } },
  { key: 'pipelines', kind: 'pipelines', scopes: SHARED, simulated: true, example: ['video'] },
  { key: 'vars', kind: 'vars', scopes: [...SHARED, 'folder'], simulated: true, example: { show: 'Alpha' } },
  { key: 'dry_run', kind: 'boolean', scopes: SHARED, simulated: true, example: true },
  { key: 'batch_max', kind: 'count', scopes: [...SHARED, 'folder'], simulated: true, example: 1000 },
  { key: 'batch_window', kind: 'duration', scopes: SHARED, simulated: false, example: '5m' },
  { key: 'batch_max_wait', kind: 'duration', scopes: SHARED, simulated: false, example: '1h' },
]

export function settingsOf(scope: Scope): Setting[] {
  return SETTINGS.filter((setting) => setting.scopes.includes(scope))
}
