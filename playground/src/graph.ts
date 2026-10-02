import type { Edge, Node, XYPosition } from '@xyflow/react'

import { type Stage, type Table, asTable, routesOf, stageName, stagesOf, targetsOf } from './config'

export type Selected =
  | { kind: 'watch'; name: string }
  | { kind: 'route'; watch: string; index: number }
  | { kind: 'pipeline'; name: string }
  | { kind: 'stage'; pipeline: string; index: number }
  | { kind: 'target'; name: string }

export interface NodeData extends Record<string, unknown> {
  label: string
  detail?: string
  selected: Selected
  /** A built-in pipeline or one of its stages, which the canvas does not change. */
  locked?: boolean
  /** A route a watch follows from the default rather than lists itself. */
  followed?: boolean
}

const COLUMN = 240
/** How wide a watch or a pipeline is drawn; what it holds is drawn inside, inset by `PAD`. */
const GROUP = 200
const PAD = 12
const HEADER = 44
// A route or a stage sits below the one before it, so a long list grows down instead of shrinking the drawing.
const STEP = 64
const GAP = 80
const TARGET_ROW = 80

export function watchId(name: string): string {
  return `watch:${name}`
}

export function routeId(watch: string, index: number): string {
  return `route:${watch}:${index}`
}

export function pipelineId(name: string): string {
  return `pipeline:${name}`
}

export function stageId(pipeline: string, index: number): string {
  return `stage:${pipeline}:${index}`
}

export function targetId(name: string): string {
  return `target:${name}`
}

/** A joint of a route to its pipeline; one the watch follows from the default is drawn dashed. */
export interface Joint extends Record<string, unknown> {
  watch: string
  pipeline: string
}

const REJECTED = { strokeDasharray: '4 4', stroke: 'var(--destructive)' }
const FOLLOWED = { strokeDasharray: '6 4' }

function heightOf(count: number): number {
  return HEADER + STEP * count + PAD
}

function inside(index: number): XYPosition {
  return { x: PAD, y: HEADER + STEP * index }
}

/** Where a node is drawn on the canvas: its own position, or its position inside the node holding it. */
export function absolute(nodes: Node<NodeData>[], node: Node<NodeData>): XYPosition {
  const parent = node.parentId ? nodes.find((one) => one.id === node.parentId) : undefined
  return parent ? { x: parent.position.x + node.position.x, y: parent.position.y + node.position.y } : node.position
}

/** Where a stage dropped at a point of the drawing goes: the pipeline drawn over that point's column, after each
 * of its stages whose middle is above the point; none on a built-in pipeline. `nodes` are as drawn, without the
 * stage being moved. */
