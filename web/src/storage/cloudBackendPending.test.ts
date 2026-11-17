import { describe, expect, it } from 'vitest'
import {
  applyPendingStructure,
  pendingContentByFileId,
  pendingNameByFileId,
} from './cloudBackendOutbox'
import type { OutboxSnapshot } from './outbox/outboxLooper'

function snapshotWith(
  lanes: Array<{ fileId: string; saves: string[] }>,
): OutboxSnapshot {
  return {
    halted: null,
    summary: {
      lanes: [],
      totalPending: 0,
      lanesNeedingAttention: 0,
      hasUnsynced: false,
    },
    queue: {
      revisions: [],
      lanes: lanes.map(({ fileId, saves }) => ({
        fileId,
        messages: saves.map((content, index) => ({
          message: { tag: 'save-content', val: { content } },
          messageId: `m${index}`,
        })),
        status: { tag: 'idle' },
      })),
    },
  } as unknown as OutboxSnapshot
}

describe('pendingContentByFileId', () => {
  it('returns the newest unsent save of each file', () => {
    const pending = pendingContentByFileId(
      snapshotWith([
        { fileId: 'a', saves: ['one', 'two'] },
        { fileId: 'b', saves: [] },
      ]),
    )
    expect([...pending]).toEqual([['a', 'two']])
  })
})

describe('applyPendingStructure', () => {
  const server = [
    { id: 'a', name: 'a.poentaro', content: 'A', revision: 1, trashedAt: null },
    { id: 'b', name: 'b.poentaro', content: 'B', revision: 1, trashedAt: null },
  ]
  const snapshotOf = (
    lanes: Array<{ fileId: string; messages: unknown[]; at?: bigint }>,
  ): OutboxSnapshot =>
    ({
      queue: {
        revisions: [],
        lanes: lanes.map(({ fileId, messages, at }) => ({
          fileId,
          status: { tag: 'idle' },
          messages: messages.map((message, index) => ({
            message,
            createdAtMs: at ?? 0n,
            messageId: `m${index}`,
          })),
        })),
      },
    }) as unknown as OutboxSnapshot

  it('keeps a file created but not yet sent', () => {
    const result = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'c',
          messages: [
            { tag: 'create-file', val: { name: 'c.poentaro', content: 'C' } },
          ],
        },
      ]),
    )
    expect(result.map((file) => file.name)).toEqual([
      'a.poentaro',
      'b.poentaro',
      'c.poentaro',
    ])
  })

  it('applies an unsent rename, trash and restore in order', () => {
    const result = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'a',
          messages: [{ tag: 'rename-file', val: { to: 'z.poentaro' } }],
        },
        {
          fileId: 'b',
          messages: [{ tag: 'trash-file' }, { tag: 'restore-file' }],
        },
      ]),
    )
    expect(result.find((file) => file.id === 'a')?.name).toBe('z.poentaro')
    expect(result.find((file) => file.id === 'b')?.trashedAt).toBeNull()
  })

  it('applies a queued batch rename to every file in it', () => {
    const result = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'batch:m1',
          messages: [
            {
              tag: 'rename-files',
              val: {
                moves: [
                  { fileId: 'a', to: 'New/a.poentaro' },
                  { fileId: 'b', to: 'New/b.poentaro' },
                ],
              },
            },
          ],
        },
      ]),
    )
    expect(result.map((file) => file.name)).toEqual([
      'New/a.poentaro',
      'New/b.poentaro',
    ])
  })

  it('replays a batch and a per-file rename in the order they were made', () => {
    const batch = {
      tag: 'rename-files',
      val: { moves: [{ fileId: 'a', to: 'New/a.poentaro' }] },
    }
    const olderBatch = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'a',
          messages: [{ tag: 'rename-file', val: { to: 'z' } }],
          at: 20n,
        },
        { fileId: 'batch:m1', messages: [batch], at: 10n },
      ]),
    )
    const newerBatch = applyPendingStructure(
      server,
      snapshotOf([
        {
          fileId: 'a',
          messages: [{ tag: 'rename-file', val: { to: 'z' } }],
          at: 10n,
        },
        { fileId: 'batch:m1', messages: [batch], at: 20n },
      ]),
    )
    expect(olderBatch.find((file) => file.id === 'a')?.name).toBe('z')
    expect(newerBatch.find((file) => file.id === 'a')?.name).toBe(
      'New/a.poentaro',
    )
  })
})

describe('pendingNameByFileId', () => {
  it('is the name a file gets once its newest queued create, rename or batch lands', () => {
    const snapshot = {
      queue: {
        revisions: [],
        lanes: [
          {
            fileId: 'a',
            status: { tag: 'idle' },
            messages: [
              {
                message: {
                  tag: 'create-file',
                  val: { name: 'A/a', content: '' },
                },
                createdAtMs: 1n,
              },
            ],
          },
          {
            fileId: 'batch:m1',
            status: { tag: 'idle' },
            messages: [
              {
                message: {
                  tag: 'rename-files',
                  val: { moves: [{ fileId: 'a', to: 'B/a' }] },
                },
                createdAtMs: 2n,
              },
            ],
          },
        ],
      },
    } as unknown as OutboxSnapshot

    expect([...pendingNameByFileId(snapshot)]).toEqual([['a', 'B/a']])
  })
})
