// The configuration as the core reads it: a TOML table. It is the page's only state; the drawing and
// every edit are made from it and back into it.

import type { Declaration } from './core'

export type Value = string | number | boolean | Value[] | Table
export interface Table {
  [key: string]: Value
}

/** The pipelines the core builds in, which a route may name without the configuration defining them. */
export const BUILT_IN_PIPELINES = ['series-video', 'series-subtitle']

/** A stage as written in `stages`: its name alone, or a one-key table of its name and parameters. */
export type Stage = string | Table

export function asTable(value: Value | undefined): Table {
  return value !== null && typeof value === 'object' && !Array.isArray(value) ? value : {}
}

export function stageName(stage: Stage): string {
  return typeof stage === 'string' ? stage : (Object.keys(stage)[0] ?? '')
}

export function stagesOf(config: Table, pipeline: string): Stage[] {
  const stages = asTable(asTable(config.pipeline)[pipeline]).stages
  return Array.isArray(stages) ? (stages as Stage[]) : []
}

/** The routes a watch claims files by, in order: its own, or the default ones. */
export function routesOf(config: Table, watch: string): Table[] {
  const own = asTable(asTable(config.watch)[watch]).routes
  return asRoutes(own === undefined ? asTable(config.default).routes : own)
}

function asRoutes(value: Value | undefined): Table[] {
  return Array.isArray(value) ? value.map((route) => asTable(route)) : []
}

/** The pipelines a watch's routes run, in the order the routes claim. */
export function pipelinesOf(config: Table, watch: string): string[] {
  return routesOf(config, watch).flatMap((route) => (typeof route.pipeline === 'string' ? [route.pipeline] : []))
}

/** The targets the configuration declares, by name, with the root each stands for. */
export function targetsOf(config: Table): Record<string, string> {
  return Object.fromEntries(
    Object.entries(asTable(config.target)).flatMap(([name, value]) => {
      const path = asTable(value).path
      return typeof path === 'string' ? [[name, path]] : []
    }),
  )
}

/** What a folder configuration started for a watch holds: the values the watch gives its files, for the folder
 * to change. Nothing is taken from the folder's name; the user names the show. */
export function folderStart(config: Table, watch: string): Table {
  const vars = { ...asTable(asTable(config.default).vars), ...asTable(asTable(asTable(config.watch)[watch]).vars) }
  return Object.keys(vars).length ? { vars } : {}
}

/** A folder configuration with the global pipeline `name` copied in, replacing its own of that name only. */
export function overridden(folder: Table, global: Table, name: string): Table {
  return setIn(folder, ['pipeline', name], asTable(global.pipeline)[name])
}

function withStages(config: Table, pipeline: string, stages: Stage[]): Table {
  const pipelines = asTable(config.pipeline)
  return {
    ...config,
    pipeline: { ...pipelines, [pipeline]: { ...asTable(pipelines[pipeline]), stages } },
  }
}

export function addStage(config: Table, pipeline: string, stage: Stage): Table {
  return insertStage(config, pipeline, stagesOf(config, pipeline).length, stage)
}

/** Places a stage at `index`, before the stage that was there. */
export function insertStage(config: Table, pipeline: string, index: number, stage: Stage): Table {
  const stages = [...stagesOf(config, pipeline)]
  stages.splice(index, 0, stage)
  return withStages(config, pipeline, stages)
}

export function removeStage(config: Table, pipeline: string, index: number): Table {
  return withStages(
    config,
    pipeline,
    stagesOf(config, pipeline).filter((_, at) => at !== index),
  )
}

/** Moves a stage by `offset` places, keeping it within the pipeline. */
export function moveStage(config: Table, pipeline: string, index: number, offset: number): Table {
  const stages = [...stagesOf(config, pipeline)]
  const to = Math.min(Math.max(index + offset, 0), stages.length - 1)
  const [stage] = stages.splice(index, 1)
  stages.splice(to, 0, stage)
  return withStages(config, pipeline, stages)
}

export function replaceStage(config: Table, pipeline: string, index: number, stage: Stage): Table {
  return withStages(
    config,
    pipeline,
    stagesOf(config, pipeline).map((current, at) => (at === index ? stage : current)),
  )
}

/** Writes the routes of a watch as its own, starting from the default ones it followed. */
export function withRoutes(config: Table, watch: string, change: (routes: Table[]) => Table[]): Table {
  return setIn(config, ['watch', watch, 'routes'], change(routesOf(config, watch)))
}

