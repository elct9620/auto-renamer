import { Select, SelectContent, SelectItem, SelectTrigger, SelectValue } from '@/components/ui/select'
import { cn } from '@/lib/utils'

// A choice cannot be the empty value, which is how the select shows its placeholder.
const NONE = '\u0000none'

/** One value out of a few: an empty value shows the placeholder, and a clearable choice can go back to it. */
export function Choice({ label, value, options, placeholder, clearable, className, onChange }: {
  label?: string
  value: string
  options: { value: string; label: string }[]
  placeholder?: string
  clearable?: boolean
  className?: string
  onChange: (value: string) => void
}) {
  return (
    <Select value={value} onValueChange={(next) => onChange(next === NONE ? '' : next)}>
      <SelectTrigger size="sm" aria-label={label} className={cn('w-full', className)}>
        <SelectValue placeholder={placeholder} />
      </SelectTrigger>
      <SelectContent position="popper">
        {clearable && <SelectItem value={NONE}>—</SelectItem>}
        {options.map((option) => <SelectItem key={option.value} value={option.value}>{option.label}</SelectItem>)}
      </SelectContent>
    </Select>
  )
}
