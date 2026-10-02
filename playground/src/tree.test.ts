import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import type { Entry } from './core'
import { read, simulate } from './core'
import { foldersUnder, marksOf, removeEntry, renameEntry, rootsOf, treeOf } from './tree'

function file(path: string): Entry {
  return { path, folder: false, modified: 0, text: '' }
}

function paths(entries: Entry[]): string[] {
  return entries.map((entry) => entry.path)
}

describe('drawing the tree', () => {
  // @behavior PGE-017
  it('shows the source and the target of a watch', () => {
    const config = read('[pipeline.p]\nstages = []\n\n[target.video]\npath = "/video"\n\n[watch.series]\nsource = "/downloads"\nroutes = [{ pipeline = "p", move = "video" }]')
    const entries = [file('/downloads/x.mkv'), file('/video/y.mkv')]
    const { source, targets } = rootsOf(config, 'series')

    expect(treeOf(entries, source).map((node) => node.path)).toEqual(['/downloads/x.mkv'])
    expect(targets).toEqual(['/video'])
    expect(treeOf(entries, targets[0]).map((node) => node.path)).toEqual(['/video/y.mkv'])
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

describe('marking a simulation', () => {
  // @behavior PGE-021
  it('marks a file the simulation moved where it arrived', () => {
    const config = `[pipeline.p]\nstages = [{ format = "Alpha" }]\n\n[target.video]\npath = "/video"\n\n[watch.w]\nsource = "/downloads"\nroutes = [{ pipeline = "p", move = "video" }]\nunit = "directory"\n`

    const marks = marksOf(simulate(config, 'w', [file('/downloads/Alpha/x.mkv')]))

    expect(marks['/video/Alpha/Alpha.mkv']).toBe('moved')
  })
})

describe('adding to the tree', () => {
  // @behavior PGE-031
  it('offers the folders already in the tree for a new path', () => {
    const entries = [file('/downloads/Alpha/Season 1/01.mkv'), file('/video/Beta/02.mkv')]

    expect(foldersUnder(entries, '/downloads')).toEqual(['Alpha/', 'Alpha/Season 1/'])
  })
})
