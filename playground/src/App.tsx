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
import { type ChangeEvent, useMemo, useState } from 'react'

import {
  STAGE_TEMPLATES,
  type Table,
  type Value,
  addStage,
  asTable,
  download,
  moveStage,
  newStage,
  readStage,
  removeStage,
  replaceStage,
  setIn,
  stagesOf,
  stageText,
} from './config'
import { type Entry, type Kind, type Simulation, check, read, render, simulate } from './core'
import { type NodeData, type Selected, toGraph } from './graph'

const EXAMPLE = `[default]
pipelines = ["video"]

[pipeline.video]
stages = [
  { filter = { ext = ["mkv", "mp4"] } },
  { number = { from = "path", into = "season", prefix = "Season" } },
  { regex = { pattern = '(\\d+)', into = "episode" } },
  { format = "{show} s{season:02}e{episode:02}" },
  "move",
]

[watch.series]
source = "/downloads"
target = "/video"
unit = { root = ["*/*"] }
vars = { show = "Alpha" }
`

const EXAMPLE_TREE: Entry[] = [
  { path: '/downloads/Alpha/Season 1/[Team] Alpha 01 [1080p].mkv', folder: false, modified: Date.now(), text: '' },
  { path: '/downloads/Alpha/Season 1/[Team] Alpha 02 [1080p].mkv', folder: false, modified: Date.now(), text: '' },
]

function Box({ data, className }: { data: NodeData; className: string }) {
  return (
    <div className={`node ${className}`}>
      <Handle type="target" position={Position.Left} />
      <strong>{data.label}</strong>
      {data.detail && <small>{data.detail}</small>}
      <Handle type="source" position={Position.Right} />
    </div>
  )
}

const nodeTypes = {
  watch: ({ data }: NodeProps<Node<NodeData>>) => <Box data={data} className="watch" />,
  pipeline: ({ data }: NodeProps<Node<NodeData>>) => <Box data={data} className="pipeline" />,
  stage: ({ data }: NodeProps<Node<NodeData>>) => <Box data={data} className="stage" />,
}

function errorOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** A text field whose value is read as an inline TOML value, applied once it reads. It starts over from
 * the configuration whenever the configuration's value changes, as after an import. */
function TomlField(props: { label: string; value: string; onApply: (value: Value | undefined) => void }) {
  return <EditedTomlField key={props.value} {...props} />
}

function EditedTomlField({ label, value, onApply }: { label: string; value: string; onApply: (value: Value | undefined) => void }) {
  const [text, setText] = useState(value)
  const [error, setError] = useState('')
  const apply = () => {
    if (text.trim() === '') {
      setError('')
      onApply(undefined)
      return
    }
    try {
      onApply(read(`value = ${text}`).value)
      setError('')
    } catch (failure) {
      setError(errorOf(failure))
    }
  }
  return (
    <label>
      {label}
      <textarea value={text} rows={2} onChange={(event) => setText(event.target.value)} onBlur={apply} />
      {error && <span className="error">{error}</span>}
    </label>
  )
}

function inline(value: Value | undefined): string {
  if (value === undefined) return ''
  return render({ value: [value] }).trim().replace(/^value = \[/, '').replace(/\]$/, '')
}

