import type { AppFileCalls } from '../protocol/clients'
import { expectValue, settled } from '../protocol/outcome'
import type { ServerFile } from './cloudBackendTypes'
import type { FileTextCache } from './fileTextCache'

/** One file of the account as the cloud backend lays out a listing. */
export interface ListedServerFile {
  id: string
  name: string
  revision: number
  trashedAt: number | null
  content: string
}

/** What the cloud backend reads from the server: one file, or every file
 * with its text. */
export function createServerReads(
  app: () => Promise<AppFileCalls>,
  texts: FileTextCache,
) {
  /** A cache that cannot be written only costs a later download. */
  const rememberText = (fileId: string, revision: number, text: string) =>
    texts.write(fileId, revision, text).catch(() => undefined)

  return {
    /** The server's revision and text of `fileId`; `undefined` when the
     * account has no such file. */
    async fetchServerFile(fileId: string): Promise<ServerFile | undefined> {
      const outcome = settled(await (await app()).readFile(fileId))
      if (outcome.kind !== 'value') return undefined
      const { summary, content } = outcome.value
      await rememberText(summary.id, summary.revision, content)
      return { revision: summary.revision, content }
    },

    /** Every file the account owns, the trash included, with its text: one
     * request for the list and one for every file's text. A file deleted in
     * between is left out. */
    async listFiles(): Promise<ListedServerFile[]> {
      const calls = await app()
      const summaries = await expectValue(calls.listFiles())
      const reads = await calls.batch((batch) =>
        summaries.map((summary) => batch.readFile(summary.id)),
      )
      return reads.map(settled).flatMap((read) => {
        if (read.kind === 'error') return []
        const file = read.value
        void rememberText(file.summary.id, file.summary.revision, file.content)
        return [
          {
            id: file.summary.id,
            name: file.summary.path,
            revision: file.summary.revision,
            trashedAt:
              file.summary.trashedAt == null
                ? null
                : file.summary.trashedAt.getTime(),
            content: file.content,
          },
        ]
      })
    },
  }
}
