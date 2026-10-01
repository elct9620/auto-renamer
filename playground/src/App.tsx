import { type ChangeEvent, useMemo, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Canvas } from '@/components/Canvas'
import { Inspector } from '@/components/Inspector'
import { Output } from '@/components/Output'
import { Palette } from '@/components/Palette'
import { TreePanel } from '@/components/TreePanel'
import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { errorOf } from '@/lib/utils'
import { type Table, asTable, download, setIn } from './config'
import { type Entry, type Kind, type Simulation, check, read, render, simulate } from './core'
import type { Selected } from './graph'
import { LANGUAGES } from './i18n'
import { marksOf, rootsOf } from './tree'

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

export default function App() {
  const { t, i18n } = useTranslation()
  const [kind, setKind] = useState<Kind>('global')
  const [config, setConfig] = useState<Table>(() => read(EXAMPLE))
  const [selected, setSelected] = useState<Selected | null>(null)
  const [message, setMessage] = useState('')
  const [entries, setEntries] = useState<Entry[]>(EXAMPLE_TREE)
  const [watch, setWatch] = useState('series')
  const [simulation, setSimulation] = useState<Simulation | null>(null)

  const text = useMemo(() => render(config), [config])
  const status = useMemo(() => {
    try {
      return { ok: true, lines: check(kind, text) }
    } catch (error) {
      return { ok: false, lines: [errorOf(error)] }
    }
  }, [kind, text])
  const watches = Object.keys(asTable(config.watch))
  const roots = rootsOf(config, watch)
  const fileName = kind === 'global' ? 'config.toml' : 'auto-renamer.toml'

  const importFile = async (event: ChangeEvent<HTMLInputElement>) => {
    const file = event.target.files?.[0]
    event.target.value = ''
    if (!file) return
    try {
      setConfig(read(await file.text()))
      setKind(file.name === 'auto-renamer.toml' ? 'folder' : 'global')
      setSelected(null)
      setMessage(t('message.imported', { name: file.name }))
    } catch (error) {
      setMessage(errorOf(error))
    }
  }

  const save = () => {
    const offered = download(kind, config, render, check)
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
    const existing = Object.keys(asTable(config[table]))
    const base = table === 'watch' ? 'new_watch' : 'new_pipeline'
    let name = base
    for (let n = 2; existing.includes(name); n += 1) name = `${base}_${n}`
    const value: Table = table === 'watch' ? { source: '/downloads' } : { stages: [] }
    setConfig(setIn(config, [table, name], value))
    setSelected({ kind: table, name })
  }

  const editEntries = (change: (entries: Entry[]) => Entry[]) => {
    setEntries(change(entries))
    setSimulation(null)
  }
  const marks = marksOf(simulation)

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
    <div className="grid h-dvh grid-cols-[minmax(0,1fr)] grid-rows-[auto_minmax(0,1fr)_auto_minmax(0,11rem)] bg-background text-foreground">
      <header className="flex flex-wrap items-center gap-2 border-b px-3 py-2">
        <h1 className="mr-2 text-sm font-semibold">{t('title')}</h1>
        <select aria-label={t('kind.label')} className="h-8 rounded-md border bg-background px-2 text-sm" value={kind}
          onChange={(event) => { setKind(event.target.value as Kind); setSelected(null) }}>
          <option value="global">{t('kind.global')}</option>
          <option value="folder">{t('kind.folder')}</option>
        </select>
        <Button variant="outline" size="sm" asChild>
          <label>{t('import')}<input type="file" accept=".toml" onChange={importFile} hidden /></label>
        </Button>
        <Button size="sm" onClick={save} disabled={!status.ok}>{t('download')}</Button>
        <Badge variant={status.ok ? 'secondary' : 'destructive'}>
          {status.ok ? (status.lines.length ? t('status.warnings', { count: status.lines.length }) : t('status.valid')) : t('status.refused')}
        </Badge>
        {message && <span className="text-xs text-muted-foreground">{message}</span>}
        <select aria-label={t('language')} className="ml-auto h-8 rounded-md border bg-background px-2 text-sm"
          value={i18n.language} onChange={(event) => i18n.changeLanguage(event.target.value)}>
          {Object.entries(LANGUAGES).map(([code, name]) => <option key={code} value={code}>{name}</option>)}
        </select>
      </header>

      <main className="grid min-h-0 grid-cols-[11rem_minmax(0,1fr)_20rem] border-b">
        <aside className="min-h-0 overflow-auto border-r">
          <Palette global={kind === 'global'} onAdd={add} />
        </aside>
        <div className="min-h-0">
          <Canvas config={config} onChange={setConfig} onSelect={setSelected} />
        </div>
        <aside className="min-h-0 overflow-auto border-l p-3">
          <Inspector config={config} kind={kind} selected={selected} onChange={setConfig} onSelect={setSelected} />
        </aside>
      </main>

      {kind === 'global' ? (
        <section className="grid h-56 min-h-0 grid-cols-[minmax(0,1fr)_10rem_minmax(0,1fr)] border-b">
          <TreePanel title={t('tree.source')} root={roots.source} entries={entries} marks={marks} onEdit={editEntries} />
          <div className="flex flex-col justify-center gap-2 border-x p-3">
            <select aria-label={t('tree.watch')} className="h-8 rounded-md border bg-background px-2 text-sm" value={watch}
              onChange={(event) => setWatch(event.target.value)}>
              {watches.map((name) => <option key={name}>{name}</option>)}
            </select>
            <Button size="sm" onClick={run} disabled={!status.ok || !watches.includes(watch)}>{t('tree.trigger')} ▶</Button>
            {simulation && (
              <Button size="sm" variant="outline" onClick={() => editEntries(() => simulation.entries)}>{t('tree.keep')}</Button>
            )}
          </div>
          <TreePanel title={t('tree.target')} root={roots.target} entries={simulation?.entries ?? entries} marks={marks} onEdit={editEntries} />
        </section>
      ) : (
        <div />
      )}

      <Output fileName={fileName} text={text} status={status} simulation={simulation} />
    </div>
  )
}
