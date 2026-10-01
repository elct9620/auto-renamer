import { useTranslation } from 'react-i18next'

import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'
import type { Simulation } from '../core'

export function Output({ fileName, text, status, simulation }: {
  fileName: string
  text: string
  status: { ok: boolean; lines: string[] }
  simulation: Simulation | null
}) {
  const { t } = useTranslation()
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
                <tr key={outcome.origin} className="border-t">
                  <td className="p-1">{outcome.origin}</td><td className="p-1">{t(`what.${outcome.what}`, outcome.what)}</td>
                  <td className="p-1">{outcome.to}</td><td className="p-1">{outcome.reason}</td>
                </tr>
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
