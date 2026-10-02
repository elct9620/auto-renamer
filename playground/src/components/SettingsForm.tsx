import { X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Choice } from '@/components/Choice'
import { Caption, TextField, ValueRows } from '@/components/Fields'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { BUILT_IN_PIPELINES, type Table, type Value, asTable, setIn, targetsOf } from '../config'
import { type Scope, type Setting, settingsOf } from '../settings'

function names(value: Value | undefined): string[] {
  return Array.isArray(value) ? value.filter((one): one is string => typeof one === 'string') : []
}

function routes(value: Value | undefined): Table[] {
  return Array.isArray(value) ? value.map((route) => asTable(route)) : []
}

/** Where a route moves, or its rejected route: a declared target, or nothing for in place. */
function MoveChoice({ label, value, targets, none, onChange }: {
  label: string
  value: Value | undefined
  targets: string[]
  none: string
  onChange: (target: string | undefined) => void
}) {
  return (
    <Choice label={label} value={typeof value === 'string' ? value : ''} clearable placeholder={none}
      options={targets.map((name) => ({ value: name, label: name }))}
      onChange={(name) => onChange(name === '' ? undefined : name)} />
  )
}

/** The routes in their order, each a pipeline picked from those defined and built in, the target it moves to,
 * whether it cleans up, and where its refused files go; a watch without its own routes follows `inherited`. */
function RoutesField({ value, inherited, defined, targets, onApply }: {
  value: Value | undefined
  inherited?: Table[]
  defined: string[]
  targets: string[]
  onApply: (value: Value | undefined) => void
}) {
  const { t } = useTranslation()
  const following = value === undefined && inherited !== undefined
  const listed = following ? inherited : routes(value)
  const change = (index: number, key: string, next: Value | undefined) =>
    onApply(listed.map((route, at) => (at === index ? setIn(route, [key], next) : route)))
  return (
    <div className="grid gap-2">
      {following && <span className="text-xs text-muted-foreground">{t('inspector.followsDefault')}</span>}
      {listed.map((route, index) => {
        const pipeline = typeof route.pipeline === 'string' ? route.pipeline : ''
        return (
          <div key={pipeline} className="grid gap-1.5 rounded-md border p-2">
            <div className="flex items-center justify-between gap-1.5">
              <Badge variant={following ? 'outline' : 'secondary'} className="font-mono">{pipeline}</Badge>
              <button aria-label={t('inspector.unlist', { name: pipeline })} onClick={() => onApply(listed.filter((_, at) => at !== index))}><X className="size-3" /></button>
            </div>
            <MoveChoice label={t('route.move')} value={route.move} targets={targets} none={t('route.inPlace')}
              onChange={(target) => change(index, 'move', target)} />
            <label className="flex items-center gap-2">
              <Switch checked={route.cleanup !== undefined} onCheckedChange={(checked) => change(index, 'cleanup', checked ? asTable(route.cleanup) : undefined)} />
              <Caption name={t('route.cleanup')} code="cleanup" />
            </label>
            <MoveChoice label={t('route.rejected')} value={asTable(route.rejected).move} targets={targets} none={t('route.noRejected')}
              onChange={(target) => change(index, 'rejected', target === undefined ? undefined : setIn(asTable(route.rejected), ['move'], target))} />
          </div>
        )
      })}
      <div className="flex gap-1.5">
        <Choice label={t('inspector.list')} value="" placeholder={t('inspector.list')}
          options={defined.filter((name) => !listed.some((route) => route.pipeline === name)).map((name) => ({ value: name, label: name }))}
          onChange={(name) => onApply([...listed, { pipeline: name }])} />
        {inherited !== undefined && !following && (
          <Button variant="ghost" size="sm" onClick={() => onApply(undefined)}>{t('inspector.useDefault')}</Button>
        )}
      </div>
    </div>
  )
}

