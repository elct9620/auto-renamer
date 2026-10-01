export { cn } from "cn"

export function errorOf(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}
