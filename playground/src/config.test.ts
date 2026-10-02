import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import {
  type Table,
  addStage,
  asTable,
  download,
  folderStart,
  overridden,
  insertStage,
  joinPipeline,
  moveStage,
  newStage,
  partPipeline,
  pipelinesOf,
  readStage,
  removePipeline,
  removeStage,
  renamePipeline,
  replaceStage,
  stageName,
  stagesOf,
  withParameter,
} from './config'
import { check, read, render, stages } from './core'
import { dropAt, laidOut, pipelineId, relaid, stageId, toGraph, watchId } from './graph'

const CONFIG = `
[default]
routes = [{ pipeline = "video" }]

[pipeline.video]
stages = [{ filter = { ext = ["mkv"] } }, { format = "{show}" }, "strip"]

[pipeline.subtitle]
stages = ["strip"]

[target.dst]
path = "/dst"

[watch.series]
source = "/src"
routes = [{ pipeline = "video", move = "dst" }, { pipeline = "subtitle", move = "dst" }]
vars = { show = "Alpha" }

[watch.movies]
source = "/movies"
`

function names(config: Table, pipeline = 'video'): string[] {
  return stagesOf(config, pipeline).map(stageName)
}

function edgeBetween(config: Table, source: string, target: string) {
  return toGraph(config).edges.find((edge) => edge.source === source && edge.target === target)
}

describe('drawing', () => {
  // @behavior PGE-001
  it('draws a pipeline as its stages in order', () => {
    const config = read(CONFIG)

    expect(edgeBetween(config, pipelineId('video'), stageId('video', 0))).toBeDefined()
    expect(edgeBetween(config, stageId('video', 0), stageId('video', 1))).toBeDefined()
    expect(edgeBetween(config, stageId('video', 1), stageId('video', 2))).toBeDefined()
    expect(names(config)).toEqual(['filter', 'format', 'strip'])
  })

  // @behavior PGE-002
  it('joins a watch to the pipelines of its routes, in order', () => {
    const config = read(CONFIG)

    expect(edgeBetween(config, watchId('series'), pipelineId('video'))?.label).toBe('1')
    expect(edgeBetween(config, watchId('series'), pipelineId('subtitle'))?.label).toBe('2')
    const x = (id: string) => toGraph(config).nodes.find((node) => node.id === id)!.position.x
    expect(x(pipelineId('video'))).toBeLessThan(x(pipelineId('subtitle')))
  })

  // @behavior PGE-003
  it('joins a watch without its own routes to the default ones', () => {
    const config = read(CONFIG)

    expect(edgeBetween(config, watchId('movies'), pipelineId('video'))).toBeDefined()
    expect(edgeBetween(config, watchId('movies'), pipelineId('subtitle'))).toBeUndefined()
  })

  // @behavior PGE-056
  it('stacks the stages of a pipeline below it in order', () => {
    const { nodes } = toGraph(read(CONFIG))
    const column = ['pipeline:video', 'stage:video:0', 'stage:video:1', 'stage:video:2']
      .map((id) => nodes.find((node) => node.id === id)!.position)

    expect(new Set(column.map(({ x }) => x)).size).toBe(1)
    expect(column.map(({ y }) => y)).toEqual([...column.map(({ y }) => y)].sort((a, b) => a - b))
    expect(new Set(column.map(({ y }) => y)).size).toBe(column.length)
  })
})

describe('layout', () => {
  function positionOf(config: Table, layout: Record<string, { x: number; y: number }>, id: string) {
    return laidOut(toGraph(config).nodes, layout).find((node) => node.id === id)?.position
  }

  // @behavior PGE-027
  it('draws a node moved by hand where it was left', () => {
    const config = read(CONFIG)

    expect(positionOf(config, { [watchId('series')]: { x: 500, y: 300 } }, watchId('series'))).toEqual({ x: 500, y: 300 })
  })

  it('drops a stage on the pipeline where it is drawn', () => {
    const config = read(CONFIG)
    const moved = laidOut(toGraph(config).nodes, { [pipelineId('subtitle')]: { x: 1200, y: 0 } })

    expect(dropAt(moved, { x: 1200, y: 600 })).toEqual({ pipeline: 'subtitle', index: 1 })
  })

  // @behavior PGE-028
  it('draws the stages of a pipeline whose order changed in their order', () => {
    const before = read(CONFIG)
    const layout = { [stageId('video', 1)]: { x: 900, y: 400 }, [watchId('series')]: { x: 500, y: 300 } }

    const after = moveStage(before, 'video', 1, -1)
    const kept = relaid(layout, before, after)

    const ordered = toGraph(after).nodes.filter((node) => node.id.startsWith('stage:video:'))
    expect(ordered.map((node) => positionOf(after, kept, node.id))).toEqual(ordered.map((node) => node.position))
  })
})

