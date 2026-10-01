import { Button } from '@/components/ui/button'
import { stages } from '../core'

export function Palette({ global, onAdd }: { global: boolean; onAdd: (table: 'watch' | 'pipeline') => void }) {
  return (
    <div className="grid content-start gap-4 p-3">
      <div className="grid gap-2">
        {global && <Button variant="outline" size="sm" onClick={() => onAdd('watch')}>Add watch</Button>}
        <Button variant="outline" size="sm" onClick={() => onAdd('pipeline')}>Add pipeline</Button>
      </div>
      <div className="grid gap-1">
        <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">Stages</h2>
        {stages().map(({ name }) => (
          <div key={name} className="rounded-md border bg-card px-2 py-1 font-mono text-xs">{name}</div>
        ))}
      </div>
    </div>
  )
}
