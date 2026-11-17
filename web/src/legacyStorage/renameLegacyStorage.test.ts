import 'fake-indexeddb/auto'
import { openDB } from 'idb'
import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import {
  createIndexedDbOutboxStore,
  openIndexedDbOutboxStore,
} from '../storage/outbox/outboxStore'
import {
  currentLocalStorageKey,
  currentOutboxDatabaseName,
  hasLegacyLocalStorage,
} from './legacyStorageNames'
import { renameLegacyStorage } from './renameLegacyStorage'

function memoryStorage(): Storage {
  const entries = new Map<string, string>()
  return {
    get length() {
      return entries.size
    },
    key: (index) => [...entries.keys()][index] ?? null,
    getItem: (key) => entries.get(key) ?? null,
    setItem: (key, value) => void entries.set(key, value),
    removeItem: (key) => void entries.delete(key),
    clear: () => entries.clear(),
  }
}

/** Saves `records` in the legacy outbox `name`, like the old build did,
 * and closes it again. */
async function seedLegacyOutbox(
  name: string,
  records: { key: string; value: string }[],
): Promise<void> {
  const database = await openDB(name, 1, {
    upgrade(upgrading) {
      upgrading.createObjectStore('records')
    },
  })
  await Promise.all(
    records.map(({ key, value }) => database.put('records', value, key)),
  )
  database.close()
}

async function databaseNames(): Promise<string[]> {
  return (await indexedDB.databases()).flatMap((database) =>
    database.name === undefined ? [] : [database.name],
  )
}

beforeEach(() => {
  vi.stubGlobal('localStorage', memoryStorage())
})

afterEach(() => {
  vi.unstubAllGlobals()
})

describe('legacy storage names', () => {
  it('maps legacy names to current ones and leaves the rest alone', () => {
    expect(currentLocalStorageKey('jianpu:files:v1')).toBe('poentaro:files:v1')
    expect(currentLocalStorageKey('poentaro:files:v1')).toBe(
      'poentaro:files:v1',
    )
    expect(currentLocalStorageKey('other')).toBe('other')
    expect(currentOutboxDatabaseName('jianpu-outbox:ann')).toBe(
      'poentaro-outbox:ann',
    )
  })

  it('notices legacy keys', () => {
    expect(hasLegacyLocalStorage()).toBe(false)
    localStorage.setItem('jianpu:part-toggles:v1', '{}')
    expect(hasLegacyLocalStorage()).toBe(true)
  })
})

describe('renameLegacyStorage', () => {
  it('moves legacy keys, keeping what is already saved under a current name', async () => {
    localStorage.setItem('jianpu:part-toggles:v1', 'legacy toggles')
    localStorage.setItem('jianpu:storage-backend:v1', 'legacy backend')
    localStorage.setItem('poentaro:storage-backend:v1', 'current backend')

    await renameLegacyStorage()

    expect(localStorage.getItem('poentaro:part-toggles:v1')).toBe(
      'legacy toggles',
    )
    expect(localStorage.getItem('poentaro:storage-backend:v1')).toBe(
      'current backend',
    )
    expect(hasLegacyLocalStorage()).toBe(false)
  })

  it('keeps a differing legacy file store as a backup', async () => {
    localStorage.setItem('jianpu:files:v1', 'from an old tab')
    localStorage.setItem('poentaro:files:v1', 'current')

    await renameLegacyStorage()

    expect(localStorage.getItem('poentaro:files:v1')).toBe('current')
    expect(localStorage.getItem('poentaro:files:v1:legacy-backup')).toBe(
      'from an old tab',
    )
  })

  it('moves a legacy outbox and deletes it', async () => {
    const account = 'moved'
    const record = { key: 'message:1', value: 'queued edit' }
    await seedLegacyOutbox(`jianpu-outbox:${account}`, [record])

    await renameLegacyStorage()

    expect(await createIndexedDbOutboxStore(account).loadAll()).toEqual([
      record,
    ])
    expect(await databaseNames()).not.toContain(`jianpu-outbox:${account}`)
  })

  it('keeps both outboxes when the current one already queued edits', async () => {
    const account = 'both'
    const legacy = { key: 'message:1', value: 'legacy edit' }
    const current = { key: 'message:2', value: 'current edit' }
    await seedLegacyOutbox(`jianpu-outbox:${account}`, [legacy])
    await createIndexedDbOutboxStore(account).apply({
      puts: [current],
      deletes: [],
    })

    await renameLegacyStorage()

    expect(await createIndexedDbOutboxStore(account).loadAll()).toEqual([
      current,
    ])
    expect(
      await openIndexedDbOutboxStore(`jianpu-outbox:${account}`).loadAll(),
    ).toEqual([legacy])
  })
})
