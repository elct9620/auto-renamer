import { Plus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { type Table, type Value, setIn } from '../config'
import type { ParameterKind } from '../core'

// Fields the stage and settings forms share.

/** What a field is for, under it. */
export function Hint({ children }: { children: React.ReactNode }) {
  return <p className="text-xs text-muted-foreground">{children}</p>
}

/** A field's name in the page's language, with the key the configuration writes beside it. */
export function Caption({ name, code, required }: {
  name: string
  code: string
  required?: boolean
}) {
  return (
    <span className="flex items-center gap-1.5 text-xs font-medium">
      {name}
      <code className="font-mono font-normal text-muted-foreground">{code}</code>
      {required && <span className="text-destructive">*</span>}
    </span>
  )
}

/** A stage by its name in the page's language, with the name the configuration writes beside it. */
export function StageName({ name }: { name: string }) {
  const { t } = useTranslation()
  return (
    <span className="inline-flex items-baseline gap-1.5">
      {t(`stages.${name}.name`)}
      <code className="font-mono text-[0.85em] font-normal text-muted-foreground">{name}</code>
    </span>
  )
}

function shown(value: Value | undefined): string {
  if (value === undefined || typeof value === 'object' && !Array.isArray(value)) return ''
  return Array.isArray(value) ? value.join(', ') : String(value)
}

/** What a field's text declares for a parameter of a kind; nothing when it is left empty. */
function parsed(kind: ParameterKind, text: string): Value | undefined {
  const trimmed = text.trim()
  if (trimmed === '') return undefined
  if (kind === 'texts') return trimmed.split(',').map((part) => part.trim()).filter((part) => part !== '')
  if (kind === 'integer') return Number.isInteger(Number(trimmed)) ? Number(trimmed) : trimmed
  return text
}

/** A value typed as text and read as `kind` once the field is left. */
export function TextField({ caption, required, kind, value, placeholder, hint, onApply }: {
  caption: React.ReactNode
  required?: boolean
  kind: ParameterKind
  value: Value | undefined
  placeholder?: string
  hint?: string
  onApply: (value: Value | undefined) => void
}) {
  const { t } = useTranslation()
  return (
    <label className="grid gap-1.5">
      {caption}
      <Input key={shown(value)} className="h-8 font-mono text-xs" defaultValue={shown(value)}
        type={kind === 'integer' ? 'number' : 'text'} min={kind === 'integer' ? 1 : undefined}
        placeholder={placeholder ?? (kind === 'texts' ? 'a, b, c' : undefined)}
        onBlur={(event) => onApply(parsed(kind, event.target.value))} />
      {hint && <Hint>{hint}</Hint>}
      {required && value === undefined && <p className="text-xs text-destructive">{t('inspector.required')}</p>}
    </label>
  )
}

/** Fixed values under names of the user's choosing, as a table of rows. */
export function ValueRows({ values, onChange }: { values: Table; onChange: (values: Table) => void }) {
  const { t } = useTranslation()
  const fixed = (text: string): Value => (/^\d+$/.test(text.trim()) ? Number(text.trim()) : text)
  return (
    <div className="grid gap-2">
      {Object.entries(values).map(([name, value]) => (
        <div key={name} className="flex gap-1.5">
          <Input key={`name-${name}`} aria-label={t('inspector.valueName')} className="h-8 font-mono text-xs" defaultValue={name} onBlur={(event) => {
            const renamed = event.target.value.trim()
            if (renamed === '' || renamed === name || renamed in values) return
            onChange(Object.fromEntries(Object.entries(values).map(([key, one]) => [key === name ? renamed : key, one])))
          }} />
          <Input key={`value-${shown(value)}`} aria-label={t('inspector.valueOf', { name })} className="h-8 font-mono text-xs" defaultValue={shown(value)}
            onBlur={(event) => onChange({ ...values, [name]: fixed(event.target.value) })} />
          <Button variant="ghost" size="icon-sm" aria-label={`Remove ${name}`}
            onClick={() => onChange(setIn(values, [name], undefined))}><X /></Button>
        </div>
      ))}
      <Button variant="outline" size="sm" onClick={() => {
        let name = 'field'
        for (let n = 2; name in values; n += 1) name = `field_${n}`
        onChange({ ...values, [name]: '' })
      }}><Plus />{t('inspector.addValue')}</Button>
    </div>
  )
}
