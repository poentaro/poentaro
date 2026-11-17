import {
  type FileStoreState,
  fileIdForName,
  createFolder as pureCreateFolder,
  deleteFolder as pureDeleteFolder,
  moveFolder as pureMoveFolder,
} from '../fileStore'
import type { FolderSnapshot, RenameMove } from '../poentaroWasm'
import { importSharedFolderState } from '../sharedFolder'
import type { CloudPlaceholders } from './cloudBackendPlaceholders'
import type { OutboxLooper } from './outbox/outboxLooper'

interface FolderOperationDependencies {
  ensureWasm: () => Promise<void>
  whenReady: () => Promise<void>
  looper: OutboxLooper
  placeholders: CloudPlaceholders
  activeFileId: (state: FileStoreState, name: string) => string
  activeIdByName: Map<string, string>
  trashedIdByName: Map<string, string>
}

/** Folder create, rename/move and delete for the cloud backend. A folder
 * rename/move is ONE outbox `RenameFiles` message covering every file inside
 * and the folders' placeholders, delivered all-or-nothing; a delete trashes
 * each file on its own. Explicit folders are persisted as placeholder files
 * (`cloudBackendPlaceholders.ts`): created with the folder, renamed with it
 * and trashed with it; a placeholder left behind in a folder that later got
 * real files is harmless and stays. */
export function createFolderOperations({
  ensureWasm,
  whenReady,
  looper,
  placeholders,
  activeFileId,
  activeIdByName,
  trashedIdByName,
}: FolderOperationDependencies) {
  return {
    async createFolder(
      state: FileStoreState,
      parent: string | undefined,
      name: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureCreateFolder(state, parent, name)
      const before = state.folders ?? []
      for (const folder of nextState.folders ?? []) {
        if (!before.includes(folder)) await placeholders.create(folder)
      }
      return nextState
    },

    /** Imports a shared folder: every file becomes a queued create and every
     * explicit empty folder a placeholder; nothing existing is touched. */
    async importFolder(
      state: FileStoreState,
      snapshot: FolderSnapshot,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = importSharedFolderState(state, snapshot)
      await whenReady()
      for (const [name, content] of Object.entries(nextState.userFiles)) {
        if (name in state.userFiles) continue
        const id = fileIdForName(nextState, name)
        activeIdByName.set(name, id)
        await looper.enqueue(id, {
          tag: 'create-file',
          val: { name, content },
        })
      }
      const before = state.folders ?? []
      for (const folder of nextState.folders ?? []) {
        if (!before.includes(folder)) await placeholders.create(folder)
      }
      return nextState
    },

    async moveFolder(
      state: FileStoreState,
      from: string,
      to: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const result = pureMoveFolder(state, from, to)
      if (!result.ok) return state
      await whenReady()
      const moves: RenameMove[] = []
      for (const change of result.changes) {
        if (change.kind === 'explicit-folder') {
          const placeholderMove = await placeholders.rename(
            change.oldPath,
            change.newPath,
          )
          if (placeholderMove) moves.push(placeholderMove)
          continue
        }
        const id = activeFileId(state, change.oldPath)
        activeIdByName.delete(change.oldPath)
        activeIdByName.set(change.newPath, id)
        moves.push({ fileId: id, to: change.newPath })
      }
      if (moves.length > 0) {
        // The batch lives on a lane of its own; its id is chosen in Rust.
        await looper.enqueue('', { tag: 'rename-files', val: { moves } })
      }
      return result.state
    },

    async deleteFolder(
      state: FileStoreState,
      folder: string,
    ): Promise<FileStoreState> {
      await ensureWasm()
      const nextState = pureDeleteFolder(state, folder)
      if (nextState === state) return nextState
      await whenReady()
      for (const name of Object.keys(state.userFiles)) {
        if (name in nextState.userFiles) continue
        const id = activeFileId(state, name)
        activeIdByName.delete(name)
        trashedIdByName.set(name, id)
        await looper.enqueue(id, { tag: 'trash-file' })
      }
      const remaining = nextState.folders ?? []
      for (const folder of state.folders ?? []) {
        if (!remaining.includes(folder)) await placeholders.trash(folder)
      }
      return nextState
    },
  }
}
