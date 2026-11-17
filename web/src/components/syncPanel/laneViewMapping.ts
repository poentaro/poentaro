import type {
  Attempt,
  LaneReason,
  LaneSummary,
  Message,
  QueuedMessage,
} from '../../poentaroWasm'
import type { OutboxSnapshot } from '../../storage/outbox/outboxLooper'
import type {
  AttemptView,
  LaneView,
  LaneViewStatus,
  MessageView,
} from './syncPanelTypes'

/** The one plain sentence shown for each reason a lane is in the panel. */
const SENTENCE_BY_REASON: Record<LaneReason, string> = {
  syncing: 'Sending your changes now.',
  'waiting-to-retry':
    'Your changes are saved on this device and will be sent when the connection allows.',
  'possibly-stuck':
    'Your changes have been waiting for a while. They are safe on this device.',
  'needs-sign-in': 'Sign in again to keep syncing this file.',
  'needs-merge':
    'This file changed on another device. Review the differences to continue.',
  failed:
    "The server refused these changes, so they can't be synced as they are.",
}

const STATUS_BY_REASON: Record<LaneReason, LaneViewStatus> = {
  syncing: 'syncing',
  'waiting-to-retry': 'waiting',
  'possibly-stuck': 'possibly-stuck',
  'needs-sign-in': 'needs-sign-in',
  'needs-merge': 'needs-merge',
  failed: 'failed',
}

const KIND_LABEL: Record<Message['tag'], string> = {
  'create-file': 'CreateFile',
  'save-content': 'SaveContent',
  'rename-file': 'RenameFile',
  'rename-files': 'RenameFiles',
  'trash-file': 'TrashFile',
  'restore-file': 'RestoreFile',
}

function timeLabel(ms: bigint): string {
  return new Date(Number(ms)).toLocaleTimeString()
}

function sizeLabel(message: Message): string {
  if (message.tag !== 'save-content' && message.tag !== 'create-file') {
    return '-'
  }
  const bytes = new TextEncoder().encode(message.val.content).length
  return bytes < 1024 ? `${bytes} B` : `${(bytes / 1024).toFixed(1)} KB`
}

export function outcomeLabel(outcome: Attempt['outcome']): string {
  switch (outcome.tag) {
    case 'succeeded':
      return 'Sent'
    case 'transient':
      return `Network problem: ${outcome.val.reason}`
    case 'unauthorized':
      return 'Signed out (401)'
    case 'revision-conflict':
      return `Changed elsewhere (server revision ${outcome.val.currentRevision})`
    case 'name-taken':
      return `Name taken, tried ${outcome.val.suggestedName}`
    case 'names-taken':
      return `${outcome.val.adjustments.length} names taken, adjusted`
    case 'permanent':
      return `${outcome.val.status} ${outcome.val.reason}`
  }
}

function attemptViews(messages: QueuedMessage[]): AttemptView[] {
  return messages.flatMap(({ attempts }) =>
    attempts.map((attempt) => ({
      atLabel: timeLabel(attempt.atMs),
      outcomeLabel: outcomeLabel(attempt.outcome),
    })),
  )
}

function messageViews(messages: QueuedMessage[]): MessageView[] {
  return messages.map(({ message, createdAtMs }) => ({
    kindLabel: KIND_LABEL[message.tag],
    sizeLabel: sizeLabel(message),
    createdLabel: timeLabel(createdAtMs),
  }))
}

export function toLaneView(
  lane: LaneSummary,
  messages: QueuedMessage[],
  fileName: string,
): LaneView {
  return {
    fileId: lane.fileId,
    fileName,
    status: STATUS_BY_REASON[lane.reason],
    sentence: SENTENCE_BY_REASON[lane.reason],
    retryInSeconds:
      lane.retryInMs === undefined
        ? undefined
        : Math.ceil(Number(lane.retryInMs) / 1000),
    messages: messageViews(messages),
    attempts: attemptViews(messages),
  }
}

/** Maps the outbox state to what the panel renders. `fileNameOf` is the only
 * thing the outbox cannot know. */
export function toLaneViews(
  snapshot: OutboxSnapshot,
  fileNameOf: (fileId: string) => string,
): LaneView[] {
  return snapshot.summary.lanes.map((lane) =>
    toLaneView(
      lane,
      snapshot.queue.lanes.find((queued) => queued.fileId === lane.fileId)
        ?.messages ?? [],
      fileNameOf(lane.fileId),
    ),
  )
}
