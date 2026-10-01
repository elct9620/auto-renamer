import {
  Background,
  type Connection,
  ControlButton,
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
import { LayoutGrid } from 'lucide-react'
import { type DragEvent, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { BaseNode, BaseNodeContent, BaseNodeHeader, BaseNodeHeaderTitle } from '@/components/base-node'
import { KIND_ICONS } from '@/components/kinds'
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
import { type Joint, type Layout, type NodeData, type Selected, dropAt, laidOut, relaid, toGraph } from '../graph'

/** The type a stage dragged from the palette carries its name under. */
export const STAGE_DRAG = 'application/x-auto-renamer-stage'

const tones = {
  watch: 'border-sky-500/60 bg-sky-50 dark:bg-sky-950',
  pipeline: 'border-amber-500/60 bg-amber-50 dark:bg-amber-950',
  stage: 'cursor-grab',
}

function ConfigNode({ data, kind, joins }: { data: NodeData; kind: keyof typeof tones; joins: 'source' | 'target' | null }) {
  const Icon = KIND_ICONS[kind]
  return (
    <BaseNode className={cn('min-w-36 text-sm', tones[kind])}>
      <Handle type="target" position={Position.Left} isConnectable={joins === 'target'} />
      <BaseNodeHeader className="justify-start">
        <Icon className="size-3.5 shrink-0 text-muted-foreground" />
        <BaseNodeHeaderTitle className="text-sm font-medium">{data.label}</BaseNodeHeaderTitle>
      </BaseNodeHeader>
      {data.detail && (
        <BaseNodeContent className="pt-0">
          <small className="max-w-48 truncate font-mono text-xs text-muted-foreground">{data.detail}</small>
        </BaseNodeContent>
      )}
      <Handle type="source" position={Position.Right} isConnectable={joins === 'source'} />
    </BaseNode>
  )
}

// A watch is joined to pipelines by its handle; stages follow each other only by order.
const nodeTypes = {
  watch: ({ data }: NodeProps<Node<NodeData>>) => <ConfigNode data={data} kind="watch" joins="source" />,
  pipeline: ({ data }: NodeProps<Node<NodeData>>) => <ConfigNode data={data} kind="pipeline" joins="target" />,
  stage: ({ data }: NodeProps<Node<NodeData>>) => <ConfigNode data={data} kind="stage" joins={null} />,
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
  const { t } = useTranslation()
  const graph = useMemo(() => toGraph(config), [config])
  const [layout, setLayout] = useState<Layout>({})
  const [drawn, setDrawn] = useState(config)
  if (drawn !== config) {
    setDrawn(config)
    setLayout(relaid(layout, drawn, config))
  }
  const shown = useMemo(() => laidOut(graph.nodes, layout), [graph, layout])
  const [nodes, setNodes, onNodesChange] = useNodesState(shown)
  const [edges, setEdges, onEdgesChange] = useEdgesState(graph.edges)
  const { screenToFlowPosition } = useReactFlow()
  useEffect(() => {
    setNodes((current) => shown.map((node) => ({ ...node, selected: current.find((one) => one.id === node.id)?.selected })))
    setEdges(graph.edges)
  }, [shown, graph, setNodes, setEdges])

  const drop = (event: DragEvent) => {
    const kind = event.dataTransfer.getData(STAGE_DRAG)
    if (!kind) return
    event.preventDefault()
    const place = dropAt(shown, screenToFlowPosition({ x: event.clientX, y: event.clientY }))
    if (place) onChange(insertStage(config, place.pipeline, place.index, newStage(kind, stages(), read)))
  }

  return (
    <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} colorMode="system" fitView
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeDragStop={(_, node) => {
        const moved = (node.data as NodeData).selected
        const place = dropAt(shown.filter((other) => other.id !== node.id), node.position)
        if (moved.kind === 'stage' && place?.pipeline === moved.pipeline && place.index !== moved.index) {
          onChange(moveStage(config, moved.pipeline, moved.index, place.index - moved.index))
          onSelect({ ...moved, index: place.index })
          return
        }
        setLayout({ ...layout, [node.id]: node.position })
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
      <Controls>
        <ControlButton title={t('canvas.resetLayout')} aria-label={t('canvas.resetLayout')} onClick={() => setLayout({})}>
          <LayoutGrid />
        </ControlButton>
      </Controls>
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
