import { Fragment } from 'react'
import { useTranslation } from 'react-i18next'

import type { Simulation } from '../core'
import { type Change, type StageView, afterStage } from '../steps'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

/** One field a stage changed: its name, the value before and the value after. */
function ChangeCells({ change }: { change: Change }) {
  return (
    <>
      <TableCell>{change.key}</TableCell>
      <TableCell className="whitespace-normal break-all text-muted-foreground">{change.from}</TableCell>
      <TableCell className="whitespace-normal break-all">{change.to}</TableCell>
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
      <Table className="font-mono text-xs">
        <TableHeader>
          <TableRow>
            <TableHead className="w-1/2">{t('output.file')}</TableHead><TableHead>{t('output.field')}</TableHead>
            <TableHead>{t('output.before')}</TableHead><TableHead>{t('output.after')}</TableHead>
          </TableRow>
        </TableHeader>
        <TableBody>
          {ran.map(({ origin, step, changes }) => {
            const rows = step.stop !== null || changes.length === 0 ? 1 : changes.length
            return (
              <Fragment key={origin}>
                <TableRow className="align-top">
                  <TableCell className="whitespace-normal break-all" rowSpan={rows}>{origin}</TableCell>
                  {step.stop !== null ? (
                    <TableCell className="whitespace-normal text-destructive" colSpan={3}>{step.stop}</TableCell>
                  ) : changes.length === 0 ? (
                    <TableCell className="text-muted-foreground" colSpan={3}>{t('output.unchanged')}</TableCell>
                  ) : (
                    <ChangeCells change={changes[0]} />
                  )}
                </TableRow>
                {step.stop === null && changes.slice(1).map((change) => (
                  <TableRow key={change.key} className="align-top"><ChangeCells change={change} /></TableRow>
                ))}
              </Fragment>
            )
          })}
        </TableBody>
      </Table>
      {taken.length > 0 && <p className="p-2 text-xs text-muted-foreground">{t('output.taken', { count: taken.length })}</p>}
    </>
  )
}
