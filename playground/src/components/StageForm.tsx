import { Plus, X } from 'lucide-react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
import { type Stage, type Value, parametersOf, stageName, withParameter, withValue } from '../config'
import type { Declaration, ParameterKind } from '../core'

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

function TextField({ label, required, kind, value, onApply }: {
  label: string
  required?: boolean
  kind: ParameterKind
  value: Value | undefined
  onApply: (value: Value | undefined) => void
}) {
  return (
    <div className="grid gap-1.5">
      <Label className="font-mono text-xs">{label}{required && <span className="text-destructive">*</span>}</Label>
      <Input key={shown(value)} className="h-8 font-mono text-xs" defaultValue={shown(value)}
        inputMode={kind === 'integer' ? 'numeric' : undefined}
        placeholder={kind === 'texts' ? 'a, b, c' : undefined}
        onBlur={(event) => onApply(parsed(kind, event.target.value))} />
      {required && value === undefined && <p className="text-xs text-destructive">Required</p>}
    </div>
  )
}

function FixedValues({ stage, declaration, onChange }: {
  stage: Stage
  declaration: Declaration
  onChange: (stage: Stage) => void
}) {
  const values = parametersOf(stage)
  const fixed = (text: string): Value => (/^\d+$/.test(text.trim()) ? Number(text.trim()) : text)
  return (
    <div className="grid gap-2">
      {Object.entries(values).map(([name, value]) => (
        <div key={name} className="flex gap-1.5">
          <Input key={`name-${name}`} className="h-8 font-mono text-xs" defaultValue={name} onBlur={(event) => {
            const renamed = event.target.value.trim()
            if (renamed === '' || renamed === name || renamed in values) return
            onChange(withParameter(withParameter(stage, declaration, renamed, value), declaration, name, undefined))
          }} />
          <Input key={`value-${shown(value)}`} className="h-8 font-mono text-xs" defaultValue={shown(value)}
            onBlur={(event) => onChange(withParameter(stage, declaration, name, fixed(event.target.value)))} />
          <Button variant="ghost" size="icon-sm" aria-label={`Remove ${name}`}
            onClick={() => onChange(withParameter(stage, declaration, name, undefined))}><X /></Button>
        </div>
      ))}
      <Button variant="outline" size="sm" onClick={() => {
        let name = 'field'
        for (let n = 2; name in values; n += 1) name = `field_${n}`
        onChange(withParameter(stage, declaration, name, ''))
      }}><Plus />Add a value</Button>
    </div>
  )
}

/** The form of one stage, made from how the core declares it. */
export function StageForm({ stage, declaration, onChange }: {
  stage: Stage
  declaration: Declaration
  onChange: (stage: Stage) => void
}) {
  if (declaration.values) return <FixedValues stage={stage} declaration={declaration} onChange={onChange} />

  const single = typeof stage === 'string' ? undefined : stage[stageName(stage)]
  const parameters = parametersOf(stage)
  return (
    <div className="grid gap-3">
      {declaration.value && (
        <TextField label={declaration.name} kind={declaration.value} value={single}
          onApply={(value) => value !== undefined && onChange(withValue(stage, value))} />
      )}
      {declaration.parameters.map((parameter) => {
        const value = parameters[parameter.name]
        const apply = (next: Value | undefined) => onChange(withParameter(stage, declaration, parameter.name, next))
        if (parameter.kind === 'boolean') {
          return (
            <label key={parameter.name} className="flex items-center gap-2 font-mono text-xs">
              <input type="checkbox" checked={value === true} onChange={(event) => apply(event.target.checked || undefined)} />
              {parameter.name}
            </label>
          )
        }
        if (parameter.choices.length > 0) {
          return (
            <div key={parameter.name} className="grid gap-1.5">
              <Label className="font-mono text-xs">{parameter.name}{parameter.required && <span className="text-destructive">*</span>}</Label>
              <select className="h-8 rounded-md border bg-background px-2 font-mono text-xs" value={shown(value)}
                onChange={(event) => apply(event.target.value || undefined)}>
                <option value="">—</option>
                {parameter.choices.map((choice) => <option key={choice}>{choice}</option>)}
              </select>
            </div>
          )
        }
        return (
          <TextField key={parameter.name} label={parameter.name} required={parameter.required} kind={parameter.kind}
            value={value} onApply={apply} />
        )
      })}
    </div>
  )
}
