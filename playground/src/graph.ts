import type { Edge, Node, XYPosition } from '@xyflow/react'

import { type Stage, type Table, asTable, routesOf, stageName, stagesOf, targetsOf } from './config'
import { claims } from './steps'

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
  /** A stage that stands for its pipeline's claim, being a filter with only filters before it. */
  claims?: boolean
}

/** How wide a watch is drawn, and a route or a stage inside a group; what a group holds is inset by `PAD`. */
const GROUP = 200
const CHILD = GROUP - 24
const PAD = 12
const HEADER = 44
// A route sits below the one before it in its watch; a stage sits right of the one before it in its pipeline.
const STEP = 64
const STAGE_GAP = 16
const STAGE_STEP = CHILD + STAGE_GAP
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

/** A joint of a route to the target it moves to. */
export interface Move extends Record<string, unknown> {
  watch: string
  index: number
}

const REJECTED = { strokeDasharray: '4 4', stroke: 'var(--destructive)' }
const FOLLOWED = { strokeDasharray: '6 4' }

function heightOf(count: number): number {
  return HEADER + STEP * count + PAD
}

/** A pipeline is one row as high as a single stage, and as wide as its stages laid side by side. */
const PIPELINE_HEIGHT = heightOf(1)

function widthOf(count: number): number {
  return Math.max(GROUP, PAD + STAGE_STEP * count - STAGE_GAP + PAD)
}

function below(index: number): XYPosition {
  return { x: PAD, y: HEADER + STEP * index }
}

function beside(index: number): XYPosition {
  return { x: PAD + STAGE_STEP * index, y: HEADER }
}

/** Where a node is drawn on the canvas: its own position, or its position inside the node holding it. */
export function absolute(nodes: Node<NodeData>[], node: Node<NodeData>): XYPosition {
  const parent = node.parentId ? nodes.find((one) => one.id === node.parentId) : undefined
  return parent ? { x: parent.position.x + node.position.x, y: parent.position.y + node.position.y } : node.position
}

/** Where a stage dropped at a point of the drawing goes: the pipeline drawn across that point's row, after each of
 * its stages whose middle is left of the point; none on a built-in pipeline. `nodes` are as drawn, without the
 * stage being moved. */
export function dropAt(nodes: Node<NodeData>[], point: XYPosition): { pipeline: string; index: number } | null {
  const middle = (node: Node<NodeData>) => node.position.y + PIPELINE_HEIGHT / 2
  const row = nodes
    .filter((node) => node.type === 'pipeline' && Math.abs(middle(node) - point.y) < (PIPELINE_HEIGHT + GAP) / 2)
    .sort((a, b) => Math.abs(middle(a) - point.y) - Math.abs(middle(b) - point.y))[0]
  if (row === undefined || row.data.selected.kind !== 'pipeline' || row.data.locked) return null
  const pipeline = row.data.selected.name
  const index = nodes.filter((node) => {
    const one = node.data.selected
    return one.kind === 'stage' && one.pipeline === pipeline && absolute(nodes, node).x + CHILD / 2 <= point.x
  }).length
  return { pipeline, index }
}

/** Where a route dropped at a place inside its watch goes: after each other route of the watch whose middle is
 * above it; none when it was dropped away from the watch's column. `nodes` are as drawn, without the route being
 * moved. */