function Inspector({ config, kind, selected, onChange, onSelect }: {
  config: Table
  kind: Kind
  selected: Selected | null
  onChange: (config: Table) => void
  onSelect: (selected: Selected | null) => void
}) {
  const [stageError, setStageError] = useState('')

  if (selected === null) {
    const root = kind === 'global' ? ['default'] : []
    return (
      <section>
        <h2>{kind === 'global' ? 'Defaults' : 'Folder configuration'}</h2>
        <p className="hint">Select a watch, a pipeline or a stage to edit it.</p>
        <TomlField key={`vars-${kind}`} label="vars" value={inline(asTable(root.length ? config.default : config).vars)}
          onApply={(value) => onChange(setIn(config, [...root, 'vars'], value))} />
        {kind === 'global' && (
          <TomlField key="default-pipelines" label="pipelines" value={inline(asTable(config.default).pipelines)}
            onApply={(value) => onChange(setIn(config, ['default', 'pipelines'], value))} />
        )}
        <TomlField key={`batch-${kind}`} label="batch_max" value={inline(asTable(root.length ? config.default : config).batch_max)}
          onApply={(value) => onChange(setIn(config, [...root, 'batch_max'], value))} />
      </section>
    )
  }

  if (selected.kind === 'watch') {
    const path = ['watch', selected.name]
    const watch = asTable(asTable(config.watch)[selected.name])
    const field = (key: string) => (
      <TomlField key={`${selected.name}-${key}`} label={key} value={inline(watch[key])}
        onApply={(value) => onChange(setIn(config, [...path, key], value))} />
    )
    return (
      <section>
        <h2>Watch {selected.name}</h2>
        {['source', 'target', 'unit', 'pipelines', 'vars', 'dry_run'].map(field)}
        <button onClick={() => { onChange(setIn(config, path, undefined)); onSelect(null) }}>Remove watch</button>
      </section>
    )
  }

  if (selected.kind === 'pipeline') {
    return (
      <section>
        <h2>Pipeline {selected.name}</h2>
        <label>
          Add a stage
          <select value="" onChange={(event) => {
            if (event.target.value) onChange(addStage(config, selected.name, newStage(event.target.value, read)))
          }}>
            <option value="">choose…</option>
            {Object.keys(STAGE_TEMPLATES).map((kind) => <option key={kind}>{kind}</option>)}
          </select>
        </label>
        <button onClick={() => { onChange(setIn(config, ['pipeline', selected.name], undefined)); onSelect(null) }}>
          Remove pipeline
        </button>
      </section>
    )
  }

  const stages = stagesOf(config, selected.pipeline)
  const stage = stages[selected.index]
  if (stage === undefined) return null
  const at = (index: number) => onSelect({ ...selected, index })
  return (
    <section>
      <h2>Stage {selected.index + 1} of {selected.pipeline}</h2>
      <label>
        Written as
        <textarea key={`${selected.pipeline}-${selected.index}-${stageText(stage, render)}`}
          defaultValue={stageText(stage, render)} rows={4}
          onBlur={(event) => {
            try {
              onChange(replaceStage(config, selected.pipeline, selected.index, readStage(event.target.value, read)))
              setStageError('')
            } catch (failure) {
              setStageError(errorOf(failure))
            }
          }} />
        {stageError && <span className="error">{stageError}</span>}
      </label>
      <div className="row">
        <button disabled={selected.index === 0} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, -1)); at(selected.index - 1)
        }}>Earlier</button>
        <button disabled={selected.index === stages.length - 1} onClick={() => {
          onChange(moveStage(config, selected.pipeline, selected.index, 1)); at(selected.index + 1)
        }}>Later</button>
        <button onClick={() => { onChange(removeStage(config, selected.pipeline, selected.index)); onSelect(null) }}>
          Remove
        </button>
      </div>
    </section>
  )
}

function TreeEditor({ entries, onChange }: { entries: Entry[]; onChange: (entries: Entry[]) => void }) {
  const [path, setPath] = useState('')
  const files = entries.filter((entry) => !entry.folder)
  return (
    <div>
      <ul className="tree">
        {files.map((entry) => (
          <li key={entry.path}>
            <code>{entry.path}</code>
            {entry.path.endsWith('/auto-renamer.toml') && (
              <textarea value={entry.text} rows={3} onChange={(event) =>
                onChange(entries.map((other) => other.path === entry.path ? { ...other, text: event.target.value } : other))} />
            )}
            <button onClick={() => onChange(entries.filter((other) => other.path !== entry.path))}>×</button>
          </li>
        ))}
      </ul>
      <div className="row">
        <input placeholder="/downloads/Show/file.mkv" value={path} onChange={(event) => setPath(event.target.value)} />
        <button disabled={!path.startsWith('/')} onClick={() => {
          onChange([...entries.filter((entry) => entry.path !== path), { path, folder: false, modified: Date.now(), text: '' }])
          setPath('')
        }}>Add file</button>
      </div>
    </div>
  )
}

