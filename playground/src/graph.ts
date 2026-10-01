import type { Edge, Node } from '@xyflow/react'

import { type Stage, type Table, asTable, pipelinesOf, stageName, stagesOf } from './config'

export type Selected =
  | { kind: 'watch'; name: string }
  | { kind: 'pipeline'; name: string }
  | { kind: 'stage'; pipeline: string; index: number }

export interface NodeData extends Record<string, unknown> {
  label: string
  detail?: string
  selected: Selected
}

const COLUMN = 240
const ROW = 110
const FIRST_STAGE = COLUMN * 2

export function watchId(name: string): string {
  return `watch:${name}`
}

export function pipelineId(name: string): string {
  return `pipeline:${name}`
}

export function stageId(pipeline: string, index: number): string {
  return `stage:${pipeline}:${index}`
}

/** A joint of a watch to a pipeline; one the watch takes from the default is drawn dashed. */
export interface Joint extends Record<string, unknown> {
  watch: string
  pipeline: string
}

/** Where a stage dropped at a point of the drawing goes: the pipeline on that row, before the stage
 * nearest the point. */
export function dropAt(config: Table, point: { x: number; y: number }): { pipeline: string; index: number } | null {
  const pipelines = Object.keys(asTable(config.pipeline))
  const pipeline = pipelines[Math.round(point.y / ROW)]
  if (pipeline === undefined) return null
  const index = Math.round((point.x - FIRST_STAGE) / COLUMN)
  return { pipeline, index: Math.min(Math.max(index, 0), stagesOf(config, pipeline).length) }
}

function stageDetail(stage: Stage): string | undefined {
  if (typeof stage === 'string') return undefined
  const settings = stage[stageName(stage)]
  return typeof settings === 'string' || typeof settings === 'number'
    ? String(settings)
    : undefined
}

/**
 * The drawing of a configuration: each watch on the left, joined in order to the pipelines it runs,
 * and each pipeline followed by its stages from left to right.
 */
export function toGraph(config: Table): { nodes: Node<NodeData>[]; edges: Edge[] } {
  const nodes: Node<NodeData>[] = []
  const edges: Edge[] = []
  const pipelines = Object.keys(asTable(config.pipeline))
  const watches = Object.keys(asTable(config.watch))

  watches.forEach((name, row) => {
    const watch = asTable(asTable(config.watch)[name])
    nodes.push({
      id: watchId(name),
      type: 'watch',
      position: { x: 0, y: row * ROW },
      draggable: false,
      data: {
        label: name,
        detail: typeof watch.source === 'string' ? watch.source : undefined,
        selected: { kind: 'watch', name },
      },
    })
    const followsDefault = watch.pipelines === undefined
    pipelinesOf(config, name).forEach((pipeline, order) => {
      edges.push({
        id: `${watchId(name)}->${pipelineId(pipeline)}`,
        source: watchId(name),
        target: pipelineId(pipeline),
        label: String(order + 1),
        style: followsDefault ? { strokeDasharray: '6 4' } : undefined,
        data: { watch: name, pipeline } satisfies Joint,
      })
    })
  })

  pipelines.forEach((pipeline, row) => {
    const y = row * ROW
    nodes.push({
      id: pipelineId(pipeline),
      type: 'pipeline',
      position: { x: COLUMN, y },
      draggable: false,
      data: { label: pipeline, selected: { kind: 'pipeline', name: pipeline } },
    })
    let previous = pipelineId(pipeline)
    stagesOf(config, pipeline).forEach((stage, index) => {
      const id = stageId(pipeline, index)
      nodes.push({
        id,
        type: 'stage',
        position: { x: FIRST_STAGE + COLUMN * index, y },
        data: {
          label: stageName(stage),
          detail: stageDetail(stage),
          selected: { kind: 'stage', pipeline, index },
        },
      })
      edges.push({ id: `${previous}->${id}`, source: previous, target: id, deletable: false, selectable: false })
      previous = id
    })
  })

  return { nodes, edges }
}
