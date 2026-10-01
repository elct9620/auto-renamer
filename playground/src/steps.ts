// What each step of a file's planning changed, for the page to show only that.

import type { SimulatedStep } from './core'

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
