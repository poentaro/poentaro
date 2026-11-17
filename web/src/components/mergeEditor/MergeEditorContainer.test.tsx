import { describe, expect, it, vi } from 'vitest'

vi.mock('@monaco-editor/react', () => ({ DiffEditor: () => null }))

import type { ConflictDetails } from '../../storage/outbox/conflictFlow'
import type { OutboxLooper } from '../../storage/outbox/outboxLooper'
import { mergeExits } from './MergeEditorContainer.tsx'

const conflict: ConflictDetails = {
  fileId: 'id-a',
  theirs: 'theirs',
  mine: 'mine',
  mergedWithMarkers: '<<<<<<< mine\nmine\n=======\ntheirs\n>>>>>>> theirs\n',
  serverRevision: 7n,
}

function setup() {
  const resolve = vi.fn(async () => undefined)
  const props = {
    conflict,
    fileName: 'a.poentaro',
    looper: { resolve } as unknown as OutboxLooper,
    onKeepBoth: vi.fn(async () => undefined),
    onDiscard: vi.fn(),
    onClose: vi.fn(),
  }
  return { resolve, props, exits: mergeExits(props) }
}

describe('merge editor exits', () => {
  it('saves the merged text against the conflicting server revision', async () => {
    const { resolve, props, exits } = setup()
    await exits.save('merged')
    expect(resolve).toHaveBeenCalledWith('id-a', {
      tag: 'merged-and-save',
      val: { content: 'merged', serverRevision: 7n },
    })
    expect(props.onClose).toHaveBeenCalled()
  })

  it('keeps both files and closes', async () => {
    const { props, exits } = setup()
    await exits.keepBoth()
    expect(props.onKeepBoth).toHaveBeenCalledWith('id-a')
    expect(props.onClose).toHaveBeenCalled()
  })

  it('hands discarding to the confirmation dialog', () => {
    const { resolve, props, exits } = setup()
    exits.discard()
    expect(props.onDiscard).toHaveBeenCalledWith('id-a')
    expect(resolve).not.toHaveBeenCalled()
  })
})
