import { File, FileCog, FilePlus, Folder, FolderPlus, Plus, X } from 'lucide-react'
import { useId, useRef, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { Badge } from '@/components/ui/badge'
import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import { Textarea } from '@/components/ui/textarea'
import type { Entry } from '../core'
import { type TreeNode, addEntry, foldersUnder, removeEntry, renameEntry, treeOf } from '../tree'

const FOLDER_CONFIGURATION = 'auto-renamer.toml'

type Edit = (change: (entries: Entry[]) => Entry[]) => void

/** A name for one place in the tree: never empty and never a path of its own. */
function isName(name: string): boolean {
  return name.trim() !== '' && !name.includes('/')
}

function Row({ node, entries, mark, onEdit, onStart }: {
  node: TreeNode
  entries: Entry[]
  mark?: string
  onEdit: Edit
  onStart: (folder: string) => void
}) {
  const { t } = useTranslation()
  const [renaming, setRenaming] = useState(false)
  const [open, setOpen] = useState(false)
  const configuration = node.name === FOLDER_CONFIGURATION
  const Icon = node.folder ? Folder : configuration ? FileCog : File
  const text = entries.find((entry) => entry.path === node.path)?.text ?? ''
  return (
    <>
      <div className="group flex items-center gap-1.5 rounded px-1 hover:bg-muted">
        <Icon className="size-3.5 shrink-0 text-muted-foreground" />
        {renaming ? (
          <Input autoFocus className="h-6 font-mono text-xs" defaultValue={node.name}
            onBlur={(event) => {
              setRenaming(false)
              if (isName(event.target.value) && event.target.value !== node.name) {
                onEdit((all) => renameEntry(all, node.path, event.target.value.trim()))
              }
            }}
            onKeyDown={(event) => event.key === 'Enter' && event.currentTarget.blur()} />
        ) : (
          <button className="truncate text-left font-mono text-xs" title={t('tree.rename')}
            onClick={() => configuration && setOpen(!open)} onDoubleClick={() => setRenaming(true)}>
            {node.name}
          </button>
        )}
        {mark && <Badge variant={mark.startsWith('moved') || mark === 'previewed' ? 'secondary' : 'outline'}>{t(`what.${mark}`, mark)}</Badge>}
        <span className="ml-auto flex opacity-0 group-hover:opacity-100 focus-within:opacity-100">
          {node.folder && (
            <Button variant="ghost" size="icon-xs" aria-label={t('tree.addHere', { name: node.name })} onClick={() => onStart(node.path)}>
              <Plus />
            </Button>
          )}
          <Button variant="ghost" size="icon-xs" aria-label={t('tree.remove', { name: node.name })}
            onClick={() => onEdit((all) => removeEntry(all, node.path))}>
            <X />
          </Button>
        </span>
      </div>
      {configuration && open && (
        <Textarea className="my-1 font-mono text-xs" rows={3} defaultValue={text} placeholder={'[vars]\nshow = "Beta"'}
          onBlur={(event) => onEdit((all) => all.map((entry) =>
            entry.path === node.path ? { ...entry, text: event.target.value } : entry))} />
      )}
    </>
  )
}

function Branch({ nodes, entries, marks, onEdit, onStart }: {
  nodes: TreeNode[]
  entries: Entry[]
  marks: Record<string, string>
  onEdit: Edit
  onStart: (folder: string) => void
}) {
  return (
    <ul className="grid gap-0.5 pl-3">
      {nodes.map((node) => (
        <li key={node.path}>
          <Row node={node} entries={entries} mark={marks[node.path]} onEdit={onEdit} onStart={onStart} />
          {node.children.length > 0 && <Branch nodes={node.children} entries={entries} marks={marks} onEdit={onEdit} onStart={onStart} />}
        </li>
      ))}
    </ul>
  )
}

/** One root of the virtual tree. It shows `entries`, and every edit is made to the tree the page keeps. */
export function TreePanel({ title, root, entries, marks, onEdit }: {
  title: string
  root: string
  entries: Entry[]
  marks: Record<string, string>
  onEdit: Edit
}) {
  const { t } = useTranslation()
  const [path, setPath] = useState('')
  const input = useRef<HTMLInputElement>(null)
  const folders = useId()
  const start = (folder: string) => {
    setPath(`${folder.slice(root.length + 1)}/`)
    input.current?.focus()
  }
  const valid = root !== '' && path.split('/').every(isName)
  const add = (folder: boolean) => {
    onEdit((all) => addEntry(all, `${root}/${path.trim()}`, folder))
    setPath('')
  }
  return (
    <div className="flex min-h-0 flex-col gap-2 p-3">
      <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        {title} <span className="font-mono normal-case">{root}</span>
      </h2>
      <div className="min-h-0 flex-1 overflow-auto">
        <Branch nodes={treeOf(entries, root)} entries={entries} marks={marks} onEdit={onEdit} onStart={start} />
      </div>
      <div className="flex gap-2">
        <Input ref={input} list={folders} aria-label={t('tree.path')} className="h-7 font-mono text-xs" placeholder={t('tree.placeholder', { path: 'Show/Season 1/file.mkv', configuration: `Show/${FOLDER_CONFIGURATION}` })}
          value={path} onChange={(event) => setPath(event.target.value)} />
        <datalist id={folders}>
          {foldersUnder(entries, root).map((folder) => <option key={folder} value={folder} />)}
        </datalist>
        <Button size="sm" variant="outline" disabled={!valid} onClick={() => add(false)}><FilePlus />{t('tree.addFile')}</Button>
        <Button size="sm" variant="outline" disabled={!valid} onClick={() => add(true)}><FolderPlus />{t('tree.addFolder')}</Button>
      </div>
    </div>
  )
}
