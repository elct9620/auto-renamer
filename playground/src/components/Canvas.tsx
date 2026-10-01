import {
  Background,
  Controls,
  Handle,
  type Node,
  type NodeProps,
  Position,
  ReactFlow,
} from '@xyflow/react'
import '@xyflow/react/dist/style.css'
import { useMemo } from 'react'

import { cn } from '@/lib/utils'
import type { Table } from '../config'
import { type NodeData, type Selected, toGraph } from '../graph'

const tones = {
  watch: 'border-sky-500/60 bg-sky-50 dark:bg-sky-950',
  pipeline: 'border-amber-500/60 bg-amber-50 dark:bg-amber-950',
  stage: 'border-border bg-card',
}

function Box({ data, tone }: { data: NodeData; tone: keyof typeof tones }) {
  return (
    <div className={cn('grid min-w-36 gap-0.5 rounded-md border px-3 py-2 text-sm shadow-xs', tones[tone])}>
      <Handle type="target" position={Position.Left} />
      <strong className="font-medium">{data.label}</strong>
      {data.detail && <small className="max-w-48 truncate font-mono text-xs text-muted-foreground">{data.detail}</small>}
      <Handle type="source" position={Position.Right} />
    </div>
  )
}

const nodeTypes = {
  watch: ({ data }: NodeProps<Node<NodeData>>) => <Box data={data} tone="watch" />,
  pipeline: ({ data }: NodeProps<Node<NodeData>>) => <Box data={data} tone="pipeline" />,
  stage: ({ data }: NodeProps<Node<NodeData>>) => <Box data={data} tone="stage" />,
}

export function Canvas({ config, onSelect }: { config: Table; onSelect: (selected: Selected | null) => void }) {
  const graph = useMemo(() => toGraph(config), [config])
  return (
    <ReactFlow nodes={graph.nodes} edges={graph.edges} nodeTypes={nodeTypes} colorMode="system" fitView
      onNodeClick={(_, node) => onSelect((node.data as NodeData).selected)}
      onPaneClick={() => onSelect(null)}>
      <Background />
      <Controls />
    </ReactFlow>
  )
}
