import { beforeAll, describe, expect, it } from 'vitest'
import { Client } from '../generated/protocol/app'
import { poentaroWasm } from '../poentaroWasm'
import { installRealWasm } from '../storage/cloudBackendTestHelpers'
import { codecFor } from './transport'

beforeAll(installRealWasm)

/** A client whose worker answers each call with `outcomes`, in order, and
 * records every request it was sent. */
function clientAnswering(outcomes: unknown[]) {
  const requests: string[] = []
  const client = new Client(codecFor('app'), async (request) => {
    requests.push(request)
    const calls = JSON.parse(
      poentaroWasm().protocolDecodeRequest('app', request),
    ) as unknown[]
    return poentaroWasm().protocolEncodeResponse(
      'app',
      request,
      JSON.stringify(outcomes.splice(0, calls.length)),
    )
  })
  return { client, requests }
}

const file = (id: string) => ({
  kind: 'value',
  value: { summary: { id, path: `${id}.poentaro`, revision: 1 }, content: id },
})

describe('the generated client batch', () => {
  it('sends every call made in the batch as one request', async () => {
    const { client, requests } = clientAnswering([
      file('a'),
      { kind: 'error', error: { kind: 'missing' } },
      file('c'),
    ])
    const outcomes = await client.batch((calls) =>
      ['a', 'b', 'c'].map((id) => calls.files.read(id)),
    )
    expect(requests).toHaveLength(1)
    expect(outcomes.map((outcome) => outcome.kind)).toEqual([
      'value',
      'error',
      'value',
    ])
  })

  it('sends a call outside a batch on its own', async () => {
    const { client, requests } = clientAnswering([file('a'), file('b')])
    await client.files.read('a')
    await client.files.read('b')
    expect(requests).toHaveLength(2)
  })

  it('sends nothing for an empty batch', async () => {
    const { client, requests } = clientAnswering([])
    expect(await client.batch(() => [])).toEqual([])
    expect(requests).toHaveLength(0)
  })

  it('fails every call of the batch when the request fails', async () => {
    const client = new Client(codecFor('app'), () =>
      Promise.reject(new Error('offline')),
    )
    await expect(
      client.batch((calls) => [calls.files.read('a'), calls.files.read('b')]),
    ).rejects.toThrow('offline')
  })
})
