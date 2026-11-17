/** Whether the worker answered that this tab's schema is outdated, for the
 * reload prompt. Once set it stays set: only a reload clears it. */
let outdated = false
const listeners = new Set<() => void>()

export function markSchemaOutdated(): void {
  if (outdated) return
  outdated = true
  for (const listener of listeners) listener()
}

export function isSchemaOutdated(): boolean {
  return outdated
}

export function subscribeSchemaOutdated(listener: () => void): () => void {
  listeners.add(listener)
  return () => listeners.delete(listener)
}
