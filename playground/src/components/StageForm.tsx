import { Choice } from '@/components/Choice'
import { TextField, ValueRows } from '@/components/Fields'
import { Switch } from '@/components/ui/switch'
import { type Stage, type Value, parametersOf, stageName, withParameter, withValue } from '../config'
import type { Declaration } from '../core'

/** The form of one stage, made from how the core declares it. */
export function StageForm({ stage, declaration, onChange }: {
  stage: Stage
  declaration: Declaration
  onChange: (stage: Stage) => void
}) {
  if (declaration.values) {
    return <ValueRows values={parametersOf(stage)} onChange={(values) =>
      onChange(Object.keys(values).length === 0 && declaration.bare ? stageName(stage) : withValue(stage, values))} />
  }

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
              <Switch checked={value === true} onCheckedChange={(checked) => apply(checked || undefined)} />
              {parameter.name}
            </label>
          )
        }
        if (parameter.choices.length > 0) {
          return (
            <div key={parameter.name} className="grid gap-1.5">
              <span className="font-mono text-xs font-medium">{parameter.name}{parameter.required && <span className="text-destructive">*</span>}</span>
              <Choice label={parameter.name} className="font-mono text-xs" value={typeof value === 'string' ? value : ''} clearable
                options={parameter.choices.map((choice) => ({ value: choice, label: choice }))}
                onChange={(choice) => apply(choice || undefined)} />
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
