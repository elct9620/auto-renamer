import {
  ArrowUpToLine,
  CaseSensitive,
  CircleDashed,
  Copy,
  Equal,
  Eraser,
  Eye,
  FolderInput,
  FolderTree,
  Funnel,
  Hash,
  Layers,
  ListOrdered,
  ListPlus,
  type LucideIcon,
  PenLine,
  Regex,
  Replace,
  Route,
  Workflow,
} from 'lucide-react'

/** The icon each kind of node carries wherever it is drawn, offered or edited. */
export const KIND_ICONS = { watch: Eye, route: Route, pipeline: Workflow, stage: Layers, target: FolderInput }

const STAGE_ICONS: Record<string, LucideIcon> = {
  filter: Funnel,
  number: Hash,
  regex: Regex,
  set: Equal,
  default: CircleDashed,
  replace: Replace,
  case: CaseSensitive,
  strip: Eraser,
  format: PenLine,
  folder: FolderTree,
  lift: ArrowUpToLine,
  next: ListPlus,
  rank: ListOrdered,
  take: Copy,
}

/** The icon a stage carries wherever it is shown, by the name the core declares it under; a stage without
 * its own falls back to the icon of the stage kind. */
export function stageIcon(name: string): LucideIcon {
  return STAGE_ICONS[name] ?? KIND_ICONS.stage
}
