import type { Edge, Node, XYPosition } from '@xyflow/react'

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
// A stage sits below the one before it, so a long pipeline grows down instead of shrinking the drawing.
const STEP = 80

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

/** Where a stage dropped at a point of the drawing goes: the pipeline drawn in that column, after each of its
 * stages whose middle is above the point. `nodes` are as drawn, without the stage being moved. */
export function dropAt(nodes: Node<NodeData>[], point: XYPosition): { pipeline: string; index: number } | null {
  const column = nodes
    .filter((node) => node.type === 'pipeline' && Math.abs(node.position.x - point.x) < COLUMN / 2)
    .sort((a, b) => Math.abs(a.position.x - point.x) - Math.abs(b.position.x - point.x))[0]
  if (column === undefined || column.data.selected.kind !== 'pipeline') return null
  const pipeline = column.data.selected.name
  const index = nodes.filter((node) => {
    const one = node.data.selected
    return one.kind === 'stage' && one.pipeline === pipeline && node.position.y + STEP / 2 <= point.y
  }).length
  return { pipeline, index }
}

function stageDetail(stage: Stage): string | undefined {
  if (typeof stage === 'string') return undefined
  const settings = stage[stageName(stage)]
  return typeof settings === 'string' || typeof settings === 'number'
    ? String(settings)
    : undefined
}

/**
 * The drawing of a configuration, read from the top down: the watches in the top row, joined in order to
 * the pipelines they run in the row below, each pipeline with its stages stacked under it.
 */
export function toGraph(config: Table): { nodes: Node<NodeData>[]; edges: Edge[] } {
  const nodes: Node<NodeData>[] = []
  const edges: Edge[] = []
  const watches = Object.keys(asTable(config.watch))
  // Pipelines stand in the order the watches run them, so joints do not cross; unlisted ones come last.
  const listed = watches.flatMap((name) => pipelinesOf(config, name))
  const pipelines = [...new Set([...listed, ...Object.keys(asTable(config.pipeline))])]
    .filter((name) => name in asTable(config.pipeline))

  watches.forEach((name, column) => {
    const watch = asTable(asTable(config.watch)[name])
    nodes.push({
      id: watchId(name),
      type: 'watch',
      position: { x: column * COLUMN, y: 0 },
      data: {
        label: name,
        detail: typeof watch.source === 'string' ? watch.source : undefined,
        selected: { kind: 'watch', name },
      },
    })
    const followsDefault = watch.routes === undefined
    pipelinesOf(config, name).forEach((pipeline, order) => {
      edges.push({
        id: `${watchId(name)}->${pipelineId(pipeline)}`,
        type: 'joint',
        source: watchId(name),
        target: pipelineId(pipeline),
        label: String(order + 1),
        style: followsDefault ? { strokeDasharray: '6 4' } : undefined,
        data: { watch: name, pipeline } satisfies Joint,
      })
    })
  })

  pipelines.forEach((pipeline, column) => {
    const x = COLUMN * column
    nodes.push({
      id: pipelineId(pipeline),
      type: 'pipeline',
      position: { x, y: ROW },
      data: { label: pipeline, selected: { kind: 'pipeline', name: pipeline } },
    })
    let previous = pipelineId(pipeline)
    stagesOf(config, pipeline).forEach((stage, index) => {
      const id = stageId(pipeline, index)
      nodes.push({
        id,
        type: 'stage',
        position: { x, y: ROW + STEP * (index + 1) },
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

/** The nodes the view opens on: the watches, the pipelines and their first stages, at full scale. A longer
 * pipeline goes on below, reached by panning, rather than shrinking every node to fit. */
export function head(nodes: Node<NodeData>[]): Node<NodeData>[] {
  return nodes.filter((node) => node.position.y < ROW + STEP * 5)
}

/** Where nodes were left by hand, by id; the configuration's order places every other node. */
export type Layout = Record<string, XYPosition>

export function laidOut(nodes: Node<NodeData>[], layout: Layout): Node<NodeData>[] {
  return nodes.map((node) => (layout[node.id] ? { ...node, position: layout[node.id] } : node))
}

/** The layout kept across a change of the configuration. A stage is known by its place in its pipeline, so a
 * pipeline whose stages changed order draws them in order again rather than at another stage's spot. */
export function relaid(layout: Layout, before: Table, after: Table): Layout {
  const pipelines = new Set([...Object.keys(asTable(before.pipeline)), ...Object.keys(asTable(after.pipeline))])
  const changed = [...pipelines].filter((pipeline) =>
    stagesOf(before, pipeline).map(stageName).join() !== stagesOf(after, pipeline).map(stageName).join())
  if (changed.length === 0) return layout
  return Object.fromEntries(Object.entries(layout).filter(([id]) =>
    !changed.some((pipeline) => id.startsWith(`stage:${pipeline}:`))))
}
