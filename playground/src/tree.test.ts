import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import type { Entry } from './core'
import { read } from './core'
import { removeEntry, renameEntry, rootsOf, treeOf } from './tree'

function file(path: string): Entry {
  return { path, folder: false, modified: 0, text: '' }
}

function paths(entries: Entry[]): string[] {
  return entries.map((entry) => entry.path)
}

describe('drawing the tree', () => {
  // @behavior PGE-017
  it('shows the source and the target of a watch', () => {
    const config = read('[watch.series]\nsource = "/downloads"\ntarget = "/video"')
    const entries = [file('/downloads/x.mkv'), file('/video/y.mkv')]
    const { source, target } = rootsOf(config, 'series')

    expect(treeOf(entries, source).map((node) => node.path)).toEqual(['/downloads/x.mkv'])
    expect(treeOf(entries, target).map((node) => node.path)).toEqual(['/video/y.mkv'])
  })

  // @behavior PGE-018
  it('shows a file inside its folders', () => {
    const [folder] = treeOf([file('/downloads/Alpha/x.mkv')], '/downloads')

    expect(folder).toMatchObject({ name: 'Alpha', folder: true })
    expect(folder.children.map((node) => node.name)).toEqual(['x.mkv'])
  })
})

describe('editing the tree', () => {
  // @behavior PGE-019
  it('removes a folder with what it holds', () => {
    const entries = [file('/downloads/Alpha/x.mkv'), file('/downloads/Beta/y.mkv')]

    expect(paths(removeEntry(entries, '/downloads/Alpha'))).toEqual(['/downloads/Beta/y.mkv'])
  })

  // @behavior PGE-020
  it('carries what a renamed folder holds', () => {
    const entries = [file('/downloads/Alpha/x.mkv'), file('/downloads/Alphabet/y.mkv')]

    expect(paths(renameEntry(entries, '/downloads/Alpha', 'Beta'))).toEqual(['/downloads/Beta/x.mkv', '/downloads/Alphabet/y.mkv'])
  })
})
