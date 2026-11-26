import { describe, expect, it } from 'vitest'
import { Client } from '../generated/protocol/api'
import { signedIn } from './signedIn'

/** A signed-in client whose worker answers each call with `replies`, in
 * order, and records every request it was sent. */
function clientAnswering(replies: unknown[]) {
  const requests: unknown[][] = []
  const api = new Client(async (request) => {
    const calls = JSON.parse(request) as unknown[]
    requests.push(calls)
    return JSON.stringify(replies.splice(0, calls.length))
  })
  return { client: signedIn(api, 'token'), requests }
}

const file = (id: string) => ({
  kind: 'answered',
  value: {
    kind: 'file with content',
    summary: {
      kind: 'summary of file at revision trashed at',
      id,
      path: `${id}.poentaro`,
      revision: 1,
      trashedAt: null,
    },
    content: id,
  },
})

describe('the generated client batch', () => {
  it('sends every call made in the batch as one request', async () => {
    const { client, requests } = clientAnswering([
      file('a'),
      { kind: 'failed', problem: { kind: 'missing' } },
      file('c'),
    ])
    const replies = await client.batch((calls) =>
      ['a', 'b', 'c'].map((id) => calls.readFile(id)),
    )
    expect(requests).toHaveLength(1)
    expect(replies.map((reply) => reply.kind)).toEqual([
      'answered',
      'failed',
      'answered',
    ])
  })

  it('sends the session token as each signed-in call’s credential', async () => {
    const { client, requests } = clientAnswering([file('a')])
    await client.readFile('a')
    expect(requests[0]).toMatchObject([
      {
        arguments: {
          endpoint: { kind: 'read file', id: 'a' },
          credential: { kind: 'bearer token', token: 'token' },
        },
      },
    ])
  })

  it('sends a call outside a batch on its own', async () => {
    const { client, requests } = clientAnswering([file('a'), file('b')])
    await client.readFile('a')
    await client.readFile('b')
    expect(requests).toHaveLength(2)
  })

  it('sends nothing for an empty batch', async () => {
    const { client, requests } = clientAnswering([])
    expect(await client.batch(() => [])).toEqual([])
    expect(requests).toHaveLength(0)
  })

  it('fails every call of the batch when the request fails', async () => {
    const client = signedIn(
      new Client(() => Promise.reject(new Error('offline'))),
      'token',
    )
    await expect(
      client.batch((calls) => [calls.readFile('a'), calls.readFile('b')]),
    ).rejects.toThrow('offline')
  })
})
