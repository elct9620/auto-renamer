import { ChevronDown, ChevronRight } from 'lucide-react'
import { Fragment, useState } from 'react'
import { useTranslation } from 'react-i18next'

import { StageName } from '@/components/Fields'
import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'
import type { Outcome, Simulation } from '../core'
import { changes } from '../steps'

/** How the pipeline planned one file: the claim, then each stage it reached with only what that stage changed. */
function Timeline({ outcome }: { outcome: Outcome }) {
  const { t } = useTranslation()
  return (
    <ol aria-label={t('output.steps', { file: outcome.origin })} className="grid gap-1 py-1 pl-6">
      {outcome.steps.map((step, index) => (
        <li key={`${step.pipeline}:${step.stage}`} className="flex flex-wrap items-baseline gap-x-2">
          <span className="font-sans font-medium">
            {step.name === null ? t('output.claimed', { pipeline: step.pipeline }) : <StageName name={step.name} />}
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

export function Output({ fileName, text, status, simulation }: {
  fileName: string
  text: string
  status: { ok: boolean; lines: string[] }
  simulation: Simulation | null
}) {
  const { t } = useTranslation()
  const [open, setOpen] = useState<string | null>(null)
  return (
    <Tabs defaultValue="results" className="flex min-h-0 min-w-0 flex-col p-3">
      <TabsList>
        <TabsTrigger value="results">{t('output.results')}</TabsTrigger>
        <TabsTrigger value="config">{fileName}</TabsTrigger>
      </TabsList>
      <TabsContent value="results" className="min-h-0 overflow-auto">
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
      <TabsContent value="config" className="min-h-0 overflow-auto">
        <pre className={cn('p-2 text-xs whitespace-pre-wrap break-all', !status.ok && 'text-destructive')}>{text}</pre>
        {status.lines.map((line) => (
          <p key={line} className={status.ok ? 'px-2 text-xs text-amber-600' : 'px-2 text-xs text-destructive'}>{line}</p>
        ))}
      </TabsContent>
    </Tabs>
  )
}
