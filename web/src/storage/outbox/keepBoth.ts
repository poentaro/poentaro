import { FILE_EXTENSION } from '../../fileExtension'
import type { FileStoreState } from '../../fileStore'
import type { CloudBackend } from '../cloudBackendTypes'

export interface KeepBothInput {
  backend: CloudBackend
  state: FileStoreState
  fileId: string
  /** The date in the copy's name, `YYYY-MM-DD`. */
  today: string
}

function nameOfFile(state: FileStoreState, fileId: string): string | undefined {
  return Object.entries(state.fileIds).find(([, id]) => id === fileId)?.[0]
}

/** `a.poentaro` -> `a (conflicted copy 2026-10-10).poentaro` */
function conflictedCopyName(name: string, today: string): string {
  const dot = name.lastIndexOf('.')
  const stem = dot > 0 ? name.slice(0, dot) : name
  const extension = dot > 0 ? name.slice(dot) : FILE_EXTENSION
  return `${stem} (conflicted copy ${today})${extension}`
}

/**
 * Saves the local text as a new file, then drops the queued edits of the
 * original so it takes the server's version. Returns the resulting state:
 * both files present, the original still the open one.
 */
export async function keepBoth(input: KeepBothInput): Promise<FileStoreState> {
  const { backend, state, fileId, today } = input
  const name = nameOfFile(state, fileId)
  if (name === undefined) return state
  const withCopy = await backend.importFile(
    state,
    conflictedCopyName(name, today),
    state.userFiles[name] ?? '',
  )
  await backend.outbox().resolve(fileId, { tag: 'discard' })
  const server = await backend.fetchServerFile(fileId)
  return {
    ...withCopy,
    active: state.active,
    userFiles: server
      ? { ...withCopy.userFiles, [name]: server.content }
      : withCopy.userFiles,
  }
}
