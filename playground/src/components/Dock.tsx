import { ChevronDown, ChevronRight } from 'lucide-react'
import { Fragment, type ReactNode, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { StageName } from '@/components/Fields'
import { Badge } from '@/components/ui/badge'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'
import type { Outcome, SimulatedStep, Simulation } from '../core'
import { changes, claimedBy } from '../steps'
import { Table, TableBody, TableCell, TableHead, TableHeader, TableRow } from '@/components/ui/table'

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
export function Dock({ trees, controls, fileName, text, status, simulation, tab, onTab }: {
  trees: TreeTab[]
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
    <Tabs value={tab} onValueChange={onTab} className="flex min-h-0 min-w-0 flex-col gap-0 border-t">
      <div className="flex flex-wrap items-center gap-2 border-b bg-muted/50 px-3 py-1.5">
        <TabsList variant="line">
          {trees.map((tree) => (
            <TabsTrigger key={tree.value} value={tree.value} className="gap-1.5">
              {tree.label}
              {tree.changed !== null && tree.changed > 0 && <Badge variant="secondary" className="h-4 px-1.5 text-[10px]">{tree.changed}</Badge>}
            </TabsTrigger>
          ))}
          <TabsTrigger value="results">{t('output.results')}</TabsTrigger>
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
          <Table className="font-mono text-xs">
            <TableHeader>
              <TableRow>
                <TableHead>{t('output.file')}</TableHead><TableHead>{t('output.became')}</TableHead>
                <TableHead>{t('output.to')}</TableHead><TableHead>{t('output.why')}</TableHead>
              </TableRow>
            </TableHeader>
            <TableBody>
              {simulation.outcomes.map((outcome) => (
                <Fragment key={outcome.origin}>
                  <TableRow>
                    <TableCell>
                      <button className="flex items-center gap-1 text-left" aria-expanded={open === outcome.origin}
                        disabled={outcome.steps.length === 0} onClick={() => setOpen(open === outcome.origin ? null : outcome.origin)}>
                        {open === outcome.origin ? <ChevronDown className="size-3.5 shrink-0" /> : <ChevronRight className="size-3.5 shrink-0" />}
                        {outcome.origin}
                      </button>
                    </TableCell>
                    <TableCell>{t(`what.${outcome.what}`, outcome.what)}</TableCell>
                    <TableCell>{outcome.to}</TableCell><TableCell>{outcome.reason}</TableCell>
                  </TableRow>
                  {open === outcome.origin && (
                    <TableRow ref={(row) => row?.scrollIntoView({ block: 'nearest' })}><TableCell colSpan={4}><Timeline outcome={outcome} /></TableCell></TableRow>
                  )}
                </Fragment>
              ))}
            </TableBody>
          </Table>
        )}
      </TabsContent>
      <TabsContent value="config" className="min-h-0 overflow-auto px-3">
        <pre className={cn('p-2 text-xs whitespace-pre-wrap break-all', !status.ok && 'text-destructive')}>{text}</pre>
        {status.lines.map((line) => (
          <p key={line} className={status.ok ? 'px-2 text-xs text-amber-600' : 'px-2 text-xs text-destructive'}>{line}</p>
        ))}
      </TabsContent>
    </Tabs>
  )
}
