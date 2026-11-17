import { generateFileId } from '../fileStore'
import { poentaroWasm, type RenameMove } from '../poentaroWasm'
import type { OutboxLooper } from './outbox/outboxLooper'

/**
 * Explicit empty folders in the cloud. The worker only knows files, so a
 * folder `A/B` that was created empty is kept alive by the placeholder file
 * `A/B/.keep` (empty content). Which paths are placeholders, and what the
 * placeholder path of a folder is, is decided in Rust
 * (`src/file_tree/placeholder.rs`); this module only remembers each
 * placeholder's file id (so a folder rename, delete or re-create can address
 * its row) and queues the matching outbox messages. Placeholders never enter
 * `FileStoreState.userFiles`, `bin` or `fileIds`.
 */
interface PlaceholderRow {
  fileId: string
  /** Its folder was deleted: the row is hidden but still holds its name, so
   * creating the folder again restores it rather than creating a new row. */
  trashed: boolean
}

interface ListedRow {
  id: string
  name: string
  trashedAt?: number | null
}

export function createPlaceholders(
  looper: OutboxLooper,
  whenReady: () => Promise<void>,
) {
  const rowByFolder = new Map<string, PlaceholderRow>()

  const placeholderPath = (folder: string): string =>
    poentaroWasm().fileStorePlaceholderPath(folder) ?? ''

  return {
    /** Splits the placeholder rows out of a listing: the real files, and the
     * explicit folders the active placeholders stand for. */
    classify<Row extends ListedRow>(
      rows: Row[],
    ): { files: Row[]; folders: string[] } {
      const classification = poentaroWasm().fileStoreClassifyCloudListing(
        rows.map(({ id, name, trashedAt }) => ({
          id,
          name,
          trashed: trashedAt !== null && trashedAt !== undefined,
        })),
      )
      rowByFolder.clear()
      for (const { folder, fileId } of classification.trashedFolders) {
        rowByFolder.set(folder, { fileId, trashed: true })
      }
      for (const { folder, fileId } of classification.folders) {
        rowByFolder.set(folder, { fileId, trashed: false })
      }
      const hidden = new Set(classification.hiddenFileIds)
      return {
        files: rows.filter(({ id }) => !hidden.has(id)),
        folders: classification.folders.map(({ folder }) => folder),
      }
    },

    /** The current path of a trashed placeholder, for its `RestoreFile`. */
    nameOf(fileId: string): string | undefined {
      const folder = [...rowByFolder].find(
        ([, row]) => row.fileId === fileId,
      )?.[0]
      return folder === undefined ? undefined : placeholderPath(folder)
    },

    async create(folder: string): Promise<void> {
      const row = rowByFolder.get(folder)
      if (row && !row.trashed) return
      await whenReady()
      if (row) {
        rowByFolder.set(folder, { fileId: row.fileId, trashed: false })
        await looper.enqueue(row.fileId, { tag: 'restore-file' })
        return
      }
      const fileId = generateFileId()
      rowByFolder.set(folder, { fileId, trashed: false })
      await looper.enqueue(fileId, {
        tag: 'create-file',
        val: { name: placeholderPath(folder), content: '' },
      })
    },

    async trash(folder: string): Promise<void> {
      const row = rowByFolder.get(folder)
      if (!row || row.trashed) return
      await whenReady()
      rowByFolder.set(folder, { fileId: row.fileId, trashed: true })
      await looper.enqueue(row.fileId, { tag: 'trash-file' })
    },

    /** The folder `from` became `to`: the placeholder rename that belongs in
     * the folder move's batch, or undefined when there is none. If a deleted
     * folder's placeholder already holds the name `to/.keep`, that row is
     * brought back instead (a rename onto it would be a `name_taken`) and the
     * old one is trashed, which no batch can express. */
    async rename(from: string, to: string): Promise<RenameMove | undefined> {
      const row = rowByFolder.get(from)
      if (!row || row.trashed) return undefined
      const target = rowByFolder.get(to)
      if (target?.trashed) {
        await this.trash(from)
        await this.create(to)
        return undefined
      }
      await whenReady()
      rowByFolder.delete(from)
      rowByFolder.set(to, row)
      return { fileId: row.fileId, to: placeholderPath(to) }
    },
  }
}

export type CloudPlaceholders = ReturnType<typeof createPlaceholders>
