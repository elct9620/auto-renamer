// The virtual tree as folders: the flat entries the core reads, drawn and edited as the source and the
// targets of a watch.

import { type Table, asTable, routesOf, targetsOf } from './config'
import type { Entry, Simulation } from './core'

/** The name a folder configuration goes by in the tree, as the CLI looks for it. */
export const FOLDER_CONFIGURATION = 'auto-renamer.toml'

export interface TreeNode {
  name: string
  path: string
  folder: boolean
  children: TreeNode[]
}

/** The source of a watch and the roots its routes move files to, their rejected routes' included; a route
 * without a target renames in place, under the source. */
export function rootsOf(config: Table, watch: string): { source: string; targets: string[] } {
  const declared = asTable(asTable(config.watch)[watch])
  const source = typeof declared.source === 'string' ? declared.source : ''
  const named = targetsOf(config)
  const rootOf = (move: unknown) => (typeof move === 'string' && named[move] !== undefined ? named[move] : source)
  const targets = routesOf(config, watch).flatMap((route) => [
    rootOf(route.move),
    ...(route.rejected === undefined ? [] : [rootOf(asTable(route.rejected).move)]),
  ])
  return { source, targets: targets.length ? [...new Set(targets)] : [source] }
}

function within(path: string, folder: string): boolean {
  return path.startsWith(`${folder}/`)
}

/** What lies under a root, folders before files and each by name. */
export function treeOf(entries: Entry[], root: string): TreeNode[] {
  const top: TreeNode[] = []
  const folders = new Map<string, TreeNode>()
  const childrenOf = (path: string): TreeNode[] => (path === root ? top : folderAt(path).children)
  const folderAt = (path: string): TreeNode => {
    let folder = folders.get(path)
    if (folder === undefined) {
      const parent = path.slice(0, path.lastIndexOf('/'))
      folder = { name: path.slice(parent.length + 1), path, folder: true, children: [] }
      folders.set(path, folder)
      childrenOf(parent).push(folder)
    }
    return folder
  }
  for (const entry of entries.filter((entry) => within(entry.path, root))) {
    if (entry.folder) {
      folderAt(entry.path)
    } else {
      const parent = entry.path.slice(0, entry.path.lastIndexOf('/'))
      childrenOf(parent).push({ name: entry.path.slice(parent.length + 1), path: entry.path, folder: false, children: [] })
    }
  }
  const sort = (nodes: TreeNode[]): TreeNode[] =>
    nodes
      .sort((a, b) => Number(b.folder) - Number(a.folder) || a.name.localeCompare(b.name))
      .map((node) => ({ ...node, children: sort(node.children) }))
  return sort(top)
}

/** The folders under a root, written relative to it and ending in `/`, for a new path to start from. */
export function foldersUnder(entries: Entry[], root: string): string[] {
  const folders = new Set<string>()
  for (const entry of entries.filter((entry) => within(entry.path, root))) {
    const parts = entry.path.slice(root.length + 1).split('/')
    const depth = entry.folder ? parts.length : parts.length - 1
    for (let end = 1; end <= depth; end += 1) folders.add(`${parts.slice(0, end).join('/')}/`)
  }
  return [...folders].sort()
}

/** Adds a file or a folder, replacing whatever was at its path. */
export function addEntry(entries: Entry[], path: string, folder: boolean): Entry[] {
  return [...entries.filter((entry) => entry.path !== path), { path, folder, modified: Date.now(), text: '' }]
}

/** Writes a file's text, adding the file when nothing is at its path yet. */
export function writeFile(entries: Entry[], path: string, text: string): Entry[] {
  const existing = entries.find((entry) => entry.path === path)
  return [
    ...entries.filter((entry) => entry.path !== path),
    { path, folder: false, modified: existing?.modified ?? Date.now(), text },
  ]
}

/** Adds a folder configuration to a folder, starting from `text`; one the folder already holds is kept as it is. */
export function startConfiguration(entries: Entry[], folder: string, text: string): Entry[] {
  const path = `${folder}/${FOLDER_CONFIGURATION}`
  return entries.some((entry) => entry.path === path) ? entries : writeFile(entries, path, text)
}

/** Removes a file, or a folder with everything it holds. */
export function removeEntry(entries: Entry[], path: string): Entry[] {
  return entries.filter((entry) => entry.path !== path && !within(entry.path, path))
}

/** Gives a file or a folder a new name in the same place; a folder carries what it holds. */
export function renameEntry(entries: Entry[], path: string, name: string): Entry[] {
  const renamed = `${path.slice(0, path.lastIndexOf('/'))}/${name}`
  return entries.map((entry) =>
    entry.path === path || within(entry.path, path)
      ? { ...entry, path: renamed + entry.path.slice(path.length) }
      : entry)
}

/** What a simulation said of each path: a file where it started, and where it arrived. */
export function marksOf(simulation: Simulation | null): Record<string, string> {
  const marks: Record<string, string> = {}
  for (const outcome of simulation?.outcomes ?? []) {
    marks[outcome.origin] = outcome.what
    if (outcome.to !== null) marks[outcome.to] = outcome.what
  }
  return marks
}
