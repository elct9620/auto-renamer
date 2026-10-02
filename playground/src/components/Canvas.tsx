import {
  Background,
  type Connection,
  ControlButton,
  Controls,
  type Edge,
  type EdgeProps,
  Handle,
  NodeToolbar,
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
import { LayoutGrid, Lock, X } from 'lucide-react'
import { type DragEvent, createContext, useContext, useEffect, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { BaseNode, BaseNodeContent, BaseNodeHeader, BaseNodeHeaderTitle } from '@/components/base-node'
import { ButtonEdge } from '@/components/button-edge'
import { GroupNode } from '@/components/labeled-group-node'
import { NodeAppendix } from '@/components/node-appendix'
import { StageFiles } from '@/components/StageFiles'
import { StageName } from '@/components/Fields'
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
} from '@/components/ui/alert-dialog'
import { Button } from '@/components/ui/button'
import { KIND_ICONS, stageIcon } from '@/components/kinds'
import { cn } from '@/lib/utils'
import {
  type Table,
  insertStage,
  joinPipeline,
  moveRoute,
  moveStage,
  newStage,
  partPipeline,
  removePipeline,
  removeRoute,
  removeStage,
  removeTarget,
  routeTo,
  setIn,
  transferStage,
} from '../config'
import { type Simulation, builtIns, read, stages } from '../core'
import { APPENDIX, type Joint, type Layout, type Move, type NodeData, type Selected, absolute, dropAt, laidOut, relaid, routeAt, toGraph } from '../graph'
import { afterStage } from '../steps'

/** The type a stage dragged from the palette carries its name under. */
export const STAGE_DRAG = 'application/x-auto-renamer-stage'

const tones = {
  watch: 'border-sky-500/60 bg-sky-50/60 dark:bg-sky-950/60',
  route: 'border-sky-500/60',
  pipeline: 'border-amber-500/60 bg-amber-50/60 dark:bg-amber-950/60',
  stage: 'cursor-grab',
  target: 'border-emerald-500/60 bg-emerald-50 dark:bg-emerald-950',
}

type Kind = keyof typeof tones

/** A route, a stage or a target: its header and detail. */
function ConfigNode({ data, kind }: { data: NodeData; kind: Kind }) {
  const Icon = kind === 'stage' ? stageIcon(data.label) : KIND_ICONS[kind]
  return (
    <BaseNode className={cn('min-w-36 text-sm', tones[kind], data.followed && 'border-dashed',
      data.locked && 'opacity-70', data.locked && kind === 'stage' && 'cursor-default')}>
      <BaseNodeHeader className="justify-start">
        <Icon className="size-3.5 shrink-0 text-muted-foreground" />
        <BaseNodeHeaderTitle className="truncate text-sm font-medium">{kind === 'stage' ? <StageName name={data.label} /> : data.label}</BaseNodeHeaderTitle>
        {data.locked && kind === 'pipeline' && <Lock className="ml-auto size-3.5 shrink-0 text-muted-foreground" />}
      </BaseNodeHeader>
      {data.detail && (
        <BaseNodeContent className="pt-0">
          <small className="max-w-48 truncate font-mono text-xs text-muted-foreground">{data.detail}</small>
        </BaseNodeContent>
      )}
    </BaseNode>
  )
}

/** How far below a selected stage, on screen, the middle of the canvas falls, so the files listed under it fit. */
const LISTED_BELOW = 140

/** The simulation the canvas shows on its stages, and the folder whose configuration is edited. */
const Run = createContext<{ simulation: Simulation | null; folder: string | null }>({ simulation: null, folder: null })

/** A stage, which once a simulation ran tells how many files it ran on and, while selected, lists them beside it. */
function StageNode({ data }: NodeProps<Node<NodeData>>) {
  const { t } = useTranslation()
  const { simulation, folder } = useContext(Run)
  const at = data.selected.kind === 'stage' ? data.selected : null
  const stage = at && { pipeline: at.pipeline, index: at.index, name: data.label, folder, claims: data.claims ?? false }
  const ran = simulation && stage ? afterStage(simulation.outcomes, stage.pipeline, stage.index, folder, stage.claims).ran.length : null
  return (
    <>
      <Handle type="target" position={Position.Left} isConnectable={false} />
      <ConfigNode data={data} kind="stage" />
      <Handle type="source" position={Position.Right} isConnectable={false} />
      {ran !== null && <NodeAppendix position="bottom" className="text-xs text-muted-foreground">{t('canvas.ran', { count: ran })}</NodeAppendix>}
      {simulation && stage && (
        <NodeToolbar position={Position.Bottom} offset={APPENDIX + 16}>
          <section aria-label={t('canvas.after', { stage: data.label })} className="max-h-60 w-[32rem] overflow-auto rounded-md border bg-popover p-2 shadow-md">
            <StageFiles simulation={simulation} stage={stage} />
          </section>
        </NodeToolbar>
      )}
    </>
  )
}

/** A watch or a pipeline: a React Flow UI group holding its routes or stages, labelled with its kind and name. */
function ConfigGroup({ data, kind }: { data: NodeData; kind: 'watch' | 'pipeline' }) {
  const Icon = KIND_ICONS[kind]
  return (
    <GroupNode className={cn('text-sm', tones[kind], data.locked && 'opacity-70')}
      label={
        <span className="flex items-center gap-1.5 font-medium">
          <Icon className="size-3.5 shrink-0 text-muted-foreground" />{data.label}
          {data.locked && <Lock className="size-3.5 shrink-0 text-muted-foreground" />}
        </span>
      } />
  )
}

// Everything flows right: a watch is joined to pipelines by its handle, and each of its routes to its pipeline and
// its targets; stages follow each other only by order.
const nodeTypes = {
  watch: ({ data }: NodeProps<Node<NodeData>>) => (
    <>
      <ConfigGroup data={data} kind="watch" />
      <Handle type="source" position={Position.Right} />
    </>
  ),
  route: ({ data }: NodeProps<Node<NodeData>>) => (
    <>
      <ConfigNode data={data} kind="route" />
      <Handle type="source" position={Position.Right} style={{ top: '35%' }} isConnectable={false} />
      <Handle type="source" id="effects" position={Position.Right} style={{ top: '70%' }} />
    </>
  ),
  pipeline: ({ data }: NodeProps<Node<NodeData>>) => (
    <>
      <ConfigGroup data={data} kind="pipeline" />
      <Handle type="target" position={Position.Left} />
      <Handle type="source" id="end" position={Position.Right} isConnectable={false} />
    </>
  ),
  stage: StageNode,
  target: ({ data }: NodeProps<Node<NodeData>>) => (
    <>
      <Handle type="target" position={Position.Left} isConnectable={false} />
      <ConfigNode data={data} kind="target" />
    </>
  ),
}

// A joint leaves a node held by a watch, so React Flow draws it above the nodes; its button is lifted above every
// joint, where a joint's pointer area cannot cover it.
const ABOVE_JOINTS = 1001

/** A joint of a route to its pipeline, with a button removing the route, the way deleting the edge would. */
function JointEdge(props: EdgeProps<Edge<Joint>>) {
  const { t } = useTranslation()
  const { deleteElements } = useReactFlow()
  return (
    <ButtonEdge {...props} labelStyle={{ zIndex: ABOVE_JOINTS }}>
      <Button variant="outline" size="icon-xs" className="rounded-full"
        aria-label={t('canvas.part', { watch: props.data?.watch, pipeline: props.data?.pipeline })}
        onClick={() => deleteElements({ edges: [{ id: props.id }] })}>
        <X />
      </Button>
    </ButtonEdge>
  )
}

const edgeTypes = { joint: JointEdge }

/** A watch is joined to a pipeline to route it, and a route to a target to move there. */
function isJoint(connection: Connection | Edge): boolean {
  return (connection.source.startsWith('watch:') && connection.target.startsWith('pipeline:'))
    || (connection.source.startsWith('route:') && connection.target.startsWith('target:'))
}

/** Removes what was deleted on the canvas: routes and stages from the last, so earlier positions stay valid. */
function removeNodes(config: Table, nodes: Node[]): Table {
  const chosen = nodes.map((node) => (node.data as NodeData).selected)
  let next = config
  const routes = chosen.flatMap((one) => (one.kind === 'route' ? [one] : []))
  for (const one of routes.sort((a, b) => b.index - a.index)) next = removeRoute(next, one.watch, one.index)
  for (const one of chosen) {
    if (one.kind === 'watch') next = setIn(next, ['watch', one.name], undefined)
    if (one.kind === 'pipeline') next = removePipeline(next, one.name)
    if (one.kind === 'target') next = removeTarget(next, one.name)
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
  const graph = useMemo(() => toGraph(config, builtIns()), [config])
  const [layout, setLayout] = useState<Layout>({})
  const [drawn, setDrawn] = useState(config)
  if (drawn !== config) {
    setDrawn(config)
    setLayout(relaid(layout, drawn, config))
  }
  const shown = useMemo(() => laidOut(graph.nodes, layout), [graph, layout])
  const [nodes, setNodes, onNodesChange] = useNodesState(shown)
  const [edges, setEdges, onEdgesChange] = useEdgesState(graph.edges)
  const { screenToFlowPosition, setCenter, getZoom } = useReactFlow()
  const { simulation } = useContext(Run)
  useEffect(() => {
    setNodes((current) => shown.map((node) => ({ ...node, selected: current.find((one) => one.id === node.id)?.selected })))
    setEdges(graph.edges)
  }, [shown, graph, setNodes, setEdges])

  // A stage dropped on another pipeline waits here until the move is confirmed.
  const [transfer, setTransfer] = useState<{ stage: { pipeline: string; index: number }; to: { pipeline: string; index: number }; name: string } | null>(null)
  const goBack = (id: string) => {
    const drawnAt = shown.find((one) => one.id === id)?.position
    if (drawnAt) setNodes((current) => current.map((one) => (one.id === id ? { ...one, position: drawnAt } : one)))
  }

  const drop = (event: DragEvent) => {
    const kind = event.dataTransfer.getData(STAGE_DRAG)
    if (!kind) return
    event.preventDefault()
    const place = dropAt(shown, screenToFlowPosition({ x: event.clientX, y: event.clientY }))
    if (place) onChange(insertStage(config, place.pipeline, place.index, newStage(kind, stages(), read)))
  }

  return (
    <>
    <ReactFlow nodes={nodes} edges={edges} nodeTypes={nodeTypes} edgeTypes={edgeTypes} colorMode="system" fitView
      // A long pipeline is wider than the canvas; the whole drawing is fitted, scaled down past the default limit.
      minZoom={0.2} fitViewOptions={{ maxZoom: 1 }}
      onNodesChange={onNodesChange}
      onEdgesChange={onEdgesChange}
      onNodeDragStop={(_, node) => {
        const moved = (node.data as NodeData).selected
        const place = dropAt(shown.filter((other) => other.id !== node.id), absolute(shown, node as Node<NodeData>))
        if (moved.kind === 'route') {
          const index = routeAt(shown.filter((other) => other.id !== node.id), moved.watch, node.position)
          if (index !== null && index !== moved.index) {
            onChange(moveRoute(config, moved.watch, moved.index, index - moved.index))
            onSelect({ ...moved, index })
            return
          }
        }
        if (moved.kind === 'stage' && place?.pipeline === moved.pipeline && place.index !== moved.index) {
          onChange(moveStage(config, moved.pipeline, moved.index, place.index - moved.index))
          onSelect({ ...moved, index: place.index })
          return
        }
        // A route or a stage stays where its list places it; dropped anywhere else, it goes back.
        if (node.parentId) {
          if (moved.kind === 'stage' && place && place.pipeline !== moved.pipeline) {
            setTransfer({ stage: moved, to: place, name: (node.data as NodeData).label })
          }
          goBack(node.id)
          return
        }
        setLayout({ ...layout, [node.id]: node.position })
      }}
      isValidConnection={isJoint}
      onConnect={(connection) => {
        const from = graph.nodes.find((node) => node.id === connection.source)?.data.selected
        const to = graph.nodes.find((node) => node.id === connection.target)?.data.selected
        if (from?.kind === 'watch' && to?.kind === 'pipeline') onChange(joinPipeline(config, from.name, to.name))
        if (from?.kind === 'route' && to?.kind === 'target') onChange(routeTo(config, from.watch, from.index, to.name))
      }}
      onDelete={({ nodes: removed, edges }) => {
        let next = removeNodes(config, removed)
        for (const edge of edges) {
          if (removed.some((node) => node.id === edge.source || node.id === edge.target)) continue
          if (edge.type === 'joint') {
            const joint = edge.data as Joint
            next = partPipeline(next, joint.watch, joint.pipeline)
          } else if (edge.data) {
            const move = edge.data as Move
            next = routeTo(next, move.watch, move.index, undefined)
          }
        }
        onChange(next)
        onSelect(null)
      }}
      onDragOver={(event) => {
        if (event.dataTransfer.types.includes(STAGE_DRAG)) event.preventDefault()
      }}
      onDrop={drop}
      onNodeClick={(_, node) => {
        const one = node as Node<NodeData>
        onSelect(one.data.selected)
        // The files after a stage are listed below it, so the stage is brought to the upper middle to leave them room.
        if (one.data.selected.kind === 'stage' && simulation) {
          const at = absolute(shown, one)
          const zoom = getZoom()
          setCenter(at.x + (one.measured?.width ?? 0) / 2, at.y + LISTED_BELOW / zoom, { zoom, duration: 200 })
        }
      }}
      onPaneClick={() => onSelect(null)}>
      <Background />
      <Controls>
        <ControlButton title={t('canvas.resetLayout')} aria-label={t('canvas.resetLayout')} onClick={() => setLayout({})}>
          <LayoutGrid />
        </ControlButton>
      </Controls>
    </ReactFlow>
    <AlertDialog open={transfer !== null} onOpenChange={(open) => { if (!open) setTransfer(null) }}>
      <AlertDialogContent>
        <AlertDialogHeader>
          <AlertDialogTitle>{t('canvas.transfer', { stage: transfer?.name, pipeline: transfer?.to.pipeline })}</AlertDialogTitle>
          <AlertDialogDescription>{t('canvas.transferHint', { from: transfer?.stage.pipeline })}</AlertDialogDescription>
        </AlertDialogHeader>
        <AlertDialogFooter>
          <AlertDialogCancel>{t('canvas.cancel')}</AlertDialogCancel>
          <AlertDialogAction onClick={() => {
            if (!transfer) return
            onChange(transferStage(config, transfer.stage.pipeline, transfer.stage.index, transfer.to.pipeline, transfer.to.index))
            onSelect({ kind: 'stage', pipeline: transfer.to.pipeline, index: transfer.to.index })
          }}>{t('canvas.move')}</AlertDialogAction>
        </AlertDialogFooter>
      </AlertDialogContent>
    </AlertDialog>
    </>
  )
}

export function Canvas({ simulation, folder, ...props }: {
  config: Table
  simulation: Simulation | null
  /** The folder whose configuration is edited, relative to the source; null for the global configuration. */
  folder: string | null
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const run = useMemo(() => ({ simulation, folder }), [simulation, folder])
  return (
    <ReactFlowProvider>
      <Run.Provider value={run}>
        <Flow {...props} />
      </Run.Provider>
    </ReactFlowProvider>
  )
}
