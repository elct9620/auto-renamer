import { ArrowDown, ArrowUp, Copy, type LucideIcon, Settings2, Trash2 } from 'lucide-react'
import { useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Choice } from '@/components/Choice'
import { StageName } from '@/components/Fields'
import { RouteForm, SettingsForm, TargetsForm } from '@/components/SettingsForm'
import { KIND_ICONS, stageIcon } from '@/components/kinds'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import {
  type Stage,
  type Table,
  addStage,
  asTable,
  moveRoute,
  moveStage,
  newStage,
  ownPipeline,
  removePipeline,
  removeRoute,
  removeStage,
  removeTarget,
  renamePipeline,
  replaceStage,
  routable,
  routesOf,
  setIn,
  stageName,
  stagesOf,
  targetsOf,
  withRoutes,
} from '../config'
import { type Kind, type Simulation, builtIns, read, stages } from '../core'
import type { Selected } from '../graph'
import { afterStage } from '../steps'
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

function Section({ title, icon: Icon, children }: { title: React.ReactNode; icon: LucideIcon; children: React.ReactNode }) {
  return (
    <section className="grid gap-3">
      <h2 className="flex items-center gap-1.5 text-sm font-semibold"><Icon className="size-4 text-muted-foreground" />{title}</h2>
      {children}
    </section>
  )
}

/** The files a triggered simulation ran through a stage, each as the stage left it. */
function FilesAfter({ simulation, pipeline, index, folder, claims }: {
  simulation: Simulation
  pipeline: string
  index: number
  folder: string | null
  claims: boolean
}) {
  const { t } = useTranslation()
  const { ran, taken } = afterStage(simulation.outcomes, pipeline, index, folder, claims)
  return (
    <section aria-label={t('inspector.after')} className="grid gap-1.5 border-t pt-3">
      <h3 className="text-xs font-semibold">{t('inspector.after')}</h3>
      {ran.length === 0 && <p className="text-xs text-muted-foreground">{t('inspector.none')}</p>}
      <ul className="grid gap-1.5">
        {ran.map(({ origin, step, changes }) => (
          <li key={origin} className="grid font-mono text-xs">
            <span className="truncate text-muted-foreground" title={origin}>{origin}</span>
            {step.stop !== null ? (
              <span className="text-destructive">{step.stop}</span>
            ) : (
              changes.map((change) => <span key={change.key}>{change.key}: {change.to}</span>)
            )}
          </li>
        ))}
      </ul>
      {taken.length > 0 && (
        <p className="text-xs text-muted-foreground">{t('inspector.taken', { count: taken.length })}</p>
      )}
    </section>
  )
}

/** Whether a stage stands for the claim: it and every stage before it are filters. */
function claims(stages: Stage[], index: number): boolean {
  return stages.slice(0, index + 1).every((one) => stageName(one) === 'filter')
}

