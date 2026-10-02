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
  readStage,
  removePipeline,
  moveRoute,
  ownPipeline,
  removeRoute,
  removeStage,
  removeTarget,
  renamePipeline,
  routable,
  routeTo,
  routesOf,
  setIn,
  replaceStage,
  stageName,
  stagesOf,
  transferStage,
  withParameter,
} from './config'
import { builtIns, check, read, render, stages } from './core'
import { absolute, dropAt, laidOut, pipelineId, relaid, routeAt, routeId, stageId, targetId, toGraph, watchId } from './graph'

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

/** The pipelines a watch's routes run, in the order the routes claim. */
function pipelinesOf(config: Table, watch: string): unknown[] {
  return routesOf(config, watch).map((route) => route.pipeline)
}

function names(config: Table, pipeline = 'video'): string[] {
  return stagesOf(config, pipeline).map(stageName)
}

function edgeBetween(config: Table, source: string, target: string) {
  return toGraph(config, builtIns()).edges.find((edge) => edge.source === source && edge.target === target)
}

function nodeOf(config: Table, id: string) {
  return toGraph(config, builtIns()).nodes.find((node) => node.id === id)
}

describe('drawing', () => {
  // @behavior PGE-001
  it('draws a pipeline as its stages in order', () => {
    const config = read(CONFIG)

    expect(nodeOf(config, stageId('video', 0))?.parentId).toBe(pipelineId('video'))
    expect(edgeBetween(config, stageId('video', 0), stageId('video', 1))).toBeDefined()
    expect(edgeBetween(config, stageId('video', 1), stageId('video', 2))).toBeDefined()
    expect(names(config)).toEqual(['filter', 'format', 'strip'])
  })

  // @behavior PGE-002
  it('joins a watch to the pipelines of its routes, in order', () => {
    const config = read(CONFIG)

    const [first, second] = [nodeOf(config, routeId('series', 0))!, nodeOf(config, routeId('series', 1))!]
    expect([first.parentId, second.parentId]).toEqual([watchId('series'), watchId('series')])
    expect(first.position.y).toBeLessThan(second.position.y)
    expect(edgeBetween(config, routeId('series', 0), pipelineId('video'))).toBeDefined()
    expect(edgeBetween(config, routeId('series', 1), pipelineId('subtitle'))).toBeDefined()
    const y = (id: string) => nodeOf(config, id)!.position.y
    expect(y(pipelineId('video'))).toBeLessThan(y(pipelineId('subtitle')))
  })

  // @behavior PGE-003
  it('joins a watch without its own routes to the default ones', () => {
    const config = read(CONFIG)

    expect(edgeBetween(config, routeId('movies', 0), pipelineId('video'))).toBeDefined()
    expect(nodeOf(config, routeId('movies', 0))?.data.followed).toBe(true)
    expect(nodeOf(config, routeId('movies', 1))).toBeUndefined()
  })

  // @behavior PGE-056
  it('runs the stages of a pipeline inside it from left to right', () => {
    const { nodes } = toGraph(read(CONFIG))
    const column = ['stage:video:0', 'stage:video:1', 'stage:video:2'].map((id) => nodes.find((node) => node.id === id)!)

    expect(column.every((node) => node.parentId === pipelineId('video'))).toBe(true)
    expect(new Set(column.map(({ position }) => position.y)).size).toBe(1)
    const xs = column.map(({ position }) => position.x)
    expect(xs).toEqual([...xs].sort((a, b) => a - b))
    expect(new Set(xs).size).toBe(column.length)
  })

  // @behavior PGE-073
  it('joins a route to the target it moves to', () => {
    const config = read(CONFIG)

    expect(nodeOf(config, targetId('dst'))).toBeDefined()
    expect(edgeBetween(config, routeId('series', 0), targetId('dst'))).toBeDefined()
  })

  // @behavior PGE-074
  it('joins a rejected route apart from its route', () => {
    const config = read(`${CONFIG}
[pipeline.fallback]
stages = ["strip"]

[target.conflict]
path = "/conflict"

[watch.clash]
source = "/clash"
routes = [{ pipeline = "video", rejected = { pipeline = "fallback", move = "conflict" } }]
`)

    const dashed = (target: string) => edgeBetween(config, routeId('clash', 0), target)?.style?.strokeDasharray
    expect(dashed(pipelineId('fallback'))).toBeDefined()
    expect(dashed(targetId('conflict'))).toBeDefined()
    expect(edgeBetween(config, routeId('clash', 0), pipelineId('video'))?.style).toBeUndefined()
  })

  // @behavior PGE-075
  it('draws a built-in pipeline a route names locked, with its stages', () => {
    const config = read('[watch.series]\nsource = "/src"\nroutes = [{ pipeline = "series-video" }]\n')

    const pipeline = nodeOf(config, pipelineId('series-video'))!
    const stage = nodeOf(config, stageId('series-video', 0))!
    expect(pipeline.data.locked).toBe(true)
    expect(pipeline.deletable).toBe(false)
    expect(stage.data.label).toBe(stageName(stagesOf({ pipeline: builtIns() }, 'series-video')[0]))
    expect([stage.draggable, stage.deletable]).toEqual([false, false])
  })

  // @behavior PGE-076
  it('draws a pipeline defined under a built-in name as its own', () => {
    const config = read('[pipeline.series-video]\nstages = ["strip"]\n\n[watch.series]\nsource = "/src"\nroutes = [{ pipeline = "series-video" }]\n')

    expect(nodeOf(config, pipelineId('series-video'))?.data.locked).toBe(false)
    expect(nodeOf(config, stageId('series-video', 0))?.data.label).toBe('strip')
    expect(nodeOf(config, stageId('series-video', 1))).toBeUndefined()
  })

  // @behavior PGE-077
  it('adds no stage dropped on a built-in pipeline', () => {
    const config = read('[watch.series]\nsource = "/src"\nroutes = [{ pipeline = "series-video" }]\n')
    const { nodes } = toGraph(config, builtIns())
    const pipeline = nodes.find((node) => node.id === pipelineId('series-video'))!

    expect(dropAt(nodes, { x: pipeline.position.x + 50, y: pipeline.position.y + 100 })).toBeNull()
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
    const moved = laidOut(toGraph(config).nodes, { [pipelineId('subtitle')]: { x: 1200, y: 900 } })

    expect(dropAt(moved, { x: 1600, y: 950 })).toEqual({ pipeline: 'subtitle', index: 1 })
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

  // @behavior PGE-088
  it('draws the routes of a watch whose order changed in their order', () => {
    const before = read(CONFIG)
    const layout = { [routeId('series', 1)]: { x: 40, y: 300 } }

    const after = moveRoute(before, 'series', 1, -1)
    const kept = relaid(layout, before, after)

    const ordered = toGraph(after).nodes.filter((node) => node.id.startsWith('route:series:'))
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

    const at = absolute(nodes, second)

    expect(dropAt(nodes, { x: at.x + 10, y: at.y })).toEqual({ pipeline: 'video', index: 1 })
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

describe('routes and targets', () => {
  // @behavior PGE-078
  it('claims first with a route moved above another', () => {
    const config = read(CONFIG)
    const { nodes } = toGraph(config, builtIns())
    const video = nodes.find((node) => node.id === routeId('series', 0))!
    const subtitle = nodes.find((node) => node.id === routeId('series', 1))!

    const index = routeAt(nodes.filter((node) => node.id !== subtitle.id), 'series', { x: video.position.x, y: video.position.y - 1 })!
    const moved = moveRoute(config, 'series', 1, index - 1)

    expect(pipelinesOf(moved, 'series')).toEqual(['subtitle', 'video'])
  })

  // @behavior PGE-079
  it('moves a route joined to a target there', () => {
    const config = routeTo(read(CONFIG), 'movies', 0, 'dst')

    expect(routesOf(config, 'movies')).toEqual([{ pipeline: 'video', move: 'dst' }])
  })

  // @behavior PGE-080
  it('leaves the other routes in order when one is removed', () => {
    const config = removeRoute(joinPipeline(read(CONFIG), 'series', 'extra'), 'series', 1)

    expect(pipelinesOf(config, 'series')).toEqual(['video', 'extra'])
  })

  // @behavior PGE-081
  it('moves no route to a removed target', () => {
    const config = read(`${CONFIG}
[watch.clash]
source = "/clash"
routes = [{ pipeline = "video", move = "dst", rejected = { move = "dst" } }, { pipeline = "subtitle", rejected = { pipeline = "subtitle", move = "dst" } }]
`)
    const withDefault = setIn(config, ['default', 'routes'], [{ pipeline: 'video', move: 'dst' }])

    const removed = removeTarget(withDefault, 'dst')

    expect(asTable(removed.target).dst).toBeUndefined()
    expect(asTable(removed.default).routes).toEqual([{ pipeline: 'video' }])
    expect(routesOf(removed, 'series')).toEqual([{ pipeline: 'video' }, { pipeline: 'subtitle' }])
    expect(routesOf(removed, 'clash')).toEqual([{ pipeline: 'video' }, { pipeline: 'subtitle', rejected: { pipeline: 'subtitle' } }])
  })

  // @behavior PGE-082
  it('renames in place with a route whose move joint is removed', () => {
    const config = routeTo(read(CONFIG), 'series', 0, undefined)

    expect(routesOf(config, 'series')[0]).toEqual({ pipeline: 'video' })
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
  // @behavior PGE-086
  it('copies a built-in pipeline overridden in a folder from the core', () => {
    const global = read('[watch.series]\nsource = "/src"\nroutes = [{ pipeline = "series-video" }]\n')

    const result = overridden({}, routable(global, builtIns()), 'series-video')

    expect(stagesOf(result, 'series-video')).toEqual(stagesOf({ pipeline: builtIns() }, 'series-video'))
  })

  // @behavior PGE-085
  it('makes a copied built-in pipeline the configuration\'s own', () => {
    const config = read('[watch.series]\nsource = "/src"\nroutes = [{ pipeline = "series-video" }]\n')

    const copied = ownPipeline(config, builtIns(), 'series-video')

    expect(stagesOf(copied, 'series-video')).toEqual(stagesOf({ pipeline: builtIns() }, 'series-video'))
    expect(stagesOf(copied, 'series-video').length).toBeGreaterThan(0)
  })

  // @behavior PGE-063
  it('copies the global pipeline in, keeping what the folder holds', () => {
    const folder = read('vars = { show = "Alpha" }\n\n[pipeline.subtitle]\nstages = ["strip"]\n\n[pipeline.video]\nstages = []\n')

    const result = overridden(folder, routable(read(CONFIG), builtIns()), 'video')

    expect(result.vars).toEqual({ show: 'Alpha' })
    expect(names(result, 'subtitle')).toEqual(['strip'])
    expect(names(result)).toEqual(names(read(CONFIG)))
  })
})

describe('moving a stage across pipelines', () => {
  // @behavior PGE-102
  it('takes a stage out of its pipeline and into the other', () => {
    const config = transferStage(read(CONFIG), 'video', 1, 'subtitle', 0)

    expect(names(config)).toEqual(['filter', 'strip'])
    expect(names(config, 'subtitle')).toEqual(['format', 'strip'])
  })

  it('places nothing for a route dropped away from its watch', () => {
    const { nodes } = toGraph(read(CONFIG), builtIns())

    expect(routeAt(nodes, 'series', { x: 600, y: 50 })).toBeNull()
  })
})
