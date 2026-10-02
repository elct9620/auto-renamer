import { Check, Download, Play, Upload } from 'lucide-react'
import { type ChangeEvent, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Canvas } from '@/components/Canvas'
import { Choice } from '@/components/Choice'
import { Inspector } from '@/components/Inspector'
import { Output } from '@/components/Output'
import { Palette } from '@/components/Palette'
import { TreePanel } from '@/components/TreePanel'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { errorOf } from '@/lib/utils'
import { type Table, asTable, download, folderStart, overridden, setIn } from './config'
import { type Entry, type Kind, type Simulation, check, read, render, simulate } from './core'
import { type Example, EXAMPLES, FIRST } from './examples'
import type { Selected } from './graph'
import { LANGUAGES } from './i18n'
import { FOLDER_CONFIGURATION, foldersUnder, marksOf, rootsOf, startConfiguration, writeFile } from './tree'

// The values the configuration switcher gives the global configuration and a folder still to be configured.
const GLOBAL = '\u0000global'
const ADD = '\u0000add:'

function checked(kind: Kind, text: string): { ok: boolean; lines: string[] } {
  try {
    check(kind, text)
    return { ok: true, lines: [] }
  } catch (error) {
    return { ok: false, lines: [errorOf(error)] }
  }
}

/** Everything the page holds starts from an example, so choosing one, even the same again, starts the page over. */
export default function App() {
  const [chosen, setChosen] = useState({ example: FIRST, times: 0 })
  return <Playground key={chosen.times} example={chosen.example}
    onChoose={(example) => setChosen({ example, times: chosen.times + 1 })} />
}

