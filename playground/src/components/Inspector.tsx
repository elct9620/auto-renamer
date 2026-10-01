import { useState } from 'react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Label } from '@/components/ui/label'
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
    <div className="grid gap-1.5">
      <Label className="font-mono text-xs">{label}</Label>
      {children}
      {error && <p className="text-xs text-destructive">{error}</p>}
    </div>
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

function Section({ title, children }: { title: string; children: React.ReactNode }) {
  return (
    <section className="grid gap-3">
      <h2 className="text-sm font-semibold">{title}</h2>
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
  const [nameError, setNameError] = useState('')

  if (selected === null) {
    const root = kind === 'global' ? ['default'] : []
    const defaults = asTable(root.length ? config.default : config)
    return (
      <Section title={kind === 'global' ? 'Defaults' : 'Folder configuration'}>
        <p className="text-xs text-muted-foreground">Select a watch, a pipeline or a stage to edit it.</p>
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
      <Section title={`Watch ${selected.name}`}>
        {['source', 'target', 'unit', 'pipelines', 'vars', 'dry_run'].map((key) => (
          <TomlField key={`${selected.name}-${key}`} label={key} value={inline(watch[key])}
            onApply={(value) => onChange(setIn(config, [...path, key], value))} />
        ))}
        <Button variant="destructive" size="sm" onClick={() => { onChange(setIn(config, path, undefined)); onSelect(null) }}>
          Remove watch
        </Button>
      </Section>
    )
  }

  if (selected.kind === 'pipeline') {
    return (
      <Section title={`Pipeline ${selected.name}`}>
        <Field label="name" error={nameError}>
          <Input key={selected.name} className="h-8 font-mono text-xs" defaultValue={selected.name} onBlur={(event) => {
            const name = event.target.value.trim()
            if (name === selected.name) return
            if (name === '' || name in asTable(config.pipeline)) {
              setNameError(name === '' ? 'A pipeline needs a name' : `A pipeline is already named ${name}`)
              return
            }
            setNameError('')
            onChange(renamePipeline(config, selected.name, name))
            onSelect({ kind: 'pipeline', name })
          }} />
        </Field>
        <Field label="Add a stage">
          <select className="h-8 rounded-md border bg-background px-2 text-sm" value="" onChange={(event) => {
            if (event.target.value) onChange(addStage(config, selected.name, newStage(event.target.value, stages(), read)))
          }}>
            <option value="">choose…</option>
            {stages().map(({ name }) => <option key={name}>{name}</option>)}
          </select>
        </Field>
        <Button variant="destructive" size="sm" onClick={() => { onChange(removePipeline(config, selected.name)); onSelect(null) }}>
          Remove pipeline
        </Button>
      </Section>
    )
  }

  const pipelineStages = stagesOf(config, selected.pipeline)
  const stage = pipelineStages[selected.index]
  if (stage === undefined) return null
  const at = (index: number) => onSelect({ ...selected, index })
  return (
    <Section title={`${stageName(stage)} · stage ${selected.index + 1} of ${selected.pipeline}`}>
      <StageForm key={`${selected.pipeline}-${selected.index}`} stage={stage}
        declaration={stages().find((declaration) => declaration.name === stageName(stage))!}
        onChange={(next) => onChange(replaceStage(config, selected.pipeline, selected.index, next))} />
      <div className="flex gap-2">
        <Button variant="outline" size="sm" disabled={selected.index === 0} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, -1)); at(selected.index - 1)
        }}>Earlier</Button>
        <Button variant="outline" size="sm" disabled={selected.index === pipelineStages.length - 1} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, 1)); at(selected.index + 1)
        }}>Later</Button>
        <Button variant="destructive" size="sm" onClick={() => { onChange(removeStage(config, selected.pipeline, selected.index)); onSelect(null) }}>
          Remove
        </Button>
      </div>
    </Section>
  )
}
