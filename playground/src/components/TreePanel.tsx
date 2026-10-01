import { File, Folder, X } from 'lucide-react'
import { useState } from 'react'

import { Button } from '@/components/ui/button'
import { Input } from '@/components/ui/input'
import type { Entry } from '../core'
import { type TreeNode, addEntry, removeEntry, treeOf } from '../tree'

function Branch({ nodes, onRemove }: { nodes: TreeNode[]; onRemove: (path: string) => void }) {
  return (
    <ul className="grid gap-0.5 pl-3">
      {nodes.map((node) => (
        <li key={node.path}>
          <div className="group flex items-center gap-1.5 rounded px-1 text-sm hover:bg-muted">
            {node.folder ? <Folder className="size-3.5 text-muted-foreground" /> : <File className="size-3.5 text-muted-foreground" />}
            <span className="truncate font-mono text-xs">{node.name}</span>
            <Button variant="ghost" size="icon-xs" className="ml-auto opacity-0 group-hover:opacity-100"
              aria-label={`Remove ${node.name}`} onClick={() => onRemove(node.path)}>
              <X />
            </Button>
          </div>
          {node.children.length > 0 && <Branch nodes={node.children} onRemove={onRemove} />}
        </li>
      ))}
    </ul>
  )
}

export function TreePanel({ title, root, entries, onChange }: {
  title: string
  root: string
  entries: Entry[]
  onChange: (entries: Entry[]) => void
}) {
  const [name, setName] = useState('')
  const path = `${root}/${name}`
  return (
    <div className="flex min-h-0 flex-col gap-2 p-3">
      <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">
        {title} <span className="font-mono normal-case">{root}</span>
      </h2>
      <div className="min-h-0 flex-1 overflow-auto">
        <Branch nodes={treeOf(entries, root)} onRemove={(removed) => onChange(removeEntry(entries, removed))} />
      </div>
      <div className="flex gap-2">
        <Input className="h-7 font-mono text-xs" placeholder="Show/Season 1/file.mkv" value={name}
          onChange={(event) => setName(event.target.value)} />
        <Button size="sm" variant="outline" disabled={name.trim() === '' || root === ''} onClick={() => {
          onChange(addEntry(entries, path, false))
          setName('')
        }}>Add file</Button>
      </div>
    </div>
  )
}
