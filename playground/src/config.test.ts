import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import {
  type Table,
  STAGE_TEMPLATES,
  addStage,
  download,
  moveStage,
  newStage,
  readStage,
  removeStage,
  replaceStage,
  stageName,
  stagesOf,
} from './config'
import { check, read, render } from './core'
import { pipelineId, stageId, toGraph, watchId } from './graph'

const CONFIG = `
[default]
pipelines = ["video"]

[pipeline.video]
stages = [{ filter = { ext = ["mkv"] } }, { format = "{show}" }, "move"]

[pipeline.subtitle]
stages = ["move"]

[watch.series]
source = "/src"
target = "/dst"
pipelines = ["video", "subtitle"]
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
    expect(names(config)).toEqual(['filter', 'format', 'move'])
  })

  // @behavior PGE-002
  it('joins a watch to the pipelines it lists, in order', () => {
    const config = read(CONFIG)

    expect(edgeBetween(config, watchId('series'), pipelineId('video'))?.label).toBe('1')
    expect(edgeBetween(config, watchId('series'), pipelineId('subtitle'))?.label).toBe('2')
  })

  // @behavior PGE-003
  it('joins a watch without its own list to the default pipelines', () => {
    const config = read(CONFIG)

    expect(edgeBetween(config, watchId('movies'), pipelineId('video'))).toBeDefined()
    expect(edgeBetween(config, watchId('movies'), pipelineId('subtitle'))).toBeUndefined()
  })
})

describe('editing', () => {
  const pipeline = (stages: string) => read(`[pipeline.video]\nstages = ${stages}`)

  // @behavior PGE-004
  it('adds a stage after the others', () => {
    const config = addStage(pipeline('[{ filter = { ext = ["mkv"] } }, "move"]'), 'video', newStage('format', read))

    expect(names(config)).toEqual(['filter', 'move', 'format'])
  })

  // @behavior PGE-005
  it('writes a stage moved earlier earlier', () => {
    const config = moveStage(pipeline('[{ filter = { ext = ["mkv"] } }, "move", { format = "{show}" }]'), 'video', 2, -1)

    expect(names(config)).toEqual(['filter', 'format', 'move'])
  })

  // @behavior PGE-006
  it('leaves the others in order when a stage is removed', () => {
    const config = removeStage(pipeline('[{ filter = { ext = ["mkv"] } }, { format = "{show}" }, "move"]'), 'video', 1)

    expect(names(config)).toEqual(['filter', 'move'])
  })

  // @behavior PGE-007
  it('writes edited settings as the value of the stage', () => {
    const before = pipeline('[{ filter = { ext = ["mkv"] } }, "move"]')

    const config = replaceStage(before, 'video', 0, readStage('{ filter = { ext = ["mp4"] } }', read))

    expect(stagesOf(config, 'video')).toEqual([{ filter: { ext: ['mp4'] } }, 'move'])
  })

  // @behavior PGE-008
  it('writes a stage without settings as its name', () => {
    const config = addStage(pipeline('[]'), 'video', newStage('move', read))

    expect(render(config)).toContain('stages = ["move"]')
  })

  it('starts every kind of stage with settings the core accepts', () => {
    for (const kind of Object.keys(STAGE_TEMPLATES)) {
      const config = addStage(read('[pipeline.p]\nstages = []'), 'p', newStage(kind, read))
      const text = `${render(config)}\n[watch.w]\nsource = "/src"\npipelines = ["p"]\n`

      expect(() => check('global', text), kind).not.toThrow()
    }
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
