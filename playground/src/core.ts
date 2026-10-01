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

export type ParameterKind = 'text' | 'texts' | 'integer' | 'boolean'

export interface Parameter {
  name: string
  kind: ParameterKind
  required: boolean
  choices: string[]
}

/** How a stage is declared, as the core describes it. `example` is inline TOML; empty for the bare name. */
export interface Declaration {
  name: string
  bare: boolean
  value: ParameterKind | null
  parameters: Parameter[]
  values: boolean
  example: string
}

/** One step of a file's planning: the claim when `stage` is null, then each stage it reached by its place in
 * the pipeline, holding the plan and fields it went on with or why it stopped. */
export interface SimulatedStep {
  pipeline: string
  stage: number | null
  name: string | null
  plan: string | null
  fields: Record<string, string>
  stop: string | null
}

/** `replaced` says a folder configuration replaced the pipeline that planned the file. */
export interface Outcome {
  origin: string
  what: string
  to: string | null
  reason: string | null
  steps: SimulatedStep[]
  replaced: boolean
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

export function stages(): Declaration[] {
  return wasm.stages()
}

export function simulate(config: string, watch: string, entries: Entry[]): Simulation {
  return wasm.simulate(config, watch, entries)
}
