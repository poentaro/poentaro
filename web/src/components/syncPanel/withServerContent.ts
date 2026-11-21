import { type FileStoreState, loadedFileText } from '../../fileStore'

/** `store` with the active file `fileId` set to the server's `content`. */
export function withServerContent(
  store: FileStoreState,
  fileId: string,
  content: string,
): FileStoreState {
  const name = Object.entries(store.fileIds).find(
    ([, id]) => id === fileId,
  )?.[0]
  if (name === undefined || !(name in store.userFiles)) return store
  return {
    ...store,
    userFiles: { ...store.userFiles, [name]: loadedFileText(content) },
  }
}
