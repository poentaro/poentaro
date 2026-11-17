import type {
  Attempt,
  HistoryRow,
  HistoryStatus,
  MessageKind,
} from '../../poentaroWasm'
import { outcomeLabel } from '../syncPanel/laneViewMapping'

export const STATUS_LABEL: Record<HistoryStatus, string> = {
  pending: 'Pending',
  retrying: 'Retrying',
  'needs-merge': 'Needs merge',
  'needs-sign-in': 'Needs sign-in',
  failed: "Couldn't sync",
  sent: 'Sent',
}

export const KIND_LABEL: Record<MessageKind, string> = {
  create: 'Create',
  edit: 'Edit',
  rename: 'Rename',
  trash: 'Trash',
  restore: 'Restore',
}

export function sizeLabel(bytes: number): string {
  if (bytes === 0) return '-'
  return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`
}

export function timeLabel(ms: bigint): string {
  const date = new Date(Number(ms))
  const sameDay = date.toDateString() === new Date().toDateString()
  return sameDay
    ? date.toLocaleTimeString()
    : `${date.toLocaleDateString()} ${date.toLocaleTimeString()}`
}

export function attemptLine(attempt: Attempt): string {
  return `${timeLabel(attempt.atMs)}: ${outcomeLabel(attempt.outcome)}`
}

/** Status and attempts of one message as plain text. Leaves the payload out
 * because it holds the person's music. */
export function buildRowReport(row: HistoryRow): string {
  return [
    `File: ${row.fileName} (${row.fileId})`,
    `Kind: ${KIND_LABEL[row.kind]}`,
    `Status: ${STATUS_LABEL[row.status]}`,
    `Size: ${sizeLabel(row.sizeBytes)}`,
    `Time: ${timeLabel(row.atMs)}`,
    ...(row.revision === undefined ? [] : [`Revision: ${row.revision}`]),
    'Attempts:',
    ...(row.attempts.length === 0
      ? ['  (none yet)']
      : row.attempts.map((attempt) => `  - ${attemptLine(attempt)}`)),
  ].join('\n')
}