/** Adds a route of a pipeline to a watch, unless one already runs it. */
export function joinPipeline(config: Table, watch: string, pipeline: string): Table {
  return withRoutes(config, watch, (routes) =>
    routes.some((route) => route.pipeline === pipeline) ? routes : [...routes, { pipeline }])
}

export function partPipeline(config: Table, watch: string, pipeline: string): Table {
  return withRoutes(config, watch, (routes) => routes.filter((route) => route.pipeline !== pipeline))
}

/** Rewrites the pipeline every route names, its rejected route's included: in the default's routes and in
 * each watch's own. A route whose pipeline is renamed to nothing goes; a rejected route loses its pipeline. */
function rewriteRouted(config: Table, change: (name: string) => string | undefined): Table {
  const rewrite = (routes: Table[]) =>
    routes.flatMap((route) => {
      const pipeline = typeof route.pipeline === 'string' ? change(route.pipeline) : undefined
      if (pipeline === undefined) return []
      const rejected = asTable(route.rejected)
      const kept = typeof rejected.pipeline === 'string' ? change(rejected.pipeline) : undefined
      const next = { ...route, pipeline }
      return [route.rejected === undefined ? next : { ...next, rejected: setIn(rejected, ['pipeline'], kept) }]
    })
  let next = config
  const defaults = asTable(config.default).routes
  if (defaults !== undefined) next = setIn(next, ['default', 'routes'], rewrite(asRoutes(defaults)))
  for (const [watch, value] of Object.entries(asTable(config.watch))) {
    const own = asTable(value).routes
    if (own !== undefined) next = setIn(next, ['watch', watch, 'routes'], rewrite(asRoutes(own)))
  }
  return next
}

/** Renames a pipeline and every route naming it. */
export function renamePipeline(config: Table, from: string, to: string): Table {
  const pipeline = asTable(config.pipeline)[from]
  return rewriteRouted(
    setIn(setIn(config, ['pipeline', from], undefined), ['pipeline', to], pipeline),
    (name) => (name === from ? to : name),
  )
}

export function removePipeline(config: Table, name: string): Table {
  return rewriteRouted(setIn(config, ['pipeline', name], undefined), (listed) => (listed === name ? undefined : listed))
}

export function setIn(config: Table, path: string[], value: Value | undefined): Table {
  const [key, ...rest] = path
  const next = { ...config }
  if (rest.length > 0) {
    next[key] = setIn(asTable(config[key]), rest, value)
  } else if (value === undefined) {
    delete next[key]
  } else {
    next[key] = value
  }
  return next
}

/** The named parameters a stage is written with; none for a bare name or a single value. */
export function parametersOf(stage: Stage): Table {
  return typeof stage === 'string' ? {} : asTable(stage[stageName(stage)])
}

/** Sets or clears one parameter; a stage left with none is written as its name when its name alone
 * declares it. */
export function withParameter(stage: Stage, declaration: Declaration, name: string, value: Value | undefined): Stage {
  const kind = stageName(stage)
  const parameters = setIn(parametersOf(stage), [name], value)
  return Object.keys(parameters).length === 0 && declaration.bare ? kind : { [kind]: parameters }
}

/** The stage declared with a single value, as `{ format = "..." }`. */
export function withValue(stage: Stage, value: Value): Stage {
  return { [stageName(stage)]: value }
}

/** The stage an inline TOML text declares, read by the core's own reader. */
export function readStage(text: string, read: (text: string) => Table): Stage {
  const stages = read(`stages = [${text}]`).stages
  if (!Array.isArray(stages) || stages.length !== 1) {
    throw new Error('write exactly one stage')
  }
  return stages[0] as Stage
}

/** A new stage of a kind, declared as the core's example of it. */
export function newStage(kind: string, stages: Declaration[], read: (text: string) => Table): Stage {
  const example = stages.find((stage) => stage.name === kind)?.example ?? ''
  return example === '' ? kind : readStage(`{ ${kind} = ${example} }`, read)
}

export interface Download {
  file?: { name: string; text: string }
  refused?: string
}

/** The file to offer for a configuration, or why the core refuses it. */
export function download(
  kind: 'global' | 'folder',
  config: Table,
  render: (table: Table) => string,
  check: (kind: 'global' | 'folder', text: string) => void,
): Download {
  const text = render(config)
  try {
    check(kind, text)
  } catch (error) {
    return { refused: error instanceof Error ? error.message : String(error) }
  }
  return { file: { name: kind === 'global' ? 'config.toml' : 'auto-renamer.toml', text } }
}
