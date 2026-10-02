import { ChevronDown, ChevronRight } from 'lucide-react'
import { Fragment, type ReactNode, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { StageName } from '@/components/Fields'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'
import type { Outcome, SimulatedStep, Simulation } from '../core'
import { type StageView, afterStage, changes, claimedBy } from '../steps'

/** How the pipeline planned one file: the claim, then each stage it reached with only what that stage changed. */
function Timeline({ outcome }: { outcome: Outcome }) {
  const { t } = useTranslation()
  const claim = (step: SimulatedStep) => {
    const pipeline = claimedBy(step)
    return pipeline === null ? t('output.claimedRejected') : t('output.claimed', { pipeline })
  }
  return (
    <ol aria-label={t('output.steps', { file: outcome.origin })} className="grid gap-1 py-1 pl-6">
      {outcome.steps.map((step, index) => (
        <li key={`${step.pipeline}:${step.stage}`} className="flex flex-wrap items-baseline gap-x-2">
          <span className="font-sans font-medium">
            {step.name !== null ? <StageName name={step.name} /> : claim(step)}
          </span>
          {step.stop !== null && (
            <span className="text-destructive">{step.stop}</span>
          )}
          {changes(outcome.steps[index - 1], step).map((change) => (
            <span key={change.key}>
              {change.key}: {change.from !== undefined && <><s className="text-muted-foreground">{change.from}</s> → </>}{change.to}
            </span>
          ))}
        </li>
      ))}
    </ol>
  )
}

/** The files a simulation ran through a stage, each with every field the stage changed, before and after; a
 * refused file says why instead. Paths are shown whole, wrapping where they must. */
function StageFiles({ simulation, stage }: { simulation: Simulation; stage: StageView }) {
  const { t } = useTranslation()
  const { ran, taken } = afterStage(simulation.outcomes, stage.pipeline, stage.index, stage.folder, stage.claims)
  if (ran.length === 0) return <p className="p-2 text-xs text-muted-foreground">{t('output.none')}</p>
  return (
    <>
      <table className="w-full text-left font-mono text-xs">
        <thead className="text-muted-foreground">
          <tr><th className="p-1">{t('output.file')}</th><th className="p-1">{t('output.field')}</th><th className="p-1">{t('output.before')}</th><th className="p-1">{t('output.after')}</th></tr>
        </thead>
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
                    <><td className="p-1">{changes[0].key}</td><td className="p-1 break-all text-muted-foreground">{changes[0].from}</td><td className="p-1 break-all">{changes[0].to}</td></>
                  )}
                </tr>
                {step.stop === null && changes.slice(1).map((change) => (
                  <tr key={change.key} className="align-top">
                    <td className="p-1">{change.key}</td><td className="p-1 break-all text-muted-foreground">{change.from}</td><td className="p-1 break-all">{change.to}</td>
                  </tr>
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

/** One tree the panel offers: its tab and what the tab shows. */
export interface TreeTab {
  value: string
  label: string
  /** How many of its files the last simulation changed; none before one ran. */
  changed: number | null
  content: ReactNode
}

/** The panel under the canvas: a tab for each tree, each counting the files a simulation changed in it, and
 * tabs for the results and the configuration text. What triggers a simulation sits beside the tabs. */
export function Dock({ trees, controls, fileName, text, status, simulation, stage, tab, onTab }: {
  trees: TreeTab[]
  /** The stage selected on the canvas, whose files get a tab once a simulation ran. */
  stage: StageView | null
  controls: ReactNode
  fileName: string
  text: string
  status: { ok: boolean; lines: string[] }
  simulation: Simulation | null
  tab: string
  onTab: (tab: string) => void
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState<string | null>(null)
  return (
    <Tabs value={tab === 'stage' && !(simulation && stage) ? 'results' : tab} onValueChange={onTab} className="flex min-h-0 min-w-0 flex-col gap-0 border-t">
      <div className="flex flex-wrap items-center gap-2 border-b px-3 py-1.5">
        <TabsList>
          {trees.map((tree) => (
            <TabsTrigger key={tree.value} value={tree.value} className="gap-1.5">
              {tree.label}
              {tree.changed !== null && tree.changed > 0 && <Badge variant="secondary" className="h-4 px-1.5 text-[10px]">{tree.changed}</Badge>}
            </TabsTrigger>
          ))}
          <TabsTrigger value="results">{t('output.results')}</TabsTrigger>
          {simulation && stage && <TabsTrigger value="stage">{t('output.stage', { stage: stage.name })}</TabsTrigger>}
          <TabsTrigger value="config">{fileName}</TabsTrigger>
        </TabsList>
        <div className="ml-auto flex items-center gap-2">{controls}</div>
      </div>
      {trees.map((tree) => (
        <TabsContent key={tree.value} value={tree.value} className="min-h-0">{tree.content}</TabsContent>
      ))}
      <TabsContent value="results" className="min-h-0 overflow-auto px-3">
        {simulation === null ? (
          <p className="p-2 text-xs text-muted-foreground">{t('output.empty')}</p>
        ) : (
          <table className="w-full text-left font-mono text-xs">
            <thead className="text-muted-foreground">
              <tr><th className="p-1">{t('output.file')}</th><th className="p-1">{t('output.became')}</th><th className="p-1">{t('output.to')}</th><th className="p-1">{t('output.why')}</th></tr>
            </thead>
            <tbody>
              {simulation.outcomes.map((outcome) => (
                <Fragment key={outcome.origin}>
                  <tr className="border-t">
                    <td className="p-1">
                      <button className="flex items-center gap-1 text-left" aria-expanded={open === outcome.origin}
                        disabled={outcome.steps.length === 0} onClick={() => setOpen(open === outcome.origin ? null : outcome.origin)}>
                        {open === outcome.origin ? <ChevronDown className="size-3.5 shrink-0" /> : <ChevronRight className="size-3.5 shrink-0" />}
                        {outcome.origin}
                      </button>
                    </td>
                    <td className="p-1">{t(`what.${outcome.what}`, outcome.what)}</td>
                    <td className="p-1">{outcome.to}</td><td className="p-1">{outcome.reason}</td>
                  </tr>
                  {open === outcome.origin && (
                    <tr ref={(row) => row?.scrollIntoView({ block: 'nearest' })}><td colSpan={4}><Timeline outcome={outcome} /></td></tr>
                  )}
                </Fragment>
              ))}
            </tbody>
          </table>
        )}
      </TabsContent>
      {simulation && stage && (
        <TabsContent value="stage" className="min-h-0 overflow-auto px-3"><StageFiles simulation={simulation} stage={stage} /></TabsContent>
      )}
      <TabsContent value="config" className="min-h-0 overflow-auto px-3">
        <pre className={cn('p-2 text-xs whitespace-pre-wrap break-all', !status.ok && 'text-destructive')}>{text}</pre>
        {status.lines.map((line) => (
          <p key={line} className={status.ok ? 'px-2 text-xs text-amber-600' : 'px-2 text-xs text-destructive'}>{line}</p>
        ))}
      </TabsContent>
    </Tabs>
  )
}
