import { describe, expect, it } from 'vitest'
import type { LaneReason } from '../poentaroWasm'
import type { OutboxSnapshot } from '../storage/outbox/outboxLooper'
import { hasUnsyncedWork, outboxSaveBadge } from './outboxSaveBadge'

function snapshotOf(
  reasons: LaneReason[],
  options: { halted?: boolean } = {},
): OutboxSnapshot {
  const lanes = reasons.map((reason, index) => ({
    fileId: `file-${index}`,
    reason,
    pendingMessages: 1,
    attempts: [],
  }))
  return {
    queue: { lanes: [] },
    summary: {
      lanes,
      totalPending: lanes.length,
      lanesNeedingAttention: lanes.filter(
        (lane) => !['syncing', 'waiting-to-retry'].includes(lane.reason),
      ).length,
      hasUnsynced: lanes.length > 0,
    },
    halted: options.halted ? { tag: 'corrupt' } : null,
  } as unknown as OutboxSnapshot
}

describe('outboxSaveBadge', () => {
  it('shows Saved for an empty queue', () => {
    expect(outboxSaveBadge(snapshotOf([]))).toEqual({
      status: 'saved',
      label: 'Saved',
    })
  })

  it('shows Saving while a lane is syncing', () => {
    expect(outboxSaveBadge(snapshotOf(['syncing']))).toEqual({
      status: 'saving',
      label: 'Saving…',
    })
  })

  it('shows changes waiting for lanes retrying later', () => {
    expect(outboxSaveBadge(snapshotOf(['waiting-to-retry']))).toEqual({
      status: 'waiting',
      label: '1 change waiting',
    })
    expect(
      outboxSaveBadge(snapshotOf(['waiting-to-retry', 'syncing'])).label,
    ).toBe('2 changes waiting')
  })

  it('shows files needing attention, singular and plural', () => {
    expect(outboxSaveBadge(snapshotOf(['needs-merge'])).label).toBe(
      '1 file needs attention',
    )
    expect(
      outboxSaveBadge(snapshotOf(['failed', 'needs-sign-in', 'syncing'])),
    ).toEqual({ status: 'attention', label: '2 files need attention' })
  })

  it('treats a halted queue as needing attention', () => {
    expect(outboxSaveBadge(snapshotOf([], { halted: true })).label).toBe(
      '1 file needs attention',
    )
  })
})

describe('hasUnsyncedWork', () => {
  it('is false for an empty queue', () => {
    expect(hasUnsyncedWork(snapshotOf([]))).toBe(false)
  })

  it('is true when any lane has messages', () => {
    expect(hasUnsyncedWork(snapshotOf(['failed']))).toBe(true)
  })

  it('is true when the queue is halted', () => {
    expect(hasUnsyncedWork(snapshotOf([], { halted: true }))).toBe(true)
  })
})
