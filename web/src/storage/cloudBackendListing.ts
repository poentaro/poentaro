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
}

/** What the cloud backend reads from the server: one file with its text, or
 * the listing of every file without text. */
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

    /** Every file the account owns, the trash included, without its text. */
    async listFiles(): Promise<ListedServerFile[]> {
      const summaries = await expectValue((await app()).listFiles())
      return summaries.map((summary) => ({
        id: summary.id,
        name: summary.path,
        revision: summary.revision,
        trashedAt:
          summary.trashedAt == null ? null : summary.trashedAt.getTime(),
      }))
    },
  }
}
