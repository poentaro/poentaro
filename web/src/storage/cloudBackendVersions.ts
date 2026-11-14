import type { AppFileCalls } from '../protocol/clients'
import { expectValue } from '../protocol/outcome'
import type { CloudBackend } from './cloudBackendTypes'

/** A cloud file's version history: the versions the worker keeps for it,
 * and one version's text. */
export function createVersionCalls(
  app: () => Promise<AppFileCalls>,
): Pick<CloudBackend, 'listVersions' | 'fetchVersion'> {
  return {
    listVersions: async (fileId) =>
      expectValue((await app()).files.versions.list(fileId)),

    fetchVersion: async (fileId, revision) => ({
      content: await expectValue(
        (await app()).files.versions.readAtRevision(fileId, revision),
      ),
    }),
  }
}