export function routeAt(nodes: Node<NodeData>[], watch: string, at: XYPosition): number | null {
  if (Math.abs(at.x - PAD) > CHILD / 2) return null
  return nodes.filter((node) => {
    const one = node.data.selected
    return one.kind === 'route' && one.watch === watch && node.position.y + STEP / 2 <= at.y
  }).length
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
 * The drawing of a configuration, read from left to right: the watches stacked in the first column, each holding its
 * routes in the order they claim; then each pipeline as a row holding its stages in order; then the targets. A
 * route leads to its pipeline and to the target it moves to; where its rejected route goes is joined apart. A
 * built-in pipeline is drawn when a route names it, locked.
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
  // A pipeline whose routes move to different targets labels each joint with its route, since they share one end.
  const movesOf = new Map<string, Set<unknown>>()
  for (const route of watches.flatMap((name) => routesOf(config, name))) {
    if (typeof route.pipeline !== 'string' || typeof route.move !== 'string') continue
    movesOf.set(route.pipeline, (movesOf.get(route.pipeline) ?? new Set()).add(route.move))
  }
  const sharedTargets = new Set([...movesOf].filter(([, moves]) => moves.size > 1).map(([pipeline]) => pipeline))
  // Rows are laid out in turn, each below the one before it; the pipelines start right of the watches and the
  // targets right of the widest pipeline.
  const tops = (heights: number[]) => heights.map((_, index) => heights.slice(0, index).reduce((sum, one) => sum + one + GAP / 2, 0))
  const watchTops = tops(watches.map((name) => heightOf(routesOf(config, name).length)))
  const pipelineTops = tops(pipelines.map(() => PIPELINE_HEIGHT))
  const pipelinesLeft = GROUP + GAP
  const targetsLeft = pipelinesLeft + Math.max(GROUP, ...pipelines.map(({ stages }) => widthOf(stages.length))) + GAP

  watches.forEach((name, row) => {
    const watch = asTable(asTable(config.watch)[name])
    const routes = routesOf(config, name)
    const followed = watch.routes === undefined
    nodes.push({
      id: watchId(name),
      type: 'watch',
      position: { x: 0, y: watchTops[row] },
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
        position: below(index),
        style: { width: CHILD },
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
      // Effects follow the pipeline that planned the files, so their joints leave the end of its row rather than
      // crossing the stages; a route whose pipeline is not drawn keeps them.
      const from = (planned: string) => (drawn.has(planned) ? { source: pipelineId(planned), sourceHandle: 'end' } : { source: id, sourceHandle: 'effects' })
      const named = sharedTargets.has(pipeline) ? `${name} ${index + 1}` : undefined
      if (typeof route.move === 'string' && targets.includes(route.move)) {
        edges.push({ id: `${id}->${targetId(route.move)}`, ...from(pipeline), target: targetId(route.move), label: named,
          style: followed ? FOLLOWED : undefined, data: { watch: name, index } satisfies Move })
      }
      const rejected = asTable(route.rejected)
      const refusedBy = typeof rejected.pipeline === 'string' && drawn.has(rejected.pipeline) ? rejected.pipeline : pipeline
      if (typeof rejected.pipeline === 'string' && drawn.has(rejected.pipeline)) {
        edges.push({ id: `${id}->${pipelineId(rejected.pipeline)}:rejected`, ...from(pipeline), target: pipelineId(rejected.pipeline),
          label: named, style: REJECTED, deletable: false })
      }
      if (typeof rejected.move === 'string' && targets.includes(rejected.move)) {
        edges.push({ id: `${id}->${targetId(rejected.move)}:rejected`, ...from(refusedBy), target: targetId(rejected.move),
          label: named, style: REJECTED, deletable: false })
      }
    })
  })

  targets.forEach((name, row) => {
    nodes.push({
      id: targetId(name),
      type: 'target',
      position: { x: targetsLeft, y: row * TARGET_ROW },
      data: { label: name, detail: targetsOf(config)[name], selected: { kind: 'target', name } },
    })
  })

  pipelines.forEach(({ name: pipeline, stages, locked }, row) => {
    nodes.push({
      id: pipelineId(pipeline),
      type: 'pipeline',
      position: { x: pipelinesLeft, y: pipelineTops[row] },
      style: { width: widthOf(stages.length), height: PIPELINE_HEIGHT },
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
        position: beside(index),
        style: { width: CHILD },
        draggable: !locked,
        deletable: !locked,
        data: {
          label: stageName(stage),
          detail: stageDetail(stage),
          locked,
          claims: claims(stages, index),
          selected: { kind: 'stage', pipeline, index },
        },
      })
      if (previous) edges.push({ id: `${previous}->${id}`, source: previous, target: id, deletable: false, selectable: false })
      previous = id
    })
  })

  return { nodes, edges }
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
