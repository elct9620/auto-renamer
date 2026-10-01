import { useTranslation } from 'react-i18next'

import { STAGE_DRAG } from '@/components/Canvas'
import { StageName } from '@/components/Fields'
import { KIND_ICONS, stageIcon } from '@/components/kinds'
import { Button } from '@/components/ui/button'
import { Tooltip, TooltipContent, TooltipTrigger } from '@/components/ui/tooltip'
import { stages } from '../core'

function StageIcon({ name }: { name: string }) {
  const Icon = stageIcon(name)
  return <Icon className="size-4 text-muted-foreground" />
}

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
          <Tooltip key={name}>
            <TooltipTrigger asChild>
              <div draggable className="flex cursor-grab items-center gap-1.5 rounded-md border bg-card px-2 py-1.5 text-sm active:cursor-grabbing"
                onDragStart={(event) => {
                  event.dataTransfer.setData(STAGE_DRAG, name)
                  event.dataTransfer.effectAllowed = 'copy'
                }}><StageIcon name={name} /><StageName name={name} /></div>
            </TooltipTrigger>
            <TooltipContent side="right" className="max-w-64">{t(`stages.${name}.about`)}</TooltipContent>
          </Tooltip>
        ))}
      </div>
    </div>
  )
}
