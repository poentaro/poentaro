import { readFileSync } from 'node:fs'
import { beforeAll, describe, expect, it } from 'vitest'
import { instantiate } from '../../../../crates/poentaro-wasm/pkg-component/poentaro_wasm.js'
import {
  type SendRequest,
  type SendResult,
  setWasmRoot,
} from '../../poentaroWasm'
import { createBaseSnapshots } from './baseSnapshots'
import {
  createConflictFlow,
  MERGED_NOTICE,
  type ServerFile,
} from './conflictFlow'
import { createMemoryOutboxStore } from './memoryOutboxStore'
import { createOutboxLooper } from './outboxLooper'

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

const BASE = 'a\nb\nc\nd\ne\n'
const score = (first: number, second: number, third: number) =>
  `score """\n[M] ${first}\n\n[M] ${second}\n\n[M] ${third}\n"""\n`
const settle = () => new Promise((resolve) => setTimeout(resolve, 20))

function makeHarness(options: {
  base?: string
  server: ServerFile
  send?: (request: SendRequest) => SendResult
}) {
  const store = createMemoryOutboxStore()
  const bases = createBaseSnapshots(store)
  const sent: SendRequest[] = []
  const looper = createOutboxLooper({
    store,
    sendMessage: async (request) => {
      sent.push(request)
      if (
        request.fileId === 'file-a' &&
        sent.filter((r) => r.fileId === 'file-a').length === 1
      ) {
        return {
          tag: 'conflict',
          val: { currentRevision: BigInt(options.server.revision) },
        }
      }
      return options.send?.(request) ?? { tag: 'ok', val: { revision: 9n } }
    },
    createScheduler: () => ({
      wakeAt: () => undefined,
      wakeNow: () => undefined,
      dispose: () => undefined,
    }),
    runAsLeader: (task) => {
      void task(new AbortController().signal)
      return () => undefined
    },
    now: () => 1_000_000,
    random: () => 0,
  })
  const notices: string[] = []
  const flow = createConflictFlow({
    looper,
    bases,
    fetchServerFile: async () => options.server,
    onNotice: (message) => notices.push(message),
  })
  const ready = (async () => {
    if (options.base !== undefined)
      await bases.recordBase('file-a', options.base)
    await looper.start()
  })()
  return { looper, flow, sent, notices, ready }
}

const save = (content: string) => ({
  tag: 'save-content' as const,
  val: { content },
})

describe('conflict flow', () => {
  it('saves a clean merge and raises a notice', async () => {
    const harness = makeHarness({
      base: score(1, 2, 3),
      server: { revision: 5, content: score(1, 2, 6) },
    })
    await harness.ready
    await harness.looper.enqueue('file-a', save(score(5, 2, 3)))
    await settle()
    await harness.looper.whenIdle()
    await settle()

    const merged = harness.sent.at(-1)
    expect(merged?.message).toEqual(save(score(5, 2, 6)))
    expect(merged?.expectedRevision).toBe(5n)
    expect(harness.notices).toEqual([MERGED_NOTICE])
    expect(harness.flow.conflicts()).toEqual([])
  })

  it('keeps a conflicting merge for the user, with markers', async () => {
    const harness = makeHarness({
      base: BASE,
      server: { revision: 5, content: 'a\nb\nc\nd\nTHEIRS\n' },
    })
    await harness.ready
    await harness.looper.enqueue('file-a', save('a\nb\nc\nd\nMINE\n'))
    await settle()

    const [conflict] = harness.flow.conflicts()
    expect(conflict?.mine).toBe('a\nb\nc\nd\nMINE\n')
    expect(conflict?.theirs).toBe('a\nb\nc\nd\nTHEIRS\n')
    expect(conflict?.mergedWithMarkers).toContain('<<<<<<<')
    expect(harness.sent).toHaveLength(1)
    expect(harness.notices).toEqual([])
  })

  it('treats a missing base as a conflict between mine and theirs', async () => {
    const harness = makeHarness({
      server: { revision: 5, content: 'theirs\n' },
    })
    await harness.ready
    await harness.looper.enqueue('file-a', save('mine\n'))
    await settle()

    const [conflict] = harness.flow.conflicts()
    expect(conflict?.mergedWithMarkers).toContain('mine')
    expect(conflict?.mergedWithMarkers).toContain('theirs')
    expect(harness.sent).toHaveLength(1)
  })

  it('lets other files keep syncing during a conflict', async () => {
    const harness = makeHarness({
      base: BASE,
      server: { revision: 5, content: 'a\nb\nc\nd\nTHEIRS\n' },
    })
    await harness.ready
    await harness.looper.enqueue('file-a', save('a\nb\nc\nd\nMINE\n'))
    await harness.looper.enqueue('file-b', save('other\n'))
    await settle()
    await harness.looper.whenIdle()

    expect(harness.sent.map((r) => r.fileId)).toContain('file-b')
    expect(harness.flow.conflicts()).toHaveLength(1)
  })

  it('forgets the conflict once the lane is discarded', async () => {
    const harness = makeHarness({
      base: BASE,
      server: { revision: 5, content: 'a\nb\nc\nd\nTHEIRS\n' },
    })
    await harness.ready
    await harness.looper.enqueue('file-a', save('a\nb\nc\nd\nMINE\n'))
    await settle()
    await harness.looper.resolve('file-a', { tag: 'discard' })
    await settle()

    expect(harness.flow.conflicts()).toEqual([])
  })
})