function UnitField({ value, onApply }: { value: Value | undefined; onApply: (value: Value | undefined) => void }) {
  const { t } = useTranslation()
  const roots = typeof value === 'object' && !Array.isArray(value) ? asTable(value).root : undefined
  const chosen = typeof value === 'string' ? value : roots !== undefined ? 'root' : ''
  return (
    <div className="grid gap-1.5">
      <Choice label={t('settings.unit')} value={chosen} clearable placeholder="directory"
        options={['directory', 'source', 'root'].map((one) => ({ value: one, label: t(`unit.${one}`) }))}
        onChange={(one) => onApply(one === '' ? undefined : one === 'root' ? { root: names(roots) } : one)} />
      {chosen === 'root' && (
        <TextField caption={<Caption name={t('settings.root')} code="root" />} kind="texts" value={roots} onApply={(patterns) => onApply({ root: patterns ?? [] })} />
      )}
    </div>
  )
}

function Field({ setting, value, inherited, defined, targets, onApply }: {
  setting: Setting
  value: Value | undefined
  inherited?: Table[]
  defined: string[]
  targets: string[]
  onApply: (value: Value | undefined) => void
}) {
  const { t } = useTranslation()
  const caption = (
    <Caption name={t(`settings.${setting.key}`)} code={setting.key} required={setting.required} />
  )
  switch (setting.kind) {
    case 'path':
      return <TextField caption={caption} required={setting.required} kind="text" value={value} placeholder={String(setting.example)}
        onApply={onApply} />
    case 'boolean':
      return (
        <label className="flex items-center gap-2">
          <Switch checked={value === true} onCheckedChange={(checked) => onApply(checked || undefined)} />
          {caption}
        </label>
      )
    case 'text':
    case 'texts':
    case 'integer':
      return <TextField caption={caption} kind={setting.kind} value={value} placeholder={String(setting.example)} onApply={onApply} />
    case 'routes':
      return <div className="grid gap-1.5">{caption}<RoutesField value={value} inherited={inherited} defined={defined} targets={targets} onApply={onApply} /></div>
    case 'vars':
      return <div className="grid gap-1.5">{caption}<ValueRows values={asTable(value)} onChange={(values) => onApply(Object.keys(values).length ? values : undefined)} /></div>
    case 'unit':
      return <div className="grid gap-1.5">{caption}<UnitField value={value} onApply={onApply} /></div>
  }
}

/** The settings of a watch, the default or a folder configuration, each with the input its kind calls for. */
export function SettingsForm({ config, scope, path, onChange }: {
  config: Table
  scope: Scope
  path: string[]
  onChange: (config: Table) => void
}) {
  let table: Table = config
  for (const key of path) table = asTable(table[key])
  const inherited = scope === 'watch' ? routes(asTable(config.default).routes) : undefined
  const defined = [...new Set([...Object.keys(asTable(config.pipeline)), ...BUILT_IN_PIPELINES])]
  const targets = Object.keys(targetsOf(config))
  return (
    <>
      {settingsOf(scope).map((setting) => (
        <Field key={`${path.join('.')}-${setting.key}`} setting={setting} value={table[setting.key]} inherited={inherited} defined={defined}
          targets={targets} onApply={(value) => onChange(setIn(config, [...path, setting.key], value))} />
      ))}
    </>
  )
}

/** The targets the configuration declares, each a name and the root it stands for, with one more to add. */
export function TargetsForm({ config, onChange }: { config: Table; onChange: (config: Table) => void }) {
  const { t } = useTranslation()
  const targets = targetsOf(config)
  return (
    <div className="grid gap-2">
      <Caption name={t('settings.target')} code="target" />
      {Object.keys(targets).map((name) => (
        <div key={name} className="grid gap-1.5 rounded-md border p-2">
          <div className="flex items-center justify-between gap-1.5">
            <Badge variant="secondary" className="font-mono">{name}</Badge>
            <button aria-label={t('inspector.unlist', { name })} onClick={() => onChange(setIn(config, ['target', name], undefined))}><X className="size-3" /></button>
          </div>
          <SettingsForm config={config} scope="target" path={['target', name]} onChange={onChange} />
        </div>
      ))}
      <TextField caption={<Caption name={t('route.newTarget')} code="target" />} kind="text" value={undefined} placeholder="video"
        onApply={(name) => typeof name === 'string' && name.trim() !== '' && onChange(setIn(config, ['target', name.trim(), 'path'], '/video'))} />
    </div>
  )
}
