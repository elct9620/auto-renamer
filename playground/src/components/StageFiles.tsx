import { useTranslation } from 'react-i18next'

import type { Simulation } from '../core'
import { type Change, type StageView, afterStage } from '../steps'

/** One field a stage changed: its name, then the value before and the value after, which wraps below when long. */
function ChangeLine({ change }: { change: Change }) {
  return (
    <li className="grid grid-cols-[auto_minmax(0,1fr)] gap-x-2">
      <span className="text-muted-foreground">{change.key}</span>
      <span className="flex flex-wrap gap-x-1.5 break-all">
        {change.from !== undefined && <><s className="text-muted-foreground">{change.from}</s><span aria-hidden>→</span></>}
        <span>{change.to}</span>
      </span>
    </li>
  )
}

/** The files a simulation ran through a stage, one card each: the file's path in full, then every field the stage
 * changed, before and after; a refused file says why instead. */
export function StageFiles({ simulation, stage }: { simulation: Simulation; stage: StageView }) {
  const { t } = useTranslation()
  const { ran, taken } = afterStage(simulation.outcomes, stage.pipeline, stage.index, stage.folder, stage.claims)
  if (ran.length === 0) return <p className="p-2 text-xs text-muted-foreground">{t('output.none')}</p>
  return (
    <>
      <ul className="grid divide-y font-mono text-xs">
        {ran.map(({ origin, step, changes }) => (
          <li key={origin} aria-label={origin} className="grid gap-1 py-2">
            <span className="break-all font-medium">{origin}</span>
            {step.stop !== null ? (
              <span className="pl-3 text-destructive">{step.stop}</span>
            ) : changes.length === 0 ? (
              <span className="pl-3 text-muted-foreground">{t('output.unchanged')}</span>
            ) : (
              <ul className="grid gap-0.5 pl-3">
                {changes.map((change) => <ChangeLine key={change.key} change={change} />)}
              </ul>
            )}
          </li>
        ))}
      </ul>
      {taken.length > 0 && <p className="p-2 text-xs text-muted-foreground">{t('output.taken', { count: taken.length })}</p>}
    </>
  )
}
