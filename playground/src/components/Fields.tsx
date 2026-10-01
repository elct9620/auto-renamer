import { Plus, X } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { type Table, type Value, setIn } from '../config'
import type { ParameterKind } from '../core'

// Fields the stage and settings forms share.

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

/** A value typed as text and read as `kind` once the field is left; `caption` replaces the plain label. */
export function TextField({ label, required, kind, value, placeholder, caption, onApply }: {
  label: string
  required?: boolean
  kind: ParameterKind
  value: Value | undefined
  placeholder?: string
  caption?: React.ReactNode
  onApply: (value: Value | undefined) => void
}) {
  const { t } = useTranslation()
  return (
    <label className="grid gap-1.5">
      {caption ?? <span className="font-mono text-xs font-medium">{label}{required && <span className="text-destructive">*</span>}</span>}
      <Input key={shown(value)} className="h-8 font-mono text-xs" defaultValue={shown(value)}
        type={kind === 'integer' ? 'number' : 'text'} min={kind === 'integer' ? 1 : undefined}
        placeholder={placeholder ?? (kind === 'texts' ? 'a, b, c' : undefined)}
        onBlur={(event) => onApply(parsed(kind, event.target.value))} />
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
