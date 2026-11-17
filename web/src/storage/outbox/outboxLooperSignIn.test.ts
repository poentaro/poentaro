import { readFileSync } from 'node:fs'
import { beforeAll, describe, expect, it, vi } from 'vitest'
import { instantiate } from '../../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import {
  type SendRequest,
  type SendResult,
  setWasmRoot,
} from '../../poentaroWasm'
import { createMemoryOutboxStore } from './memoryOutboxStore'
import { createOutboxLooper } from './outboxLooper'
import type { OutboxStore, StoredRecord } from './outboxStore'

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

const scheduler = () => ({
  wakeAt: () => undefined,
  wakeNow: () => undefined,
  dispose: () => undefined,
})

/** A store whose next read, once `holdNextRead` is called, reads the
 * records at once but only answers when `release` is called -- a slow
 * IndexedDB read that other writes overtake. */
function storeWithHeldRead(inner: OutboxStore) {
  let holding = false
  let release: () => void = () => undefined
  const store: OutboxStore = {
    async loadAll(): Promise<StoredRecord[]> {
      const released = holding
        ? new Promise<void>((resolve) => {
            release = resolve
          })
        : Promise.resolve()
      holding = false
      const records = await inner.loadAll()
      await released
      return records
    },
    apply: (changes) => inner.apply(changes),
  }
  return {
    store,
    holdNextRead: () => {
      holding = true
    },
    release: () => release(),
  }
}

describe('outbox looper after signing in again', () => {
  it('resumes a lane stopped for sign-in even while it is becoming the leader', async () => {
    const memory = createMemoryOutboxStore()
    const refused = createOutboxLooper({
      store: memory,
      sendMessage: async (): Promise<SendResult> => ({ tag: 'unauthorized' }),
      createScheduler: scheduler,
      runAsLeader: (task) => {
        const controller = new AbortController()
        void task(controller.signal)
        return () => controller.abort()
      },
      now: () => 1_000_000,
      random: () => 0,
    })
    await refused.start()
    await refused.enqueue('file-a', {
      tag: 'save-content',
      val: { content: 'written while signed out' },
    })
    await refused.whenIdle()
    refused.stop()

    const held = storeWithHeldRead(memory)
    let grantLeadership: () => void = () => undefined
    const sent: SendRequest[] = []
    const signedIn = createOutboxLooper({
      store: held.store,
      sendMessage: vi.fn(async (request: SendRequest) => {
        sent.push(request)
        return { tag: 'ok' as const, val: { revision: 1n } }
      }),
      createScheduler: scheduler,
      runAsLeader: (task) => {
        const controller = new AbortController()
        grantLeadership = () => void task(controller.signal)
        return () => controller.abort()
      },
      now: () => 1_000_000,
      random: () => 0,
    })
    await signedIn.start()
    // The lock arrives and the new leader starts re-reading the store...
    held.holdNextRead()
    grantLeadership()
    // ...while the lanes stopped for sign-in are released.
    const resolving = signedIn.resolveSignedIn()
    await settle()
    held.release()
    await resolving
    await settle()
    await signedIn.whenIdle()

    expect(sent.map((request) => request.fileId)).toEqual(['file-a'])
    expect(signedIn.snapshot().summary.hasUnsynced).toBe(false)
  })
})
