import { Fragment } from 'react'
import { useTranslation } from 'react-i18next'

import type { Simulation } from '../core'
import { type Change, type StageView, afterStage } from '../steps'
import { TableHead } from './TableHead'

/** One field a stage changed: its name, the value before and the value after. */
function ChangeCells({ change }: { change: Change }) {
  return (
    <>
      <td className="p-1">{change.key}</td>
      <td className="p-1 break-all text-muted-foreground">{change.from}</td>
      <td className="p-1 break-all">{change.to}</td>
    </>
  )
}

/** The files a simulation ran through a stage, each with every field the stage changed, before and after; a
 * refused file says why instead. Paths are shown whole, wrapping where they must. */
export function StageFiles({ simulation, stage }: { simulation: Simulation; stage: StageView }) {
  const { t } = useTranslation()
  const { ran, taken } = afterStage(simulation.outcomes, stage.pipeline, stage.index, stage.folder, stage.claims)
  if (ran.length === 0) return <p className="p-2 text-xs text-muted-foreground">{t('output.none')}</p>
  return (
    <>
      <table className="w-full text-left font-mono text-xs">
        <TableHead columns={[t('output.file'), t('output.field'), t('output.before'), t('output.after')]} />
        <tbody>
          {ran.map(({ origin, step, changes }) => {
            const rows = step.stop !== null || changes.length === 0 ? 1 : changes.length
            return (
              <Fragment key={origin}>
                <tr className="border-t align-top">
                  <td className="p-1 break-all" rowSpan={rows}>{origin}</td>
                  {step.stop !== null ? (
                    <td className="p-1 text-destructive" colSpan={3}>{step.stop}</td>
                  ) : changes.length === 0 ? (
                    <td className="p-1 text-muted-foreground" colSpan={3}>{t('output.unchanged')}</td>
                  ) : (
                    <ChangeCells change={changes[0]} />
                  )}
                </tr>
                {step.stop === null && changes.slice(1).map((change) => (
                  <tr key={change.key} className="align-top"><ChangeCells change={change} /></tr>
                ))}
              </Fragment>
            )
          })}
        </tbody>
      </table>
      {taken.length > 0 && <p className="p-2 text-xs text-muted-foreground">{t('output.taken', { count: taken.length })}</p>}
    </>
  )
}
