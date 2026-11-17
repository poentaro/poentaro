import { describe, expect, it } from 'vitest'
import type { LaneReason, LaneSummary, QueuedMessage } from '../../poentaroWasm'
import { toLaneView } from './laneViewMapping'

const summaryFor = (
  reason: LaneReason,
  overrides: Partial<LaneSummary> = {},
): LaneSummary => ({
  fileId: 'file-a',
  reason,
  pendingMessages: 1,
  attempts: [],
  ...overrides,
})

const saveMessage = (content: string): QueuedMessage => ({
  messageId: 'm1',
  createdAtMs: 1_000n,
  message: { tag: 'save-content', val: { content } },
  attempts: [],
})

describe('toLaneView', () => {
  const cases: Array<{ reason: LaneReason; status: string }> = [
    { reason: 'syncing', status: 'syncing' },
    { reason: 'waiting-to-retry', status: 'waiting' },
    { reason: 'possibly-stuck', status: 'possibly-stuck' },
    { reason: 'needs-sign-in', status: 'needs-sign-in' },
    { reason: 'needs-merge', status: 'needs-merge' },
    { reason: 'failed', status: 'failed' },
  ]

  it.each(cases)('maps $reason to $status with a sentence', (testCase) => {
    const view = toLaneView(summaryFor(testCase.reason), [], 'a.poentaro')
    expect(view.status).toBe(testCase.status)
    expect(view.sentence.length).toBeGreaterThan(0)
    expect(view.fileName).toBe('a.poentaro')
  })

  it('rounds the retry delay up to whole seconds', () => {
    const view = toLaneView(
      summaryFor('waiting-to-retry', { retryInMs: 1_200n }),
      [],
      'a.poentaro',
    )
    expect(view.retryInSeconds).toBe(2)
  })

  it('leaves the retry delay out when none is scheduled', () => {
    expect(
      toLaneView(summaryFor('failed'), [], 'a.poentaro').retryInSeconds,
    ).toBeUndefined()
  })

  it('describes queued messages by kind and size', () => {
    const view = toLaneView(summaryFor('failed'), [saveMessage('abc')], 'a')
    expect(view.messages[0]).toMatchObject({
      kindLabel: 'SaveContent',
      sizeLabel: '3 B',
    })
  })

  it('lists attempts with their outcome', () => {
    const message: QueuedMessage = {
      ...saveMessage('x'),
      attempts: [
        {
          atMs: 2_000n,
          outcome: { tag: 'permanent', val: { status: 400, reason: 'bad' } },
        },
      ],
    }
    const view = toLaneView(summaryFor('failed'), [message], 'a')
    expect(view.attempts[0]?.outcomeLabel).toBe('400 bad')
  })
})
