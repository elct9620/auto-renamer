import { describe, expect, it } from 'vitest'

import './setup.test-helper'
import type { Entry } from './core'
import { read, simulate } from './core'
import { changedUnder, foldersUnder, marksOf, removeEntry, renameEntry, revertFiles, rootsOf, startConfiguration, treeOf } from './tree'

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

  // @behavior PGE-090
  it('takes the target of a rejected route as a root', () => {
    const config = read('[pipeline.p]\nstages = []\n\n[target.video]\npath = "/video"\n\n[target.conflict]\npath = "/conflict"\n\n[watch.series]\nsource = "/downloads"\nroutes = [{ pipeline = "p", move = "video", rejected = { move = "conflict" } }]')

    expect(rootsOf(config, 'series').targets).toEqual(['/video', '/conflict'])
  })

  // @behavior PGE-091
  it('opens a folder configuration already there as it is', () => {
    const existing = { path: '/downloads/Alpha/auto-renamer.toml', folder: false, modified: 0, text: 'vars = { show = "Beta" }\n' }

    const entries = startConfiguration([existing], '/downloads/Alpha', 'vars = { show = "Alpha" }\n')

    expect(entries).toEqual([existing])
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

describe('undoing a keep', () => {
  // @behavior PGE-112
  it('takes the files back and leaves the folder configurations as they are now', () => {
    const before = [file('/downloads/A/x.mkv'), file('/downloads/A/auto-renamer.toml'), file('/downloads/B/auto-renamer.toml')]
    const now = [file('/downloads/A/A.mkv'), { ...file('/downloads/A/auto-renamer.toml'), text: 'edited' }, file('/downloads/C/auto-renamer.toml')]

    const reverted = revertFiles(now, before)

    expect(paths(reverted).sort()).toEqual(['/downloads/A/auto-renamer.toml', '/downloads/A/x.mkv', '/downloads/C/auto-renamer.toml'])
    expect(reverted.find((entry) => entry.path === '/downloads/A/auto-renamer.toml')?.text).toBe('edited')
  })
})

describe('marking a simulation', () => {
  // @behavior PGE-021
  it('marks a file the simulation moved where it arrived', () => {
    const config = `[pipeline.p]\nstages = [{ format = "Alpha" }]\n\n[target.video]\npath = "/video"\n\n[watch.w]\nsource = "/downloads"\nroutes = [{ pipeline = "p", move = "video" }]\nunit = "directory"\n`

    const marks = marksOf(simulate(config, 'w', [file('/downloads/Alpha/x.mkv')]), '/downloads')

    expect(marks['/video/Alpha/Alpha.mkv']).toBe('moved')
  })

  // @behavior PGE-096
  it('marks a file the simulation took where it was', () => {
    const config = `[pipeline.p]\nstages = [{ format = "Alpha" }]\n\n[target.video]\npath = "/video"\n\n[watch.w]\nsource = "/downloads"\nroutes = [{ pipeline = "p", move = "video" }]\nunit = "directory"\n`

    const marks = marksOf(simulate(config, 'w', [file('/downloads/Alpha/x.mkv')]), '/downloads')

    expect(marks['/downloads/Alpha/x.mkv']).toBe('moved')
  })

  // @behavior PGE-097
  it('counts the files a simulation changed under each root', () => {
    const config = `[pipeline.p]\nstages = [{ filter = { ext = ["mkv"] } }, { format = "Alpha" }]\n\n[target.video]\npath = "/video"\n\n[watch.w]\nsource = "/downloads"\nroutes = [{ pipeline = "p", move = "video" }]\nunit = "directory"\n`

    const simulation = simulate(config, 'w', [file('/downloads/Alpha/x.mkv'), file('/downloads/Alpha/notes.txt')])

    expect([changedUnder(simulation, '/downloads', '/downloads'), changedUnder(simulation, '/downloads', '/video')]).toEqual([1, 1])
  })
})

describe('adding to the tree', () => {
  // @behavior PGE-031
  it('offers the folders already in the tree for a new path', () => {
    const entries = [file('/downloads/Alpha/Season 1/01.mkv'), file('/video/Beta/02.mkv')]

    expect(foldersUnder(entries, '/downloads')).toEqual(['Alpha/', 'Alpha/Season 1/'])
  })
})
