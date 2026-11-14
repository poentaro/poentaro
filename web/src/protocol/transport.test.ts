import { afterEach, describe, expect, it, vi } from 'vitest'
import { NetworkFailure } from '../syncedShare/workerClient'
import { isSchemaOutdated } from './schemaOutdated'
import {
  SCHEMA_HEADER,
  SchemaOutdated,
  SessionEnded,
  sendTo,
} from './transport'

function answer(status: number, body = ''): void {
  vi.stubGlobal(
    'fetch',
    vi.fn(async () => new Response(body, { status })),
  )
}

/** The request the last test sent. */
function sent(): { url: unknown; init: RequestInit | undefined } {
  const call = vi.mocked(fetch).mock.calls[0]
  if (!call) throw new Error('fetch was not called')
  return { url: call[0], init: call[1] }
}

afterEach(() => vi.unstubAllGlobals())

describe('sendTo', () => {
  it('posts the request to the endpoint with the schema hash and the token', async () => {
    answer(200, 'RESULTS')
    const send = sendTo({
      host: 'localhost:8787',
      protocol: 'app',
      schemaHash: 'abc',
      token: 'tok',
    })
    expect(await send('files/list')).toBe('RESULTS')
    const { url, init } = sent()
    expect(url).toBe('http://localhost:8787/app')
    expect(init?.method).toBe('POST')
    expect(init?.body).toBe('files/list')
    const headers = init?.headers as Record<string, string>
    expect(headers[SCHEMA_HEADER]).toBe('abc')
    expect(headers.Authorization).toBe('Bearer tok')
  })

  it('sends no credential to the public endpoint', async () => {
    answer(200)
    await sendTo({ host: 'h.example', protocol: 'public', schemaHash: 'abc' })(
      'x',
    )
    const { url, init } = sent()
    expect(url).toBe('https://h.example/public')
    expect(
      (init?.headers as Record<string, string>).Authorization,
    ).toBeUndefined()
  })

  it('turns a 401 into an ended session', async () => {
    answer(401)
    const send = sendTo({
      host: 'h',
      protocol: 'app',
      schemaHash: 'a',
      token: 't',
    })
    await expect(send('x')).rejects.toBeInstanceOf(SessionEnded)
  })

  it('turns a 409 into a reload prompt', async () => {
    answer(409)
    const send = sendTo({ host: 'h', protocol: 'public', schemaHash: 'old' })
    await expect(send('x')).rejects.toBeInstanceOf(SchemaOutdated)
    expect(isSchemaOutdated()).toBe(true)
  })

  it('turns a rejected fetch into a network failure', async () => {
    vi.stubGlobal(
      'fetch',
      vi.fn(async () => {
        throw new TypeError('Failed to fetch')
      }),
    )
    const send = sendTo({ host: 'h', protocol: 'public', schemaHash: 'a' })
    await expect(send('x')).rejects.toBeInstanceOf(NetworkFailure)
  })
})
