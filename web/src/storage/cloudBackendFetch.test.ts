import { describe, expect, it } from 'vitest'
import {
  createTestBackend,
  fetchMock,
  jsonResponse,
} from './cloudBackendTestHelpers'

describe('createCloudBackend: fetchServerFile()', () => {
  const files = [
    {
      id: 'id-a',
      name: 'a.poentaro',
      content: '1 2',
      revision: 4,
      trashedAt: null,
    },
  ]

  it('returns the revision and content of the listed file', async () => {
    fetchMock.mockResolvedValue(jsonResponse(200, { files }))
    const { backend } = createTestBackend()
    expect(await backend.fetchServerFile('id-a')).toEqual({
      revision: 4,
      content: '1 2',
    })
  })

  it('returns undefined for an unlisted file', async () => {
    fetchMock.mockResolvedValue(jsonResponse(200, { files }))
    const { backend } = createTestBackend()
    expect(await backend.fetchServerFile('nope')).toBeUndefined()
  })
})