export function dropAt(nodes: Node<NodeData>[], point: XYPosition): { pipeline: string; index: number } | null {
  const middle = (node: Node<NodeData>) => node.position.x + GROUP / 2
  const column = nodes
    .filter((node) => node.type === 'pipeline' && Math.abs(middle(node) - point.x) < COLUMN / 2)
    .sort((a, b) => Math.abs(middle(a) - point.x) - Math.abs(middle(b) - point.x))[0]
  if (column === undefined || column.data.selected.kind !== 'pipeline' || column.data.locked) return null
  const pipeline = column.data.selected.name
  const index = nodes.filter((node) => {
    const one = node.data.selected
    return one.kind === 'stage' && one.pipeline === pipeline && absolute(nodes, node).y + STEP / 2 <= point.y
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

/** The pipeline a name stands for: the configuration's own, or the built-in one it does not replace. */
function pipelineOf(config: Table, builtIns: Table, name: string): { stages: Stage[]; locked: boolean } | null {
  if (name in asTable(config.pipeline)) return { stages: stagesOf(config, name), locked: false }
  if (name in builtIns) return { stages: stagesOf({ pipeline: builtIns }, name), locked: true }
  return null
}

/** The pipelines a route runs: its own, then its rejected route's. */
function pipelinesRouted(route: Table): string[] {
  return [route.pipeline, asTable(route.rejected).pipeline].filter((name): name is string => typeof name === 'string')
}

/**
 * The drawing of a configuration, read from the top down: each watch holding its routes in the order they claim,
 * the targets beside them, and below them each pipeline holding its stages in order. A route leads to its
 * pipeline and to the target it moves to; where its rejected route goes is joined apart. A built-in pipeline
 * is drawn when a route names it, locked.
 */
export function toGraph(config: Table, builtIns: Table = {}): { nodes: Node<NodeData>[]; edges: Edge[] } {
  const nodes: Node<NodeData>[] = []
  const edges: Edge[] = []
  const watches = Object.keys(asTable(config.watch))
  const targets = Object.keys(targetsOf(config))
  // Pipelines stand in the order the watches route them, so joints do not cross; unrouted ones come last.
  const routed = watches.flatMap((name) => routesOf(config, name).flatMap(pipelinesRouted))
  const pipelines = [...new Set([...routed, ...Object.keys(asTable(config.pipeline))])]
    .flatMap((name) => {
      const pipeline = pipelineOf(config, builtIns, name)
      return pipeline ? [{ name, ...pipeline }] : []
    })
  const drawn = new Set(pipelines.map(({ name }) => name))
  const top = Math.max(0, ...watches.map((name) => heightOf(routesOf(config, name).length))) + GAP

  watches.forEach((name, column) => {
    const watch = asTable(asTable(config.watch)[name])
    const routes = routesOf(config, name)
    const followed = watch.routes === undefined
    nodes.push({
      id: watchId(name),
      type: 'watch',
      position: { x: column * COLUMN, y: 0 },
      style: { width: GROUP, height: heightOf(routes.length) },
      data: {
        label: name,
        detail: typeof watch.source === 'string' ? watch.source : undefined,
        selected: { kind: 'watch', name },
      },
    })
    routes.forEach((route, index) => {
      const id = routeId(name, index)
      const pipeline = typeof route.pipeline === 'string' ? route.pipeline : ''
      nodes.push({
        id,
        type: 'route',
        parentId: watchId(name),
        extent: 'parent',
        position: inside(index),
        style: { width: GROUP - PAD * 2 },
        deletable: false,
        data: { label: `${index + 1} · ${pipeline}`, followed, selected: { kind: 'route', watch: name, index } },
      })
      if (drawn.has(pipeline)) {
        edges.push({
          id: `${id}->${pipelineId(pipeline)}`,
          type: 'joint',
          source: id,
          target: pipelineId(pipeline),
          style: followed ? FOLLOWED : undefined,
          data: { watch: name, pipeline } satisfies Joint,
        })
      }
      if (typeof route.move === 'string' && targets.includes(route.move)) {
        edges.push({ id: `${id}->${targetId(route.move)}`, source: id, sourceHandle: 'effects', target: targetId(route.move),
          style: followed ? FOLLOWED : undefined, deletable: false })
      }
      const rejected = asTable(route.rejected)
      if (typeof rejected.pipeline === 'string' && drawn.has(rejected.pipeline)) {
        edges.push({ id: `${id}->${pipelineId(rejected.pipeline)}:rejected`, source: id, target: pipelineId(rejected.pipeline),
          style: REJECTED, deletable: false })
      }
      if (typeof rejected.move === 'string' && targets.includes(rejected.move)) {
        edges.push({ id: `${id}->${targetId(rejected.move)}:rejected`, source: id, sourceHandle: 'effects',
          target: targetId(rejected.move), style: REJECTED, deletable: false })
      }
    })
  })

  targets.forEach((name, row) => {
    nodes.push({
      id: targetId(name),
      type: 'target',
      position: { x: watches.length * COLUMN, y: row * TARGET_ROW },
      deletable: false,
      data: { label: name, detail: targetsOf(config)[name], selected: { kind: 'target', name } },
    })
  })

  pipelines.forEach(({ name: pipeline, stages, locked }, column) => {
    nodes.push({
      id: pipelineId(pipeline),
      type: 'pipeline',
      position: { x: COLUMN * column, y: top },
      style: { width: GROUP, height: heightOf(stages.length) },
      deletable: !locked,
      data: { label: pipeline, locked, selected: { kind: 'pipeline', name: pipeline } },
    })
    let previous: string | null = null
    stages.forEach((stage, index) => {
      const id = stageId(pipeline, index)
      nodes.push({
        id,
        type: 'stage',
        parentId: pipelineId(pipeline),
        extent: 'parent',
        position: inside(index),
        style: { width: GROUP - PAD * 2 },
        draggable: !locked,
        deletable: !locked,
        data: {
          label: stageName(stage),
          detail: stageDetail(stage),
          locked,
          selected: { kind: 'stage', pipeline, index },
        },
      })
      if (previous) edges.push({ id: `${previous}->${id}`, source: previous, target: id, deletable: false, selectable: false })
      previous = id
    })
  })

  return { nodes, edges }
}

/** The nodes the view opens on: the watches, the targets and the first stages of each pipeline, at full scale. A
 * longer pipeline goes on below, reached by panning, rather than shrinking every node to fit. */
export function head(nodes: Node<NodeData>[]): Node<NodeData>[] {
  const first = 4
  return nodes.filter((node) => {
    const one = node.data.selected
    if (one.kind === 'stage') return one.index < first
    if (one.kind === 'pipeline') return !nodes.some((other) => other.parentId === node.id && other.data.selected.kind === 'stage'
      && other.data.selected.index >= first)
    return true
  })
}

/** Where nodes were left by hand, by id; the configuration's order places every other node. A node held by
 * another is kept by its place inside it. */
export type Layout = Record<string, XYPosition>

export function laidOut(nodes: Node<NodeData>[], layout: Layout): Node<NodeData>[] {
  return nodes.map((node) => (layout[node.id] ? { ...node, position: layout[node.id] } : node))
}

/** The layout kept across a change of the configuration. A stage or a route is known by its place in its list, so
 * a list whose order changed draws it in order again rather than at another one's spot. */
export function relaid(layout: Layout, before: Table, after: Table): Layout {
  const pipelines = new Set([...Object.keys(asTable(before.pipeline)), ...Object.keys(asTable(after.pipeline))])
  const watches = new Set([...Object.keys(asTable(before.watch)), ...Object.keys(asTable(after.watch))])
  const routed = (config: Table, watch: string) => routesOf(config, watch).map((route) => String(route.pipeline)).join()
  const changed = [
    ...[...pipelines].filter((pipeline) =>
      stagesOf(before, pipeline).map(stageName).join() !== stagesOf(after, pipeline).map(stageName).join())
      .map((pipeline) => `stage:${pipeline}:`),
    ...[...watches].filter((watch) => routed(before, watch) !== routed(after, watch)).map((watch) => `route:${watch}:`),
  ]
  if (changed.length === 0) return layout
  return Object.fromEntries(Object.entries(layout).filter(([id]) => !changed.some((prefix) => id.startsWith(prefix))))
}
