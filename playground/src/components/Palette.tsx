import { GripVertical } from 'lucide-react'
import { useTranslation } from 'react-i18next'

import { STAGE_DRAG } from '@/components/Canvas'
import { KIND_ICONS } from '@/components/kinds'
import { Button } from '@/components/ui/button'
import { stages } from '../core'

export function Palette({ global, onAdd }: { global: boolean; onAdd: (table: 'watch' | 'pipeline') => void }) {
  const { t } = useTranslation()
  return (
    <div className="grid content-start gap-4 p-3">
      <div className="grid gap-2">
        {global && <Button variant="outline" size="sm" onClick={() => onAdd('watch')}><KIND_ICONS.watch />{t('palette.addWatch')}</Button>}
        <Button variant="outline" size="sm" onClick={() => onAdd('pipeline')}><KIND_ICONS.pipeline />{t('palette.addPipeline')}</Button>
      </div>
      <div className="grid gap-1">
        <h2 className="text-xs font-semibold uppercase tracking-wide text-muted-foreground">{t('palette.stages')}</h2>
        <p className="text-xs text-muted-foreground">{t('palette.hint')}</p>
        {stages().map(({ name }) => (
          <div key={name} draggable className="flex cursor-grab items-center gap-1.5 rounded-md border bg-card px-2 py-1 font-mono text-xs active:cursor-grabbing"
            onDragStart={(event) => {
              event.dataTransfer.setData(STAGE_DRAG, name)
              event.dataTransfer.effectAllowed = 'copy'
            }}><GripVertical className="size-3.5 text-muted-foreground" />{name}</div>
        ))}
      </div>
    </div>
  )
}
