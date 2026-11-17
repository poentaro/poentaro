import { importOriginMigration } from '../originMigration/importOriginMigration'
import { renameLegacyFiles, renameLegacyStorage } from './renameLegacyStorage'

/** Everything that must happen to saved data before the app first reads
 * it: data saved before the rename to Poentaro moves to the current names,
 * then data handed over from the app's old origin is imported, then any
 * file still ending in the legacy extension is renamed. */
export async function prepareStorage(): Promise<void> {
  try {
    await renameLegacyStorage()
  } catch (error) {
    console.warn('could not rename storage saved before the rename', error)
  }
  await importOriginMigration()
  try {
    await renameLegacyFiles()
  } catch (error) {
    console.warn('could not rename files saved before the rename', error)
  }
}
