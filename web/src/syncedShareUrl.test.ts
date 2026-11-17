import { afterEach, beforeEach, describe, expect, it, vi } from 'vitest'
import { buildSyncedShareUrl, parseSyncedShare } from './syncedShareUrl'

// A well-formed share id in the worker's generated `shareIdFormat.json`
// shape (11 base64url chars) -- shareIds are generated server-side now (task
// 11), so tests use a fixed sample instead of deriving one.
const SAMPLE_SHARE_ID = 'abcDEF123_-'

describe('parseSyncedShare (?s= query)', () => {
  it('parses a bare share id', () => {
    expect(parseSyncedShare(`?s=${SAMPLE_SHARE_ID}`, '')).toEqual({
      shareId: SAMPLE_SHARE_ID,
    })
  })

  it('returns null when there is neither ?s= nor a legacy hash', () => {
    expect(parseSyncedShare('', '')).toBeNull()
    expect(parseSyncedShare('?file=a.poentaro', '#share=abc123')).toBeNull()
  })

  it('rejects a malformed share id', () => {
    expect(parseSyncedShare('?s=not@valid@id', '')).toBeNull()
    expect(parseSyncedShare('?s=', '')).toBeNull()
    expect(
      parseSyncedShare(`?s=${SAMPLE_SHARE_ID.slice(0, -1)}`, ''),
    ).toBeNull()
  })

  it('parses a --filename suffix after the fixed-length share id, appending .poentaro', () => {
    expect(parseSyncedShare(`?s=${SAMPLE_SHARE_ID}--My-Song`, '')).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: 'My-Song.poentaro',
    })
  })

  it('decodes percent-encoded CJK filenames and keeps further dashes', () => {
    const search = `?s=${SAMPLE_SHARE_ID}--${encodeURIComponent('彌勒淨土鄉--2')}`
    expect(parseSyncedShare(search, '')).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: '彌勒淨土鄉--2.poentaro',
    })
  })

  it('rejects trailing content that is not a --filename suffix', () => {
    expect(parseSyncedShare(`?s=${SAMPLE_SHARE_ID}foo`, '')).toBeNull()
  })

  it('still rejects a malformed share id even with a --filename suffix', () => {
    expect(parseSyncedShare('?s=not@valid@id--foo', '')).toBeNull()
  })

  it('works alongside other query params', () => {
    expect(
      parseSyncedShare(`?file=a.poentaro&s=${SAMPLE_SHARE_ID}`, ''),
    ).toEqual({ shareId: SAMPLE_SHARE_ID })
  })
})

describe('parseSyncedShare (legacy #synced= hash)', () => {
  it('still parses a legacy hash link', () => {
    expect(parseSyncedShare('', `#synced=${SAMPLE_SHARE_ID}`)).toEqual({
      shareId: SAMPLE_SHARE_ID,
    })
  })

  it('leaves a legacy filename suffix unescaped', () => {
    const hash = `#synced=${SAMPLE_SHARE_ID}--快樂天堂 100% A & B -- more`
    expect(parseSyncedShare('', hash)).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: '快樂天堂 100% A & B -- more.poentaro',
    })
  })

  it('rejects a malformed legacy share id', () => {
    expect(parseSyncedShare('', '#synced=not@valid@id')).toBeNull()
  })

  it('prefers ?s= over a legacy hash when both are present', () => {
    expect(
      parseSyncedShare(`?s=${SAMPLE_SHARE_ID}`, '#synced=zzzzzzzzzzz'),
    ).toEqual({ shareId: SAMPLE_SHARE_ID })
  })
})

describe('buildSyncedShareUrl', () => {
  beforeEach(() => {
    vi.stubGlobal('window', { location: { origin: 'https://example.test' } })
  })
  afterEach(() => {
    vi.unstubAllGlobals()
  })

  const buildAndParse = (filename?: string) => {
    const url = new URL(buildSyncedShareUrl(SAMPLE_SHARE_ID, filename))
    return { url, parsed: parseSyncedShare(url.search, url.hash) }
  }

  it('puts the share id in ?s= and emits no hash', () => {
    const { url, parsed } = buildAndParse()
    expect(url.hash).toBe('')
    expect(url.searchParams.get('s')).toBe(SAMPLE_SHARE_ID)
    expect(parsed).toEqual({ shareId: SAMPLE_SHARE_ID })
  })

  it('replaces whitespace in the filename with dashes and drops .poentaro', () => {
    const { url, parsed } = buildAndParse('My   Great\tSong.poentaro')
    expect(url.searchParams.get('s')).toBe(`${SAMPLE_SHARE_ID}--My-Great-Song`)
    expect(parsed).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: 'My-Great-Song.poentaro',
    })
  })

  it('round-trips a CJK filename', () => {
    const { parsed } = buildAndParse('彌勒淨土鄉.poentaro')
    expect(parsed).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: '彌勒淨土鄉.poentaro',
    })
  })

  it('keeps non-ASCII characters readable instead of percent-encoding them', () => {
    expect(buildSyncedShareUrl(SAMPLE_SHARE_ID, '彌勒淨土鄉.poentaro')).toBe(
      `https://example.test/?s=${SAMPLE_SHARE_ID}--彌勒淨土鄉`,
    )
  })

  it('still escapes characters that would break the query string', () => {
    const { parsed } = buildAndParse('a&b#c+d%e.poentaro')
    expect(parsed).toEqual({
      shareId: SAMPLE_SHARE_ID,
      filename: 'a&b#c+d%e.poentaro',
    })
  })
})