describe('editing', () => {
  const pipeline = (stages: string) => read(`[pipeline.video]\nstages = ${stages}`)

  // @behavior PGE-004
  it('adds a stage after the others', () => {
    const config = addStage(pipeline('[{ filter = { ext = ["mkv"] } }, "strip"]'), 'video', newStage('format', stages(), read))

    expect(names(config)).toEqual(['filter', 'strip', 'format'])
  })

  // @behavior PGE-005
  it('writes a stage moved earlier earlier', () => {
    const config = moveStage(pipeline('[{ filter = { ext = ["mkv"] } }, "strip", { format = "{show}" }]'), 'video', 2, -1)

    expect(names(config)).toEqual(['filter', 'format', 'strip'])
  })

  // @behavior PGE-006
  it('leaves the others in order when a stage is removed', () => {
    const config = removeStage(pipeline('[{ filter = { ext = ["mkv"] } }, { format = "{show}" }, "strip"]'), 'video', 1)

    expect(names(config)).toEqual(['filter', 'strip'])
  })

  // @behavior PGE-007
  it('writes edited parameters as the value of the stage', () => {
    const before = pipeline('[{ filter = { ext = ["mkv"] } }, "strip"]')

    const config = replaceStage(before, 'video', 0, readStage('{ filter = { ext = ["mp4"] } }', read))

    expect(stagesOf(config, 'video')).toEqual([{ filter: { ext: ['mp4'] } }, 'strip'])
  })

  // @behavior PGE-008
  it('writes a stage without parameters as its name', () => {
    const config = addStage(pipeline('[]'), 'video', newStage('strip', stages(), read))

    expect(render(config)).toContain('stages = ["strip"]')
  })

  // @behavior PGE-010
  it('writes a stage dropped between two stages between them', () => {
    const config = insertStage(pipeline('[{ filter = { ext = ["mkv"] } }, "strip"]'), 'video', 1, newStage('format', stages(), read))

    expect(names(config)).toEqual(['filter', 'format', 'strip'])
  })

  it('drops a stage before the stage nearest the point', () => {
    const config = read(CONFIG)
    const { nodes } = toGraph(config)
    const [, second] = nodes.filter((node) => node.id.startsWith('stage:video:'))

    expect(dropAt(nodes, { x: second.position.x, y: second.position.y + 10 })).toEqual({ pipeline: 'video', index: 1 })
  })

  // @behavior PGE-016
  it('starts a new stage with the example the core describes', () => {
    const example = stages().find((stage) => stage.name === 'regex')?.example

    const config = addStage(pipeline('[]'), 'video', newStage('regex', stages(), read))

    expect(stagesOf(config, 'video')).toEqual(stagesOf(read(`[pipeline.video]\nstages = [{ regex = ${example} }]`), 'video'))
  })
})

describe('forms', () => {
  const declaration = (name: string) => stages().find((stage) => stage.name === name)!

  // @behavior PGE-022
  it('writes a parameter set in the form into the stage', () => {
    const stage = withParameter({ filter: { ext: ['mkv'] } }, declaration('filter'), 'invert', true)

    expect(stage).toEqual({ filter: { ext: ['mkv'], invert: true } })
  })

  // @behavior PGE-023
  it('writes a stage left without parameters as its name', () => {
    const stage = withParameter({ strip: { groups: ['[]'] } }, declaration('strip'), 'groups', undefined)

    expect(render({ stages: [stage] })).toContain('stages = ["strip"]')
  })
})

describe('joining', () => {
  // @behavior PGE-011
  it('routes a pipeline a watch is joined to last', () => {
    const config = joinPipeline(read(CONFIG), 'series', 'extra')

    expect(pipelinesOf(config, 'series')).toEqual(['video', 'subtitle', 'extra'])
  })

  // @behavior PGE-012
  it('gives a watch following the default its own routes once joined', () => {
    const config = joinPipeline(read(CONFIG), 'movies', 'subtitle')

    expect(asTable(asTable(config.watch).movies).routes).toEqual([{ pipeline: 'video' }, { pipeline: 'subtitle' }])
  })

  // @behavior PGE-013
  it('drops the pipeline of a removed joint from the watch', () => {
    const config = partPipeline(read(CONFIG), 'series', 'video')

    expect(pipelinesOf(config, 'series')).toEqual(['subtitle'])
  })
})

describe('pipelines', () => {
  // @behavior PGE-014
  it('renames a pipeline where it is routed', () => {
    const config = renamePipeline(read(CONFIG), 'video', 'episode')

    expect(asTable(config.default).routes).toEqual([{ pipeline: 'episode' }])
    expect(pipelinesOf(config, 'series')).toEqual(['episode', 'subtitle'])
  })

  // @behavior PGE-015
  it('routes a removed pipeline nowhere', () => {
    const config = removePipeline(read(CONFIG), 'video')

    expect(asTable(config.default).routes).toEqual([])
    expect(pipelinesOf(config, 'series')).toEqual(['subtitle'])
    expect(asTable(config.pipeline).video).toBeUndefined()
  })
})

describe('download', () => {
  // @behavior PGE-009
  it('offers no file for a configuration the core refuses, and shows why', () => {
    const refused = read('[pipeline.video]\nstages = ["shred"]')

    const offered = download('global', refused, render, check)

    expect(offered.file).toBeUndefined()
    expect(offered.refused).toContain('shred')
  })

  it('offers a folder configuration under the name the CLI looks for', () => {
    const offered = download('folder', read('[vars]\nshow = "Beta"'), render, check)

    expect(offered.file?.name).toBe('auto-renamer.toml')
  })
})

describe('starting a folder configuration', () => {
  // @behavior PGE-059
  it('starts with the values the watch gives its files', () => {
    const config = read('[default]\nvars = { group = "Team" }\n\n[watch.series]\nsource = "/downloads"\nvars = { show = "Alpha" }\n')

    const started = read(render(folderStart(config, 'series')))

    expect(started).toEqual({ vars: { group: 'Team', show: 'Alpha' } })
  })
})

describe('overriding a pipeline in a folder', () => {
  // @behavior PGE-063
  it('copies the global pipeline in, keeping what the folder holds', () => {
    const folder = read('vars = { show = "Alpha" }\n\n[pipeline.subtitle]\nstages = ["strip"]\n\n[pipeline.video]\nstages = []\n')

    const result = overridden(folder, read(CONFIG), 'video')

    expect(result.vars).toEqual({ show: 'Alpha' })
    expect(names(result, 'subtitle')).toEqual(['strip'])
    expect(names(result)).toEqual(names(read(CONFIG)))
  })
})
