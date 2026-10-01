import { useTranslation } from 'react-i18next'

import { Choice } from '@/components/Choice'
import { Caption, Hint, TextField, ValueRows } from '@/components/Fields'
import { Switch } from '@/components/ui/switch'
import { type Stage, type Value, parametersOf, stageName, withParameter, withValue } from '../config'
import type { Declaration } from '../core'

/** The form of one stage, made from how the core declares it. */
export function StageForm({ stage, declaration, onChange }: {
  stage: Stage
  declaration: Declaration
  onChange: (stage: Stage) => void
}) {
  const { t } = useTranslation()
  const about = <Hint>{t(`stages.${declaration.name}.about`)}</Hint>
  const hint = (parameter: string) => t(`stages.${declaration.name}.parameters.${parameter}.about`)
  const name = (parameter: string) => t(`stages.${declaration.name}.parameters.${parameter}.name`)
  if (declaration.values) {
    return <div className="grid gap-3">{about}<ValueRows values={parametersOf(stage)} onChange={(values) =>
      onChange(Object.keys(values).length === 0 && declaration.bare ? stageName(stage) : withValue(stage, values))} /></div>
  }

  const single = typeof stage === 'string' ? undefined : stage[stageName(stage)]
  const parameters = parametersOf(stage)
  return (
    <div className="grid gap-3">
      {about}
      {declaration.value && (
        <TextField caption={<Caption name={t(`stages.${declaration.name}.value.name`)} code={declaration.name} />} kind={declaration.value} value={single} hint={t(`stages.${declaration.name}.value.about`)}
          onApply={(value) => value !== undefined && onChange(withValue(stage, value))} />
      )}
      {declaration.parameters.map((parameter) => {
        const value = parameters[parameter.name]
        const apply = (next: Value | undefined) => onChange(withParameter(stage, declaration, parameter.name, next))
        if (parameter.kind === 'boolean') {
          return (
            <div key={parameter.name} className="grid gap-1.5">
              <label className="flex items-center gap-2">
                <Switch checked={value === true} onCheckedChange={(checked) => apply(checked || undefined)} />
                <Caption name={name(parameter.name)} code={parameter.name} />
              </label>
              <Hint>{hint(parameter.name)}</Hint>
            </div>
          )
        }
        if (parameter.choices.length > 0) {
          return (
            <div key={parameter.name} className="grid gap-1.5">
              <Caption name={name(parameter.name)} code={parameter.name} required={parameter.required} />
              <Choice label={name(parameter.name)} className="font-mono text-xs" value={typeof value === 'string' ? value : ''} clearable
                options={parameter.choices.map((choice) => ({ value: choice, label: choice }))}
                onChange={(choice) => apply(choice || undefined)} />
              <Hint>{hint(parameter.name)}</Hint>
            </div>
          )
        }
        return (
          <TextField key={parameter.name} caption={<Caption name={name(parameter.name)} code={parameter.name} required={parameter.required} />}
            required={parameter.required} kind={parameter.kind}
            value={value} hint={hint(parameter.name)} onApply={apply} />
        )
      })}
    </div>
  )
}
