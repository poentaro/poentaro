import { afterEach, describe, expect, it, vi } from 'vitest'
import { downloadLocalCopy } from './downloadLocalCopy'

describe('downloadLocalCopy', () => {
  afterEach(() => vi.unstubAllGlobals())

  function setup() {
    const anchor = { href: '', download: '', click: vi.fn() }
    const createObjectURL = vi.fn(() => 'blob:fake')
    const revokeObjectURL = vi.fn()
    vi.stubGlobal('document', {
      createElement: vi.fn(() => anchor),
      body: { appendChild: vi.fn(), removeChild: vi.fn() },
    })
    vi.stubGlobal('URL', { createObjectURL, revokeObjectURL })
    return { anchor, createObjectURL, revokeObjectURL }
  }

  it('downloads a blob with the content and a .poentaro name', async () => {
    const { anchor, createObjectURL, revokeObjectURL } = setup()
    downloadLocalCopy('song', 'hello')
    const blob = (createObjectURL.mock.calls[0] as unknown as [Blob])[0]
    expect(await blob.text()).toBe('hello')
    expect(anchor.download).toBe('song.poentaro')
    expect(anchor.href).toBe('blob:fake')
    expect(anchor.click).toHaveBeenCalledOnce()
    expect(revokeObjectURL).toHaveBeenCalledWith('blob:fake')
  })

  it('keeps an existing .poentaro extension', () => {
    const { anchor } = setup()
    downloadLocalCopy('song.poentaro', 'x')
    expect(anchor.download).toBe('song.poentaro')
  })

  it('drops the folders from the name', () => {
    const { anchor } = setup()
    downloadLocalCopy('hymns/advent/song.poentaro', 'x')
    expect(anchor.download).toBe('song.poentaro')
  })
})
