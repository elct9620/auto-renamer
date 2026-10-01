import { Tabs, TabsContent, TabsList, TabsTrigger } from '@/components/ui/tabs'
import { cn } from '@/lib/utils'
import type { Simulation } from '../core'

export function Output({ fileName, text, status, simulation }: {
  fileName: string
  text: string
  status: { ok: boolean; lines: string[] }
  simulation: Simulation | null
}) {
  return (
    <Tabs defaultValue="results" className="flex min-h-0 min-w-0 flex-col p-3">
      <TabsList>
        <TabsTrigger value="results">Results</TabsTrigger>
        <TabsTrigger value="config">{fileName}</TabsTrigger>
      </TabsList>
      <TabsContent value="results" className="min-h-0 overflow-auto">
        {simulation === null ? (
          <p className="p-2 text-xs text-muted-foreground">Trigger the watch to see where each file goes.</p>
        ) : (
          <table className="w-full text-left font-mono text-xs">
            <thead className="text-muted-foreground">
              <tr><th className="p-1">File</th><th className="p-1">Became</th><th className="p-1">To</th><th className="p-1">Why</th></tr>
            </thead>
            <tbody>
              {simulation.outcomes.map((outcome) => (
                <tr key={outcome.origin} className="border-t">
                  <td className="p-1">{outcome.origin}</td><td className="p-1">{outcome.what}</td>
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
