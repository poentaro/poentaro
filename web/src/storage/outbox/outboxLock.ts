const LOCK_NAME = 'poentaro-outbox-looper'

/**
 * Runs `task` only in the one tab that holds the outbox looper lock. Other tabs wait
 * for the lock; the browser hands it over when the holding tab closes. Returns a stop
 * function that aborts the task's signal and releases (or cancels waiting for) the lock.
 * Without `navigator.locks`, `task` simply runs (single-tab fallback).
 */
export function runAsLeader(
  task: (signal: AbortSignal) => Promise<void>,
): () => void {
  const controller = new AbortController()
  const { signal } = controller
  const stop = () => controller.abort()

  if (typeof navigator === 'undefined' || !navigator.locks) {
    void task(signal).catch(() => undefined)
    return stop
  }

  const aborted = new Promise<void>((resolve) => {
    signal.addEventListener('abort', () => resolve(), { once: true })
  })
  navigator.locks
    .request(LOCK_NAME, { signal }, async () => {
      await Promise.race([task(signal), aborted])
    })
    .catch(() => undefined)
  return stop
}