/** A route's own form, with its place among the watch's routes and its removal. */
function RouteInspector({ config, selected, onChange, onSelect }: {
  config: Table
  selected: { kind: 'route'; watch: string; index: number }
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const { t } = useTranslation()
  const { watch, index } = selected
  const routes = routesOf(config, watch)
  const route = routes[index]
  if (route === undefined) return null
  const at = (to: number) => onSelect({ ...selected, index: to })
  return (
    <Section icon={KIND_ICONS.route} title={t('inspector.route', { position: index + 1, watch })}>
      {asTable(asTable(config.watch)[watch]).routes === undefined && (
        <p className="text-xs text-muted-foreground">{t('inspector.followsDefault')}</p>
      )}
      <RouteForm key={`${watch}-${index}`} route={route} pipelines={Object.keys(routable(config, builtIns()))}
        targets={Object.keys(targetsOf(config))}
        onChange={(next) => onChange(withRoutes(config, watch, (all) => all.map((one, place) => (place === index ? next : one))))} />
      <div className="flex gap-2">
        <Button variant="outline" size="sm" disabled={index === 0} onClick={() => {
          onChange(moveRoute(config, watch, index, -1)); at(index - 1)
        }}><ArrowUp />{t('inspector.earlier')}</Button>
        <Button variant="outline" size="sm" disabled={index === routes.length - 1} onClick={() => {
          onChange(moveRoute(config, watch, index, 1)); at(index + 1)
        }}><ArrowDown />{t('inspector.later')}</Button>
        <Button variant="destructive" size="sm" onClick={() => { onChange(removeRoute(config, watch, index)); onSelect(null) }}>
          <Trash2 />{t('inspector.removeRoute')}
        </Button>
      </div>
    </Section>
  )
}

/** A target's path and its removal, which takes every move to it along. */
function TargetInspector({ config, name, onChange, onSelect }: {
  config: Table
  name: string
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const { t } = useTranslation()
  return (
    <Section icon={KIND_ICONS.target} title={t('inspector.target', { name })}>
      <SettingsForm config={config} scope="target" path={['target', name]} onChange={onChange} />
      <Button variant="destructive" size="sm" onClick={() => { onChange(removeTarget(config, name)); onSelect(null) }}>
        <Trash2 />{t('inspector.removeTarget')}
      </Button>
    </Section>
  )
}

/** A built-in pipeline, or one of its stages: copied in or overridden in a folder to be changed, and a stage still
 * lists the files after it. */
function BuiltInInspector({ config, pipeline, stage, folders, simulation, folder, onChange, onOverride }: {
  config: Table
  pipeline: string
  stage: number | null
  folders: string[]
  simulation: Simulation | null
  folder: string | null
  onChange: (config: Table) => void
  onOverride?: (pipeline: string, folder: string) => void
}) {
  const { t } = useTranslation()
  if (!(pipeline in builtIns())) return null
  return (
    <Section icon={KIND_ICONS.pipeline} title={t('inspector.pipeline', { name: pipeline })}>
      <p className="text-xs text-muted-foreground">{t('inspector.builtIn')}</p>
      <Button variant="outline" size="sm" onClick={() => onChange(ownPipeline(config, builtIns(), pipeline))}>
        <Copy />{t('inspector.copyBuiltIn')}
      </Button>
      {onOverride && (
        <Field label={t('inspector.override')}>
          <Choice value="" placeholder={t('inspector.chooseFolder')} options={folders.map((one) => ({ value: one, label: one }))}
            onChange={(one) => onOverride(pipeline, one)} />
        </Field>
      )}
      {simulation && stage !== null && <FilesAfter simulation={simulation} pipeline={pipeline} index={stage} folder={folder}
        claims={claims(stagesOf({ pipeline: builtIns() }, pipeline), stage)} />}
    </Section>
  )
}

/** A pipeline the configuration defines: its name, a stage to add, an override in a folder, and its removal. */
function PipelineInspector({ config, name: pipeline, folders, onChange, onSelect, onOverride }: {
  config: Table
  name: string
  folders: string[]
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
  onOverride?: (pipeline: string, folder: string) => void
}) {
  const { t } = useTranslation()
  const [nameError, setNameError] = useState('')
  return (
    <Section icon={KIND_ICONS.pipeline} title={t('inspector.pipeline', { name: pipeline })}>
      <Field label={t('inspector.name')} error={nameError}>
        <Input key={pipeline} className="h-8 font-mono text-xs" defaultValue={pipeline} onBlur={(event) => {
          const name = event.target.value.trim()
          if (name === pipeline) return
          if (name === '' || name in asTable(config.pipeline)) {
            setNameError(name === '' ? t('inspector.nameMissing') : t('inspector.nameTaken', { name }))
            return
          }
          setNameError('')
          onChange(renamePipeline(config, pipeline, name))
          onSelect({ kind: 'pipeline', name })
        }} />
      </Field>
      <Field label={t('inspector.addStage')}>
        <Choice value="" placeholder={t('inspector.choose')} options={stages().map(({ name }) => ({ value: name, label: name }))}
          onChange={(name) => onChange(addStage(config, pipeline, newStage(name, stages(), read)))} />
      </Field>
      {onOverride && (
        <Field label={t('inspector.override')}>
          <Choice value="" placeholder={t('inspector.chooseFolder')} options={folders.map((folder) => ({ value: folder, label: folder }))}
            onChange={(folder) => onOverride(pipeline, folder)} />
        </Field>
      )}
      <Button variant="destructive" size="sm" onClick={() => { onChange(removePipeline(config, pipeline)); onSelect(null) }}>
        <Trash2 />{t('inspector.removePipeline')}
      </Button>
    </Section>
  )
}

/** A stage of a pipeline the configuration defines: its form, its place, its removal and the files after it. */
function StageInspector({ config, selected, simulation, folder, onChange, onSelect }: {
  config: Table
  selected: { kind: 'stage'; pipeline: string; index: number }
  simulation: Simulation | null
  folder: string | null
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const { t } = useTranslation()
  const pipelineStages = stagesOf(config, selected.pipeline)
  const stage = pipelineStages[selected.index]
  if (stage === undefined) return null
  const at = (index: number) => onSelect({ ...selected, index })
  return (
    <Section icon={stageIcon(stageName(stage))} title={<>
      <StageName name={stageName(stage)} /> · {t('inspector.stage', { position: selected.index + 1, pipeline: selected.pipeline })}
    </>}>
      <StageForm key={`${selected.pipeline}-${selected.index}`} stage={stage}
        declaration={stages().find((declaration) => declaration.name === stageName(stage))!}
        onChange={(next) => onChange(replaceStage(config, selected.pipeline, selected.index, next))} />
      <div className="flex gap-2">
        <Button variant="outline" size="sm" disabled={selected.index === 0} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, -1)); at(selected.index - 1)
        }}><ArrowUp />{t('inspector.earlier')}</Button>
        <Button variant="outline" size="sm" disabled={selected.index === pipelineStages.length - 1} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, 1)); at(selected.index + 1)
        }}><ArrowDown />{t('inspector.later')}</Button>
        <Button variant="destructive" size="sm" onClick={() => { onChange(removeStage(config, selected.pipeline, selected.index)); onSelect(null) }}>
          <Trash2 />{t('inspector.remove')}
        </Button>
      </div>
      {simulation && <FilesAfter simulation={simulation} pipeline={selected.pipeline} index={selected.index} folder={folder}
        claims={claims(pipelineStages, selected.index)} />}
    </Section>
  )
}

