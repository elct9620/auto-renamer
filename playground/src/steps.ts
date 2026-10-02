// What each step of a file's planning changed, for the page to show only that.

import { type Stage, stageName } from './config'
import type { Outcome, SimulatedStep } from './core'

/** The pipeline that claimed a file at a step; none when a rejected route without a pipeline claimed it. */
export function claimedBy(step: SimulatedStep): string | null {
  return step.pipeline === '' ? null : step.pipeline
}

/** One thing a step changed: a field, or the plan under the name `plan`; `from` is absent for a new value. */
export interface Change {
  key: string
  from?: string
  to: string
}

/** What a step changed from the step before it; the claim and a stopped step change nothing. */
export function changes(before: SimulatedStep | undefined, step: SimulatedStep): Change[] {
  if (before === undefined || step.stop !== null) return []
  const found: Change[] = []
  for (const [key, to] of Object.entries(step.fields)) {
    const from = before.fields[key]
    if (from !== to) found.push(from === undefined ? { key, to } : { key, from, to })
  }
  // The plan follows `name` and `ext`, so it is told only when a path stage moved it elsewhere.
  const named = found.some((change) => change.key === 'name' || change.key === 'ext')
  if (step.plan !== null && step.plan !== before.plan && !named) {
    found.push(before.plan === null ? { key: 'plan', to: step.plan } : { key: 'plan', from: before.plan, to: step.plan })
  }
  return found
}

/** One file as a stage left it: what it held then and what that stage changed. */
export interface AtStage {
  origin: string
  step: SimulatedStep
  changes: Change[]
}

/**
 * The files the stage at `index` of `pipeline` ran on, as that stage left them. `folder` is the folder whose
 * configuration is being edited, relative to the source, or null for the global configuration: a stage of the
 * global configuration runs on the files no folder configuration took from it, and lists those taken apart; a
 * stage of a folder configuration runs on the files under that folder whose pipeline it replaced. A leading
 * filter claims rather than runs, so with `claims` the stage stands for the claim and lists the files claimed.
 */
export function afterStage(outcomes: Outcome[], pipeline: string, index: number, folder: string | null, claims = false): {
  ran: AtStage[]
  taken: string[]
} {
  const ran: AtStage[] = []
  const taken: string[] = []
  const under = folder ? `${folder}/` : ''
  for (const outcome of outcomes) {
    const at = outcome.steps.findIndex((step) => step.pipeline === pipeline && step.stage === (claims ? null : index))
    if (at < 0) continue
    const ours = folder === null ? !outcome.replaced : outcome.replaced && outcome.origin.startsWith(under)
    if (ours) ran.push({ origin: outcome.origin, step: outcome.steps[at], changes: changes(outcome.steps[at - 1], outcome.steps[at]) })
    else if (folder === null) taken.push(outcome.origin)
  }
  return { ran, taken }
}

/** Whether the stage at `index` stands for the claim: it and every stage before it are filters. */
export function claims(stages: Stage[], index: number): boolean {
  return stages.slice(0, index + 1).every((one) => stageName(one) === 'filter')
}

/** A stage whose files the panel lists: where it sits, and whether it stands for the claim. */
export interface StageView {
  pipeline: string
  index: number
  name: string
  /** The folder whose configuration is edited, relative to the source; null for the global configuration. */
  folder: string | null
  claims: boolean
}