function Playground({ example, onChoose }: { example: Example; onChoose: (example: Example) => void }) {
  const { t, i18n } = useTranslation()
  const [config, setConfig] = useState<Table>(() => read(example.config))
  // The folder configuration being edited, by its path in the tree; the global configuration otherwise.
  const [editing, setEditing] = useState<string | null>(null)
  const [selected, setSelected] = useState<Selected | null>(null)
  const [message, setMessage] = useState('')
  const [entries, setEntries] = useState<Entry[]>(example.entries)
  const [watch, setWatch] = useState(example.watch)
  const [simulation, setSimulation] = useState<Simulation | null>(null)

  const folder = entries.find((entry) => entry.path === editing)
  const kind: Kind = folder ? 'folder' : 'global'
  const edited = useMemo(() => (folder ? read(folder.text) : config), [folder, config])
  const globalText = useMemo(() => render(config), [config])
  const text = useMemo(() => (folder ? render(edited) : globalText), [folder, edited, globalText])
  const status = useMemo(() => checked(kind, text), [kind, text])
  const runnable = useMemo(() => checked('global', globalText).ok, [globalText])
  const watches = Object.keys(asTable(config.watch))
  const roots = rootsOf(config, watch)
  const fileName = kind === 'global' ? 'config.toml' : FOLDER_CONFIGURATION

  // A simulation answers for the configuration it ran, so any edit to a configuration clears it.
  const edit = (next: Table) => {
    if (folder) editEntries((all) => writeFile(all, folder.path, render(next)))
    else {
      setConfig(next)
      setSimulation(null)
    }
  }

  const open = (path: string | null) => {
    if (path !== null) {
      try {
        read(entries.find((entry) => entry.path === path)?.text ?? '')
      } catch (error) {
        setMessage(errorOf(error))
        return
      }
    }
    setEditing(path)
    setSelected(null)
  }

  const importFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    try {
      const table = read(await file.text())
      if (file.name === FOLDER_CONFIGURATION) {
        const path = folder?.path ?? `${roots.source}/${FOLDER_CONFIGURATION}`
        editEntries((all) => writeFile(all, path, render(table)))
        setEditing(path)
      } else {
        setConfig(table)
        setEditing(null)
      }
      setSelected(null)
      setMessage(t('message.imported', { name: file.name }))
    } catch (error) {
      setMessage(errorOf(error))
    }
  }

  const save = () => {
    const offered = download(kind, edited, render, check)
    if (!offered.file) {
      setMessage(t('message.refused', { reason: offered.refused }))
      return
    }
    const link = document.createElement('a')
    const url = URL.createObjectURL(new Blob([offered.file.text], { type: 'application/toml' }))
    link.href = url
    link.download = offered.file.name
    link.click()
    URL.revokeObjectURL(url)
    setMessage(t('message.downloaded', { name: offered.file.name }))
  }

  const add = (table: 'watch' | 'pipeline') => {
    const existing = Object.keys(asTable(edited[table]))
    const base = table === 'watch' ? 'new_watch' : 'new_pipeline'
    let name = base
    for (let n = 2; existing.includes(name); n += 1) name = `${base}_${n}`
    const value: Table = table === 'watch' ? { source: '/downloads' } : { stages: [] }
    edit(setIn(edited, [table, name], value))
    setSelected({ kind: table, name })
  }

  const editEntries = (change: (entries: Entry[]) => Entry[]) => {
    setEntries(change(entries))
    setSimulation(null)
  }
  const marks = marksOf(simulation)

  // The header offers the global configuration, every folder configuration of the source, and a new one for
  // each folder still without one.
  const folderOf = (path: string) => path.slice(0, -FOLDER_CONFIGURATION.length - 1)
  const configurations = entries.filter((entry) => entry.path.startsWith(`${roots.source}/`) && entry.path.endsWith(`/${FOLDER_CONFIGURATION}`))
  const folders = [roots.source, ...foldersUnder(entries, roots.source).map((folder) => `${roots.source}/${folder.slice(0, -1)}`)]
  const bare = folders.filter((folder) => !configurations.some((entry) => folderOf(entry.path) === folder))
  const editable = [
    { value: GLOBAL, label: t('editing.global') },
    ...configurations.map((entry) => ({ value: entry.path, label: t('editing.folder', { folder: folderOf(entry.path) }) })),
    ...bare.map((folder) => ({ value: `${ADD}${folder}`, label: t('editing.add', { folder }) })),
  ]
  // A global pipeline overridden in a folder lands in that folder's configuration, which opens to be changed.
  const override = (pipeline: string, folder: string) => {
    const path = `${folder}/${FOLDER_CONFIGURATION}`
    const existing = entries.find((entry) => entry.path === path)
    let table: Table
    try {
      table = existing ? read(existing.text) : folderStart(config, watch)
    } catch (error) {
      setMessage(errorOf(error))
      return
    }
    editEntries((all) => writeFile(all, path, render(overridden(table, config, pipeline))))
    open(path)
  }
  const choose = (value: string) => {
    if (value === GLOBAL) return open(null)
    if (value.startsWith(ADD)) {
      const folder = value.slice(ADD.length)
      editEntries((all) => startConfiguration(all, folder, render(folderStart(config, watch))))
      return open(`${folder}/${FOLDER_CONFIGURATION}`)
    }
    open(value)
  }

  const run = () => {
    try {
      setSimulation(simulate(globalText, watch, entries))
      setMessage('')
    } catch (error) {
      setSimulation(null)
      setMessage(errorOf(error))
    }
  }

  return (
    <div className="grid h-dvh grid-cols-[minmax(0,1fr)] grid-rows-[auto_minmax(0,1fr)_auto_minmax(0,11rem)] bg-background text-foreground">
      <header className="flex flex-wrap items-center gap-2 border-b px-3 py-2">
        <h1 className="mr-2 text-sm font-semibold">Auto Renamer Playground</h1>
        <Choice label={t('editing.label')} className="w-auto" value={editing ?? GLOBAL} options={editable} onChange={choose} />
        <Button variant="outline" size="sm" asChild>
          <label><Upload />{t('import')}<input type="file" accept=".toml" onChange={importFile} hidden /></label>
        </Button>
        <Button size="sm" onClick={save} disabled={!status.ok}><Download />{t('download')}</Button>
        <Choice label={t('examples.label')} placeholder={t('examples.label')} className="w-auto" value=""
          options={EXAMPLES.map((one) => ({ value: one.id, label: t(`examples.${one.id}`) }))}
          onChange={(id) => onChoose(EXAMPLES.find((one) => one.id === id)!)} />
        <Badge variant={status.ok ? 'secondary' : 'destructive'}>
          {status.ok ? t('status.valid') : t('status.refused')}
        </Badge>
        {message && <span className="text-xs text-muted-foreground">{message}</span>}
        <Choice label={t('language')} className="ml-auto w-auto" value={i18n.language}
          options={Object.entries(LANGUAGES).map(([code, name]) => ({ value: code, label: name }))}
          onChange={(code) => i18n.changeLanguage(code)} />
      </header>

      <main className="grid min-h-0 grid-cols-[11rem_minmax(0,1fr)_20rem] border-b">
        <aside className="min-h-0 overflow-auto border-r">
          <Palette global={!folder} onAdd={add} />
        </aside>
        <div className="min-h-0">
          <Canvas key={editing ?? ''} config={edited} onChange={edit} onSelect={setSelected} />
        </div>
        <aside className="min-h-0 overflow-auto border-l p-3">
          <Inspector config={edited} kind={kind} selected={selected} folders={folders} simulation={simulation}
            folder={folder ? folderOf(folder.path).slice(roots.source.length + 1) : null} onChange={edit} onSelect={setSelected}
            onOverride={folder ? undefined : override} />
        </aside>
      </main>

      <section className="grid h-56 min-h-0 grid-cols-[minmax(0,1fr)_10rem_minmax(0,1fr)] border-b">
        <TreePanel title={t('tree.source')} root={roots.source} entries={entries} marks={marks} editing={editing}
          starting={render(folderStart(config, watch))} onEdit={editEntries} onOpen={open} />
        <div className="flex flex-col justify-center gap-2 border-x p-3">
          <Choice label={t('tree.watch')} value={watch} options={watches.map((name) => ({ value: name, label: name }))}
            onChange={setWatch} />
          <Button size="sm" onClick={run} disabled={!runnable || !watches.includes(watch)}><Play />{t('tree.trigger')}</Button>
          {simulation && (
            <Button size="sm" variant="outline" onClick={() => editEntries(() => simulation.entries)}><Check />{t('tree.keep')}</Button>
          )}
        </div>
        <TreePanel title={t('tree.target')} root={roots.target} entries={simulation?.entries ?? entries} marks={marks} editing={editing}
          onEdit={editEntries} />
      </section>

      <Output fileName={fileName} text={text} status={status} simulation={simulation} />
    </div>
  )
}
