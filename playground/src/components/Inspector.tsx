import { ArrowLeft, ArrowRight, type LucideIcon, Settings2, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Choice } from '@/components/Choice'
import { KIND_ICONS } from '@/components/kinds'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import { errorOf } from '@/lib/utils'
import {
  type Table,
  type Value,
  addStage,
  asTable,
  moveStage,
  newStage,
  removePipeline,
  removeStage,
  renamePipeline,
  replaceStage,
  setIn,
  stageName,
  stagesOf,
} from '../config'
import { type Kind, read, render, stages } from '../core'
import type { Selected } from '../graph'
import { StageForm } from './StageForm'

function Field({ label, children, error }: { label: string; children: React.ReactNode; error?: string }) {
  return (
    <label className="grid gap-1.5">
      <span className="font-mono text-xs font-medium">{label}</span>
      {children}
      {error && <p className="text-xs text-destructive">{error}</p>}
    </label>
  )
}

/** A text field whose value is read as an inline TOML value, applied once it reads. It starts over from
 * the configuration whenever the configuration's value changes, as after an import. */
function TomlField(props: { label: string; value: string; onApply: (value: Value | undefined) => void }) {
  return <EditedTomlField key={props.value} {...props} />
}

function EditedTomlField({ label, value, onApply }: { label: string; value: string; onApply: (value: Value | undefined) => void }) {
  const [text, setText] = useState(value)
  const [error, setError] = useState('')
  const apply = () => {
    if (text.trim() === '') {
      setError('')
      onApply(undefined)
      return
    }
    try {
      onApply(read(`value = ${text}`).value)
      setError('')
    } catch (failure) {
      setError(errorOf(failure))
    }
  }
  return (
    <Field label={label} error={error}>
      <Textarea className="min-h-9 font-mono text-xs" value={text} rows={1}
        onChange={(event) => setText(event.target.value)} onBlur={apply} />
    </Field>
  )
}

function inline(value: Value | undefined): string {
  if (value === undefined) return ''
  return render({ value: [value] }).trim().replace(/^value = \[/, '').replace(/\]$/, '')
}

function Section({ title, icon: Icon, children }: { title: string; icon: LucideIcon; children: React.ReactNode }) {
  return (
    <section className="grid gap-3">
      <h2 className="flex items-center gap-1.5 text-sm font-semibold"><Icon className="size-4 text-muted-foreground" />{title}</h2>
      {children}
    </section>
  )
}

export function Inspector({ config, kind, selected, onChange, onSelect }: {
  config: Table
  kind: Kind
  selected: Selected | null
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const { t } = useTranslation()
  const [nameError, setNameError] = useState('')

  if (selected === null) {
    const root = kind === 'global' ? ['default'] : []
    const defaults = asTable(root.length ? config.default : config)
    return (
      <Section icon={Settings2} title={kind === 'global' ? t('inspector.defaults') : t('inspector.folder')}>
        <p className="text-xs text-muted-foreground">{t('inspector.hint')}</p>
        <TomlField key={`vars-${kind}`} label="vars" value={inline(defaults.vars)}
          onApply={(value) => onChange(setIn(config, [...root, 'vars'], value))} />
        {kind === 'global' && (
          <TomlField key="default-pipelines" label="pipelines" value={inline(defaults.pipelines)}
            onApply={(value) => onChange(setIn(config, ['default', 'pipelines'], value))} />
        )}
        <TomlField key={`batch-${kind}`} label="batch_max" value={inline(defaults.batch_max)}
          onApply={(value) => onChange(setIn(config, [...root, 'batch_max'], value))} />
      </Section>
    )
  }

  if (selected.kind === 'watch') {
    const path = ['watch', selected.name]
    const watch = asTable(asTable(config.watch)[selected.name])
    return (
      <Section icon={KIND_ICONS.watch} title={t('inspector.watch', { name: selected.name })}>
        {['source', 'target', 'unit', 'pipelines', 'vars', 'dry_run'].map((key) => (
          <TomlField key={`${selected.name}-${key}`} label={key} value={inline(watch[key])}
            onApply={(value) => onChange(setIn(config, [...path, key], value))} />
        ))}
        <Button variant="destructive" size="sm" onClick={() => { onChange(setIn(config, path, undefined)); onSelect(null) }}>
          <Trash2 />{t('inspector.removeWatch')}
        </Button>
      </Section>
    )
  }

  if (selected.kind === 'pipeline') {
    return (
      <Section icon={KIND_ICONS.pipeline} title={t('inspector.pipeline', { name: selected.name })}>
        <Field label={t('inspector.name')} error={nameError}>
          <Input key={selected.name} className="h-8 font-mono text-xs" defaultValue={selected.name} onBlur={(event) => {
            const name = event.target.value.trim()
            if (name === selected.name) return
            if (name === '' || name in asTable(config.pipeline)) {
              setNameError(name === '' ? t('inspector.nameMissing') : t('inspector.nameTaken', { name }))
              return
            }
            setNameError('')
            onChange(renamePipeline(config, selected.name, name))
            onSelect({ kind: 'pipeline', name })
          }} />
        </Field>
        <Field label={t('inspector.addStage')}>
          <Choice value="" placeholder={t('inspector.choose')} options={stages().map(({ name }) => ({ value: name, label: name }))}
            onChange={(name) => onChange(addStage(config, selected.name, newStage(name, stages(), read)))} />
        </Field>
        <Button variant="destructive" size="sm" onClick={() => { onChange(removePipeline(config, selected.name)); onSelect(null) }}>
          <Trash2 />{t('inspector.removePipeline')}
        </Button>
      </Section>
    )
  }

  const pipelineStages = stagesOf(config, selected.pipeline)
  const stage = pipelineStages[selected.index]
  if (stage === undefined) return null
  const at = (index: number) => onSelect({ ...selected, index })
  return (
    <Section icon={KIND_ICONS.stage} title={t('inspector.stage', { kind: stageName(stage), position: selected.index + 1, pipeline: selected.pipeline })}>
      <StageForm key={`${selected.pipeline}-${selected.index}`} stage={stage}
        declaration={stages().find((declaration) => declaration.name === stageName(stage))!}
        onChange={(next) => onChange(replaceStage(config, selected.pipeline, selected.index, next))} />
      <div className="flex gap-2">
        <Button variant="outline" size="sm" disabled={selected.index === 0} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, -1)); at(selected.index - 1)
        }}><ArrowLeft />{t('inspector.earlier')}</Button>
        <Button variant="outline" size="sm" disabled={selected.index === pipelineStages.length - 1} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, 1)); at(selected.index + 1)
        }}>{t('inspector.later')}<ArrowRight /></Button>
        <Button variant="destructive" size="sm" onClick={() => { onChange(removeStage(config, selected.pipeline, selected.index)); onSelect(null) }}>
          <Trash2 />{t('inspector.remove')}
        </Button>
      </div>
    </Section>
  )
}
