import type { AppFileCalls } from '../protocol/clients'
import { expectValue, settled } from '../protocol/outcome'
import type { ServerFile } from './cloudBackendTypes'

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
export function createServerReads(app: () => Promise<AppFileCalls>) {
  return {
    /** The server's revision and text of `fileId`; `undefined` when the
     * account has no such file. */
    async fetchServerFile(fileId: string): Promise<ServerFile | undefined> {
      const outcome = settled(await (await app()).files.read(fileId))
      return outcome.kind === 'value'
        ? {
            revision: outcome.value.summary.revision,
            content: outcome.value.content,
          }
        : undefined
    },

    /** Every file the account owns, the trash included, with its text. */
    async listFiles(): Promise<ListedServerFile[]> {
      const calls = await app()
      const summaries = await expectValue(calls.files.list())
      return Promise.all(
        summaries.map(async (summary) => {
          const file = await expectValue(calls.files.read(summary.id))
          return {
            id: summary.id,
            name: summary.path,
            revision: file.summary.revision,
            trashedAt: file.summary.trashedAt ?? null,
            content: file.content,
          }
        }),
      )
    },
  }
}
