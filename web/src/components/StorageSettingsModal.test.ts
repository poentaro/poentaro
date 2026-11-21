import { describe, expect, it, vi } from 'vitest'
import type { FileStoreState } from '../fileStore'
import { loadedFiles, loadedText } from '../fileStore'
import type { CloudBackend } from '../storage/cloudBackend'
import { resolveCloudConflict } from './StorageSettingsModal'

function makeStore(active: string, content: string): FileStoreState {
  return {
    active,
    userFiles: loadedFiles({ [active]: content }),
    bin: loadedFiles({}),
    fileIds: { [active]: 'id-1' },
  }
}

describe('resolveCloudConflict', () => {
  it('overwrite-mine realigns the revision and re-pushes the current in-memory content, keeping the store unchanged', async () => {
    const forceOverwrite = vi.fn().mockResolvedValue(undefined)
    const backend = { forceOverwrite } as unknown as CloudBackend
    const store = makeStore('a.poentaro', 'mine')

    const result = await resolveCloudConflict('overwrite-mine', backend, store)

    expect(forceOverwrite).toHaveBeenCalledWith(store)
    expect(result).toBe(store)
  })

  it('discard-mine reads the file from the server and replaces the active file content with the remote version', async () => {
    const fetchServerFile = vi
      .fn()
      .mockResolvedValue({ id: 'id-1', content: 'theirs', revision: 3 })
    const updateActiveContent = vi.fn(
      (state: FileStoreState, content: string) => ({
        ...state,
        userFiles: loadedFiles({ [state.active]: content }),
      }),
    )
    const backend = {
      fetchServerFile,
      updateActiveContent,
    } as unknown as CloudBackend
    const store = makeStore('a.poentaro', 'mine')

    const result = await resolveCloudConflict('discard-mine', backend, store)

    expect(fetchServerFile).toHaveBeenCalledWith('id-1')
    expect(updateActiveContent).toHaveBeenCalledWith(store, 'theirs')
    expect(loadedText(result, 'a.poentaro')).toBe('theirs')
  })
})