export default function App() {
  const [kind, setKind] = useState<Kind>('global')
  const [config, setConfig] = useState<Table>(() => read(EXAMPLE))
  const [selected, setSelected] = useState<Selected | null>(null)
  const [message, setMessage] = useState('')
  const [entries, setEntries] = useState<Entry[]>(EXAMPLE_TREE)
  const [watch, setWatch] = useState('series')
  const [simulation, setSimulation] = useState<Simulation | null>(null)

  const graph = useMemo(() => toGraph(config), [config])
  const text = useMemo(() => render(config), [config])
  const status = useMemo(() => {
    try {
      const warnings = check(kind, text)
      return { ok: true, lines: warnings }
    } catch (error) {
      return { ok: false, lines: [errorOf(error)] }
    }
  }, [kind, text])
  const watches = Object.keys(asTable(config.watch))

  const importFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    try {
      setConfig(read(await file.text()))
      setKind(file.name === 'auto-renamer.toml' ? 'folder' : 'global')
      setSelected(null)
      setMessage(`Imported ${file.name}`)
    } catch (error) {
      setMessage(errorOf(error))
    }
  }

  const save = () => {
    const offered = download(kind, config, render, check)
    if (!offered.file) {
      setMessage(`Not downloadable: ${offered.refused}`)
      return
    }
    const link = document.createElement('a')
    link.href = URL.createObjectURL(new Blob([offered.file.text], { type: 'application/toml' }))
    link.download = offered.file.name
    link.click()
    URL.revokeObjectURL(link.href)
    setMessage(`Downloaded ${offered.file.name}`)
  }

  const add = (table: 'watch' | 'pipeline') => {
    const existing = Object.keys(asTable(config[table]))
    let name = table === 'watch' ? 'new_watch' : 'new_pipeline'
    for (let n = 2; existing.includes(name); n += 1) name = `${table === 'watch' ? 'new_watch' : 'new_pipeline'}_${n}`
    const value: Table = table === 'watch' ? { source: '/downloads' } : { stages: [] }
    setConfig(setIn(config, [table, name], value))
    setSelected({ kind: table, name })
  }

  const run = () => {
    try {
      setSimulation(simulate(text, watch, entries))
      setMessage('')
    } catch (error) {
      setSimulation(null)
      setMessage(errorOf(error))
    }
  }

  return (
    <div className="app">
      <header>
        <h1>auto-renamer playground</h1>
        <div className="row">
          <select value={kind} onChange={(event) => { setKind(event.target.value as Kind); setSelected(null) }}>
            <option value="global">Global configuration</option>
            <option value="folder">Folder configuration</option>
          </select>
          {kind === 'global' && <button onClick={() => add('watch')}>Add watch</button>}
          <button onClick={() => add('pipeline')}>Add pipeline</button>
          <label className="button">Import<input type="file" accept=".toml" onChange={importFile} hidden /></label>
          <button onClick={save} disabled={!status.ok}>Download</button>
        </div>
        {message && <p className="message">{message}</p>}
      </header>
      <main>
        <div className="canvas">
          <ReactFlow nodes={graph.nodes} edges={graph.edges} nodeTypes={nodeTypes} fitView
            onNodeClick={(_, node) => setSelected((node.data as NodeData).selected)}
            onPaneClick={() => setSelected(null)}>
            <Background />
            <Controls />
          </ReactFlow>
        </div>
        <aside>
          <Inspector config={config} kind={kind} selected={selected} onChange={setConfig} onSelect={setSelected} />
        </aside>
      </main>
      <footer>
        <section>
          <h2>{kind === 'global' ? 'config.toml' : 'auto-renamer.toml'}</h2>
          <pre className={status.ok ? '' : 'refused'}>{text}</pre>
          {status.lines.map((line) => <p key={line} className={status.ok ? 'warning' : 'error'}>{line}</p>)}
        </section>
        {kind === 'global' && (
          <section>
            <h2>Simulation</h2>
            <p className="hint">Files in the source, and any <code>auto-renamer.toml</code> beside them. Each unit is processed as one batch.</p>
            <TreeEditor entries={entries} onChange={(next) => { setEntries(next); setSimulation(null) }} />
            <div className="row">
              <select value={watch} onChange={(event) => setWatch(event.target.value)}>
                {watches.map((name) => <option key={name}>{name}</option>)}
              </select>
              <button onClick={run} disabled={!status.ok || !watches.includes(watch)}>Trigger</button>
              {simulation && <button onClick={() => { setEntries(simulation.entries); setSimulation(null) }}>Keep the result</button>}
            </div>
            {simulation && (
              <table>
                <thead><tr><th>File</th><th>Became</th><th>To</th><th>Why</th></tr></thead>
                <tbody>
                  {simulation.outcomes.map((outcome) => (
                    <tr key={outcome.origin}>
                      <td>{outcome.origin}</td><td>{outcome.what}</td><td>{outcome.to}</td><td>{outcome.reason}</td>
                    </tr>
                  ))}
                </tbody>
              </table>
            )}
          </section>
        )}
      </footer>
    </div>
  )
}
