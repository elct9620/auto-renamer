import { X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Choice } from '@/components/Choice'
import { TextField, ValueRows } from '@/components/Fields'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Switch } from '@/components/ui/switch'
import { type Table, type Value, asTable, setIn } from '../config'
import { type Scope, type Setting, settingsOf } from '../settings'

function names(value: Value | undefined): string[] {
  return Array.isArray(value) ? value.filter((one): one is string => typeof one === 'string') : []
}

/** An ordered list of pipelines picked from those defined; a watch without its own list follows `inherited`. */
function PipelinesField({ value, inherited, defined, onApply }: {
  value: Value | undefined
  inherited?: string[]
  defined: string[]
  onApply: (value: Value | undefined) => void
}) {
  const { t } = useTranslation()
  const following = value === undefined && inherited !== undefined
  const listed = following ? inherited : names(value)
  const taken = new Set(listed)
  return (
    <div className="grid gap-1.5">
      <div className="flex flex-wrap gap-1">
        {listed.map((name) => (
          <Badge key={name} variant={following ? 'outline' : 'secondary'} className="font-mono">
            {name}
            <button aria-label={t('inspector.unlist', { name })} onClick={() => onApply(listed.filter((one) => one !== name))}><X className="size-3" /></button>
          </Badge>
        ))}
        {following && <span className="text-xs text-muted-foreground">{t('inspector.followsDefault')}</span>}
      </div>
      <div className="flex gap-1.5">
        <Choice label={t('inspector.list')} value="" placeholder={t('inspector.list')}
          options={defined.filter((name) => !taken.has(name)).map((name) => ({ value: name, label: name }))}
          onChange={(name) => onApply([...listed, name])} />
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
      <Choice label="unit" value={chosen} clearable placeholder="directory"
        options={['directory', 'source', 'root'].map((one) => ({ value: one, label: t(`unit.${one}`) }))}
        onChange={(one) => onApply(one === '' ? undefined : one === 'root' ? { root: names(roots) } : one)} />
      {chosen === 'root' && (
        <TextField label="root" kind="texts" value={roots} onApply={(patterns) => onApply({ root: patterns ?? [] })} />
      )}
    </div>
  )
}

function Field({ setting, value, inherited, defined, onApply }: {
  setting: Setting
  value: Value | undefined
  inherited?: string[]
  defined: string[]
  onApply: (value: Value | undefined) => void
}) {
  const { t } = useTranslation()
  const caption = (
    <span className="flex items-center gap-1.5 font-mono text-xs font-medium">
      {setting.key}{setting.required && <span className="text-destructive">*</span>}
      {!setting.simulated && <Badge variant="outline">{t('inspector.notSimulated')}</Badge>}
    </span>
  )
  switch (setting.kind) {
    case 'path':
    case 'duration':
      return <TextField label={setting.key} required={setting.required} kind="text" value={value} placeholder={String(setting.example)}
        caption={caption} onApply={onApply} />
    case 'count':
      return <TextField label={setting.key} kind="integer" value={value} placeholder={String(setting.example)} caption={caption} onApply={onApply} />
    case 'boolean':
      return (
        <label className="flex items-center gap-2">
          <Switch checked={value === true} onCheckedChange={(checked) => onApply(checked || undefined)} />
          {caption}
        </label>
      )
    case 'pipelines':
      return <div className="grid gap-1.5">{caption}<PipelinesField value={value} inherited={inherited} defined={defined} onApply={onApply} /></div>
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
  const inherited = scope === 'watch' ? names(asTable(config.default).pipelines) : undefined
  const defined = Object.keys(asTable(config.pipeline))
  return settingsOf(scope).map((setting) => (
    <Field key={`${path.join('.')}-${setting.key}`} setting={setting} value={table[setting.key]} inherited={inherited} defined={defined}
      onApply={(value) => onChange(setIn(config, [...path, setting.key], value))} />
  ))
}
