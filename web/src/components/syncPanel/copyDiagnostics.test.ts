import { afterEach, describe, expect, it, vi } from 'vitest'
import { buildDiagnostics, copyDiagnostics } from './copyDiagnostics'
import type { LaneView } from './syncPanelTypes'

const lane: LaneView = {
  fileId: 'f1',
  fileName: 'song.poentaro',
  status: 'possibly-stuck',
  sentence: 'Stuck',
  retryInSeconds: 30,
  messages: [{ kindLabel: 'Edit', sizeLabel: '2 KB', createdLabel: '1m ago' }],
  attempts: [{ atLabel: '10:00', outcomeLabel: 'Timed out' }],
}

describe('buildDiagnostics', () => {
  it('reports lane, status, messages and attempts', () => {
    const text = buildDiagnostics([lane])
    expect(text).toContain('song.poentaro')
    expect(text).toContain('possibly-stuck')
    expect(text).toContain('Edit, 2 KB, 1m ago')
    expect(text).toContain('10:00: Timed out')
    expect(text).toContain('Retry in: 30s')
  })

  it('never includes file contents', () => {
    const withPayload = { ...lane, content: 'SECRET-PAYLOAD' } as LaneView
    expect(buildDiagnostics([withPayload])).not.toContain('SECRET-PAYLOAD')
  })
})

describe('copyDiagnostics', () => {
  afterEach(() => vi.unstubAllGlobals())

  it('writes to the clipboard', async () => {
    const writeText = vi.fn().mockResolvedValue(undefined)
    vi.stubGlobal('navigator', { clipboard: { writeText } })
    expect(await copyDiagnostics('abc')).toBe('copied')
    expect(writeText).toHaveBeenCalledWith('abc')
  })

  it('selects text when the clipboard rejects', async () => {
    vi.stubGlobal('navigator', {
      clipboard: { writeText: vi.fn().mockRejectedValue(new Error('no')) },
    })
    const area = { value: '', select: vi.fn() }
    vi.stubGlobal('document', {
      createElement: vi.fn(() => area),
      body: { appendChild: vi.fn() },
    })
    expect(await copyDiagnostics('abc')).toBe('selected')
    expect(area.value).toBe('abc')
    expect(area.select).toHaveBeenCalledOnce()
  })
})
