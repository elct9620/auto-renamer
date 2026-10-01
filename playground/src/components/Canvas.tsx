import {
  Background,
  type Connection,
  Controls,
  type Edge,
  Handle,
  type Node,
  type NodeProps,
  Position,
  ReactFlow,
  ReactFlowProvider,
  useEdgesState,
  useNodesState,
  useReactFlow,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { type DragEvent, useEffect, useMemo } from 'react'

import { cn } from '@/lib/utils'
import {
  type Table,
  insertStage,
  joinPipeline,
  moveStage,
  newStage,
  partPipeline,
  removePipeline,
  removeStage,
  setIn,
} from '../config'
import { read, stages } from '../core'
import { type Joint, type NodeData, type Selected, dropAt, toGraph } from '../graph'

/** The type a stage dragged from the palette carries its name under. */
export const STAGE_DRAG = 'application/x-auto-renamer-stage'

const tones = {
  watch: 'border-sky-500/60 bg-sky-50 dark:bg-sky-950',
  pipeline: 'border-amber-500/60 bg-amber-50 dark:bg-amber-950',
  stage: 'border-border bg-card cursor-grab',
}

function Box({ data, tone, selected, joins }: {
  data: NodeData
  tone: keyof typeof tones
  selected: boolean
  joins: 'source' | 'target' | null
}) {
  return (
    <div className={cn('grid min-w-36 gap-0.5 rounded-md border px-3 py-2 text-sm shadow-xs', tones[tone], selected && 'ring-2 ring-ring')}>
      <Handle type="target" position={Position.Left} isConnectable={joins === 'target'} />
      <strong className="font-medium">{data.label}</strong>
      {data.detail && <small className="max-w-48 truncate font-mono text-xs text-muted-foreground">{data.detail}</small>}
      <Handle type="source" position={Position.Right} isConnectable={joins === 'source'} />
    </div>
  )
}

// A watch is joined to pipelines by its handle; stages follow each other only by order.
const nodeTypes = {
  watch: ({ data, selected }: NodeProps<Node<NodeData>>) => <Box data={data} tone="watch" selected={selected} joins="source" />,
  pipeline: ({ data, selected }: NodeProps<Node<NodeData>>) => <Box data={data} tone="pipeline" selected={selected} joins="target" />,
  stage: ({ data, selected }: NodeProps<Node<NodeData>>) => <Box data={data} tone="stage" selected={selected} joins={null} />,
}

function isJoint(connection: Connection | Edge): boolean {
  return connection.source.startsWith('watch:') && connection.target.startsWith('pipeline:')
}

/** Removes what was deleted on the canvas: stages from the last, so earlier positions stay valid. */
function removeNodes(config: Table, nodes: Node[]): Table {
  const chosen = nodes.map((node) => (node.data as NodeData).selected)
  let next = config
  for (const one of chosen) {
    if (one.kind === 'watch') next = setIn(next, ['watch', one.name], undefined)
    if (one.kind === 'pipeline') next = removePipeline(next, one.name)
  }
  const stagesChosen = chosen.flatMap((one) => (one.kind === 'stage' ? [one] : []))
  for (const one of stagesChosen.sort((a, b) => b.index - a.index)) next = removeStage(next, one.pipeline, one.index)
  return next
}

function Flow({ config, onChange, onSelect }: {
  config: Table
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const graph = useMemo(() => toGraph(config), [config])
  const [nodes, setNodes, onNodesChange] = useNodesState(graph.nodes)
  const [edges, setEdges, onEdgesChange] = useEdgesState(graph.edges)
  const { screenToFlowPosition } = useReactFlow()
  useEffect(() => {
    setNodes(graph.nodes)
    setEdges(graph.edges)
  }, [graph, setNodes, setEdges])

  const drop = (event: DragEvent) => {
    const kind = event.dataTransfer.getData(STAGE_DRAG)
    if (!kind) return
    event.preventDefault()
    const place = dropAt(config, screenToFlowPosition({ x: event.clientX, y: event.clientY }))
    if (place) onChange(insertStage(config, place.pipeline, place.index, newStage(kind, stages(), read)))
  }

  return (
    <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} colorMode="system" fitView
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeDragStop={(_, node) => {
        const moved = (node.data as NodeData).selected
        const place = dropAt(config, node.position)
        if (moved.kind !== 'stage' || place?.pipeline !== moved.pipeline) {
          setNodes(graph.nodes)
          return
        }
        const to = Math.min(place.index, graph.nodes.filter((other) => other.id.startsWith(`stage:${moved.pipeline}:`)).length - 1)
        onChange(moveStage(config, moved.pipeline, moved.index, to - moved.index))
        onSelect({ ...moved, index: to })
      }}
      isValidConnection={isJoint}
      onConnect={(connection) => {
        const watch = graph.nodes.find((node) => node.id === connection.source)?.data.selected
        const pipeline = graph.nodes.find((node) => node.id === connection.target)?.data.selected
        if (watch?.kind === 'watch' && pipeline?.kind === 'pipeline') onChange(joinPipeline(config, watch.name, pipeline.name))
      }}
      onDelete={({ nodes: removed, edges }) => {
        let next = removeNodes(config, removed)
        for (const edge of edges) {
          const joint = edge.data as Joint | undefined
          if (joint && !removed.some((node) => node.id === edge.source || node.id === edge.target)) {
            next = partPipeline(next, joint.watch, joint.pipeline)
          }
        }
        onChange(next)
        onSelect(null)
      }}
      onDragOver={(event) => {
        if (event.dataTransfer.types.includes(STAGE_DRAG)) event.preventDefault()
      }}
      onDrop={drop}
      onNodeClick={(_, node) => onSelect((node.data as NodeData).selected)}
      onPaneClick={() => onSelect(null)}>
      <Background />
      <Controls />
    </ReactFlow>
  )
}

export function Canvas(props: {
  config: Table
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  return (
    <ReactFlowProvider>
      <Flow {...props} />
    </ReactFlowProvider>
  )
}
