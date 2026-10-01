// The configuration as the core reads it: a TOML table. It is the page's only state; the drawing and
// every edit are made from it and back into it.

export type Value = string | number | boolean | Value[] | Table
export interface Table {
  [key: string]: Value
}

/** A stage as written in `stages`: its name alone, or a one-key table of its name and settings. */
export type Stage = string | Table

/** Each stage the core declares, with settings it accepts, so a new stage starts out valid. */
export const STAGE_TEMPLATES: Record<string, string> = {
  filter: '{ ext = ["mkv", "mp4"] }',
  number: '{ from = "path", into = "season", prefix = "Season" }',
  regex: `{ pattern = '(\\d+)', into = "episode" }`,
  set: '{ kind = "video" }',
  default: '{ season = 1 }',
  replace: '{ find = "_", with = " " }',
  case: '{ to = "lower" }',
  strip: '{}',
  format: '"{show} s{season:02}e{episode:02}"',
  lift: '1',
  folder: '"{show}"',
  next: '{ into = "episode", like = "{show} s{season:02}e{episode:02}" }',
  rank: '{ into = "episode", by = ["season"] }',
  take: '{ fields = ["episode"] }',
  move: '',
  cleanup: '',
}

export function asTable(value: Value | undefined): Table {
  return value !== null && typeof value === 'object' && !Array.isArray(value) ? value : {}
}

function asNames(value: Value | undefined): string[] {
  return Array.isArray(value) ? value.filter((name): name is string => typeof name === 'string') : []
}

export function stageName(stage: Stage): string {
  return typeof stage === 'string' ? stage : (Object.keys(stage)[0] ?? '')
}

export function stagesOf(config: Table, pipeline: string): Stage[] {
  const stages = asTable(asTable(config.pipeline)[pipeline]).stages
  return Array.isArray(stages) ? (stages as Stage[]) : []
}

/** The pipelines a watch runs, in order: its own list, or the default one. */
export function pipelinesOf(config: Table, watch: string): string[] {
  const own = asTable(asTable(config.watch)[watch]).pipelines
  return own === undefined ? asNames(asTable(config.default).pipelines) : asNames(own)
}

function withStages(config: Table, pipeline: string, stages: Stage[]): Table {
  const pipelines = asTable(config.pipeline)
  return {
    ...config,
    pipeline: { ...pipelines, [pipeline]: { ...asTable(pipelines[pipeline]), stages } },
  }
}

export function addStage(config: Table, pipeline: string, stage: Stage): Table {
  return withStages(config, pipeline, [...stagesOf(config, pipeline), stage])
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

/** A stage as the inline TOML a person edits, such as `{ filter = { ext = ["mkv"] } }` or `"move"`. */
export function stageText(stage: Stage, render: (table: Table) => string): string {
  return render({ stages: [stage] })
    .trim()
    .replace(/^stages = \[/, '')
    .replace(/\]$/, '')
}

/** The stage an inline TOML text declares, read by the core's own reader. */
export function readStage(text: string, read: (text: string) => Table): Stage {
  const stages = read(`stages = [${text}]`).stages
  if (!Array.isArray(stages) || stages.length !== 1) {
    throw new Error('write exactly one stage')
  }
  return stages[0] as Stage
}

/** A new stage of a kind, with the settings its template gives. */
export function newStage(kind: string, read: (text: string) => Table): Stage {
  const settings = STAGE_TEMPLATES[kind] ?? ''
  return settings === '' ? kind : readStage(`{ ${kind} = ${settings} }`, read)
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
  check: (kind: 'global' | 'folder', text: string) => string[],
): Download {
  const text = render(config)
  try {
    check(kind, text)
  } catch (error) {
    return { refused: error instanceof Error ? error.message : String(error) }
  }
  return { file: { name: kind === 'global' ? 'config.toml' : 'auto-renamer.toml', text } }
}