/** With nothing selected: the defaults and the targets of the global configuration, or a folder configuration's
 * own settings. */
function DefaultsInspector({ config, kind, onChange }: { config: Table; kind: Kind; onChange: (config: Table) => void }) {
  const { t } = useTranslation()
  const global = kind === 'global'
  return (
    <Section icon={Settings2} title={global ? t('inspector.defaults') : t('inspector.folder')}>
      <p className="text-xs text-muted-foreground">{t('inspector.hint')}</p>
      <SettingsForm config={config} scope={global ? 'default' : 'folder'} path={global ? ['default'] : []} onChange={onChange} />
      {global && <TargetsForm config={config} onChange={onChange} />}
    </Section>
  )
}

function WatchInspector({ config, name, onChange, onSelect }: {
  config: Table
  name: string
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const { t } = useTranslation()
  const path = ['watch', name]
  return (
    <Section icon={KIND_ICONS.watch} title={t('inspector.watch', { name })}>
      <SettingsForm config={config} scope="watch" path={path} onChange={onChange} />
      <Button variant="destructive" size="sm" onClick={() => { onChange(setIn(config, path, undefined)); onSelect(null) }}>
        <Trash2 />{t('inspector.removeWatch')}
      </Button>
    </Section>
  )
}

export function Inspector({ config, kind, selected, folders = [], simulation = null, folder = null, onChange, onSelect, onOverride }: {
  config: Table
  kind: Kind
  selected: Selected | null
  simulation?: Simulation | null
  /** The folder whose configuration is edited, relative to the source; null for the global configuration. */
  folder?: string | null
  /** The source folders a global pipeline can be overridden in. */
  folders?: string[]
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
  onOverride?: (pipeline: string, folder: string) => void
}) {
  if (selected === null) return <DefaultsInspector config={config} kind={kind} onChange={onChange} />

  if (selected.kind === 'watch') return <WatchInspector config={config} name={selected.name} onChange={onChange} onSelect={onSelect} />

  if (selected.kind === 'route') return <RouteInspector config={config} selected={selected} onChange={onChange} onSelect={onSelect} />

  if (selected.kind === 'target') {
    return <TargetInspector config={config} name={selected.name} onChange={onChange} onSelect={onSelect} />
  }

  // A built-in pipeline and its stages are changed only once copied in as the configuration's own.
  const pipeline = selected.kind === 'pipeline' ? selected.name : selected.pipeline
  if (!(pipeline in asTable(config.pipeline))) {
    return <BuiltInInspector config={config} pipeline={pipeline} stage={selected.kind === 'stage' ? selected.index : null}
      folders={folders} simulation={simulation} folder={folder} onChange={onChange} onOverride={onOverride} />
  }

  if (selected.kind === 'pipeline') {
    return <PipelineInspector config={config} name={selected.name} folders={folders} onChange={onChange} onSelect={onSelect}
      onOverride={onOverride} />
  }

  return <StageInspector config={config} selected={selected} simulation={simulation} folder={folder} onChange={onChange}
    onSelect={onSelect} />
}
