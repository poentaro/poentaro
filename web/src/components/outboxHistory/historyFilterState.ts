import type { HistoryFilter, MessageKind } from '../../poentaroWasm'

/** The filter controls as the person left them; empty string means "any". */
export interface HistoryFilterState {
  query: string
  status: string
  kind: string
  fileId: string
  age: string
  size: string
  tries: string
}

export const EMPTY_FILTER_STATE: HistoryFilterState = {
  query: '',
  status: '',
  kind: '',
  fileId: '',
  age: '',
  size: '',
  tries: '',
}

export const STATUS_OPTIONS = [
  { value: 'pending', label: 'Pending' },
  { value: 'retrying', label: 'Retrying' },
  { value: 'needs-merge', label: 'Needs merge' },
  { value: 'needs-sign-in', label: 'Needs sign-in' },
  { value: 'failed', label: "Couldn't sync" },
  { value: 'sent', label: 'Sent' },
  { value: 'problem', label: 'Any problem' },
] as const

export const KIND_OPTIONS: ReadonlyArray<{
  value: MessageKind
  label: string
}> = [
  { value: 'create', label: 'Create' },
  { value: 'edit', label: 'Edit' },
  { value: 'rename', label: 'Rename' },
  { value: 'trash', label: 'Trash' },
  { value: 'restore', label: 'Restore' },
]

const HOUR_MS = 3_600_000n
const AGE_OPTIONS = [
  { value: '1h', label: 'Last hour', ms: HOUR_MS },
  { value: '24h', label: 'Last 24 hours', ms: 24n * HOUR_MS },
  { value: '7d', label: 'Last 7 days', ms: 168n * HOUR_MS },
] as const

const SIZE_OPTIONS = [
  { value: '1kb', label: '1 KB or more', bytes: 1024 },
  { value: '3kb', label: '3 KB or more', bytes: 3 * 1024 },
] as const

const TRIES_OPTIONS = [
  { value: 'untried', label: 'None yet' },
  { value: 'two-or-more', label: '2 or more' },
] as const

export { AGE_OPTIONS, SIZE_OPTIONS, TRIES_OPTIONS }

export function isFilterActive(state: HistoryFilterState): boolean {
  return Object.values(state).some((value) => value !== '')
}

/** Turns the control values into the filter the outbox understands; the
 * outbox decides what matches. */
export function toHistoryFilter(state: HistoryFilterState): HistoryFilter {
  return {
    query: state.query,
    status: statusOf(state.status),
    kind: KIND_OPTIONS.find((option) => option.value === state.kind)?.value,
    fileId: state.fileId === '' ? undefined : state.fileId,
    maxAgeMs: AGE_OPTIONS.find((option) => option.value === state.age)?.ms,
    minSizeBytes: SIZE_OPTIONS.find((option) => option.value === state.size)
      ?.bytes,
    attempts:
      TRIES_OPTIONS.find((option) => option.value === state.tries)?.value ??
      'any',
  }
}

function statusOf(value: string): HistoryFilter['status'] {
  if (value === 'problem') return { tag: 'problem' }
  const only = STATUS_OPTIONS.find(
    (option) => option.value === value && option.value !== 'problem',
  )
  return only
    ? { tag: 'only', val: only.value as Exclude<typeof only.value, 'problem'> }
    : { tag: 'any' }
}
