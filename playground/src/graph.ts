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

export function watchId(name: string): string {
  return `watch:${name}`
}

export function pipelineId(name: string): string {
  return `pipeline:${name}`
}

export function stageId(pipeline: string, index: number): string {
  return `stage:${pipeline}:${index}`
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
      data: {
        label: name,
        detail: typeof watch.source === 'string' ? watch.source : undefined,
        selected: { kind: 'watch', name },
      },
    })
    pipelinesOf(config, name).forEach((pipeline, order) => {
      edges.push({
        id: `${watchId(name)}->${pipelineId(pipeline)}`,
        source: watchId(name),
        target: pipelineId(pipeline),
        label: String(order + 1),
      })
    })
  })

  pipelines.forEach((pipeline, row) => {
    const y = row * ROW
    nodes.push({
      id: pipelineId(pipeline),
      type: 'pipeline',
      position: { x: COLUMN, y },
      data: { label: pipeline, selected: { kind: 'pipeline', name: pipeline } },
    })
    let previous = pipelineId(pipeline)
    stagesOf(config, pipeline).forEach((stage, index) => {
      const id = stageId(pipeline, index)
      nodes.push({
        id,
        type: 'stage',
        position: { x: COLUMN * (index + 2), y },
        data: {
          label: stageName(stage),
          detail: stageDetail(stage),
          selected: { kind: 'stage', pipeline, index },
        },
      })
      edges.push({ id: `${previous}->${id}`, source: previous, target: id })
      previous = id
    })
  })

  return { nodes, edges }
}
