// The core of auto-renamer, compiled to WebAssembly, with the shapes its answers come in.

import * as wasm from './wasm/auto_renamer_wasm.js'
import type { Table } from './config'

export type Kind = 'global' | 'folder'

/** One file or folder of the virtual tree; `modified` is milliseconds since the Unix epoch. */
export interface Entry {
  path: string
  folder: boolean
  modified: number
  text: string
}

export interface Outcome {
  origin: string
  what: string
  to: string | null
  reason: string | null
}

export interface Simulation {
  outcomes: Outcome[]
  entries: Entry[]
}

export const init = wasm.default
export const initSync = wasm.initSync

export function check(kind: Kind, text: string): string[] {
  return wasm.check(kind, text)
}

export function read(text: string): Table {
  return wasm.read(text)
}

export function render(table: Table): string {
  return wasm.render(table)
}

export function simulate(config: string, watch: string, entries: Entry[]): Simulation {
  return wasm.simulate(config, watch, entries)
}
