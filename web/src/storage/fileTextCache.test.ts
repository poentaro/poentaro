import { describe, expect, it } from 'vitest'
import { createFileTextCache } from './fileTextCache'
import { createMemoryOutboxStore } from './outbox/memoryOutboxStore'

describe('file text cache', () => {
  it('hits only on the exact revision', async () => {
    const cache = createFileTextCache(createMemoryOutboxStore())
    await cache.write('file-a', 3, 'three')
    expect(await cache.read('file-a', 3)).toBe('three')
    expect(await cache.read('file-a', 2)).toBeUndefined()
    expect(await cache.read('file-a', 4)).toBeUndefined()
  })

  it('misses for an unknown file', async () => {
    const cache = createFileTextCache(createMemoryOutboxStore())
    expect(await cache.read('missing', 1)).toBeUndefined()
  })

  it('overwrites the text on a later write', async () => {
    const cache = createFileTextCache(createMemoryOutboxStore())
    await cache.write('file-a', 1, 'one')
    await cache.write('file-a', 2, 'two')
    expect(await cache.read('file-a', 1)).toBeUndefined()
    expect(await cache.read('file-a', 2)).toBe('two')
  })

  it('drops only the named file', async () => {
    const cache = createFileTextCache(createMemoryOutboxStore())
    await cache.write('file-a', 1, 'a')
    await cache.write('file-b', 1, 'b')
    await cache.drop('file-a')
    expect(await cache.read('file-a', 1)).toBeUndefined()
    expect(await cache.read('file-b', 1)).toBe('b')
  })

  it('stores an empty text as a real value', async () => {
    const cache = createFileTextCache(createMemoryOutboxStore())
    await cache.write('file-a', 1, '')
    expect(await cache.read('file-a', 1)).toBe('')
  })
})
