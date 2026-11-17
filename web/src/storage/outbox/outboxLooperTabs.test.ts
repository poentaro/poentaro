import { readFileSync } from 'node:fs'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { instantiate } from '../../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import { type SendRequest, setWasmRoot } from '../../poentaroWasm'
import { createMemoryOutboxStore } from './memoryOutboxStore'
import { createOutboxLooper, type OutboxChangeChannel } from './outboxLooper'

beforeAll(async () => {
  const bytes = readFileSync(
    new URL(
      '../../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.core.wasm',
      import.meta.url,
    ),
  )
  const module = await WebAssembly.compile(bytes)
  setWasmRoot(await instantiate(() => module, {}))
})

const settle = () => new Promise((resolve) => setTimeout(resolve, 20))

function linkedChannels(): [OutboxChangeChannel, OutboxChangeChannel] {
  const listeners = [new Set<() => void>(), new Set<() => void>()]
  const channelFor = (self: number): OutboxChangeChannel => ({
    announce: () => {
      for (const listener of listeners[1 - self] ?? []) listener()
    },
    listen: (onChange) => {
      listeners[self]?.add(onChange)
      return () => listeners[self]?.delete(onChange)
    },
  })
  return [channelFor(0), channelFor(1)]
}

describe('outbox looper across tabs', () => {
  it('delivers a message queued from a follower tab and keeps both tabs messages', async () => {
    const store = createMemoryOutboxStore()
    const [leaderChannel, followerChannel] = linkedChannels()
    const sent: SendRequest[] = []
    const shared = {
      store,
      createScheduler: () => ({
        wakeAt: () => undefined,
        wakeNow: () => undefined,
        dispose: () => undefined,
      }),
      now: () => 1_000_000,
      random: () => 0,
    }
    const leader = createOutboxLooper({
      ...shared,
      changes: leaderChannel,
      sendMessage: vi.fn(async (request: SendRequest) => {
        sent.push(request)
        return { tag: 'ok' as const, val: { revision: BigInt(sent.length) } }
      }),
      runAsLeader: (task) => {
        const controller = new AbortController()
        void task(controller.signal)
        return () => controller.abort()
      },
    })
    const follower = createOutboxLooper({
      ...shared,
      changes: followerChannel,
      sendMessage: vi.fn(),
      runAsLeader: () => () => undefined,
    })
    await leader.start()
    await follower.start()
    await settle()

    await follower.enqueue('file-a', {
      tag: 'save-content',
      val: { content: 'from follower' },
    })
    await settle()
    await leader.whenIdle()

    expect(sent.map((request) => request.fileId)).toEqual(['file-a'])
    expect(leader.snapshot().summary.hasUnsynced).toBe(false)
  })
})
