import type {
  CreateFileAtContentError,
  MoveFileToError,
} from '../src/generated/protocol/app'
import { settled } from '../src/protocol/outcome'
import { signInE2eAccount } from './e2eAccounts.ts'
import { appClient, publicClient, valueOrFail } from './protocolClients'

// Seeds and mutates real rows through the same generated AnyX clients as
// the app. Calls from Node bypass browser interception while exercising
// the worker's session verification and D1 writes.

/** Same origin `playwright.config.ts` points the app's own `VITE_SYNCED_SHARE_HOST`
 * at -- see that file's `webServer` entry for `cf dev --port 8797`
 * (deliberately not `just dev`'s 8787, see that entry's comment). */
export const CLOUD_WORKER_ORIGIN = 'http://localhost:8797'

// One session per account for the seeding calls below, signed in on first
// use. Cleared before every scenario (`clearSeedTokens`, from
// `cloud-account.steps.ts`), since a scenario may reset its account.
const seedTokens = new Map<string, Promise<string>>()

async function cachedSeedToken(email: string): Promise<string> {
  let token = seedTokens.get(email)
  if (!token) {
    token = signInE2eAccount(email)
    seedTokens.set(email, token)
  }
  return await token
}

/** The session token the seeding calls use for `email`. */
export async function seedTokenFor(email: string): Promise<string> {
  return await cachedSeedToken(email)
}

export function clearSeedTokens(): void {
  seedTokens.clear()
}

/** Deterministic default seed content -- distinct enough to tell apart from
 * a scenario's own edits in a failing assertion's diff, but never itself
 * asserted on by name. */
export const DEFAULT_CLOUD_FILE_CONTENT = [
  'metadata { title "Cloud Backend Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

/** Creates a new active cloud file directly against the real local worker,
 * bypassing the browser entirely. `id` mirrors `web/src/fileStore.ts`'s
 * `generateFileId()` (a plain `crypto.randomUUID()`) -- the server trusts
 * the client-generated id verbatim (see `CreateFileRequest`'s doc comment
 * in `protocol.rs`), so any unique string works here too.
 *
 * Idempotent across reruns of the same scenario: `scripts/resolve-e2e-
 * flakes.ts` reruns a still-failing scenario against the very same
 * already-running `cf dev` + local D1 instance (see
 * `playwright.config.ts`'s `reuseExistingServer`), so a plain `POST /files`
 * would fail with `name_taken` on any rerun once the first attempt's row is
 * already sitting there under this exact name -- see `reseedExistingFile`. */
export async function seedCloudFile(
  email: string,
  name: string,
  content: string = DEFAULT_CLOUD_FILE_CONTENT,
) {
  const client = await appClient(await seedTokenFor(email))
  const result = settled(
    await client.createFileAtContent(crypto.randomUUID(), name, content),
  )
  if (result.kind === 'value')
    return {
      id: result.value.id,
      name: result.value.path,
      content,
      revision: result.value.revision,
      trashedAt: null,
    }
  if (result.error.kind === 'taken')
    return reseedExistingFile(email, name, content)
  throw new Error(`seedCloudFile: ${JSON.stringify(result.error)}`)
}

/** Reuses a pre-existing row already sitting under `name` for this account
 * (restoring it out of the bin first if a previous run's scenario trashed
 * it) and overwrites its content, so a colliding `seedCloudFile` call ends
 * up in exactly the state a fresh create would have produced. */
async function reseedExistingFile(
  email: string,
  name: string,
  content: string,
) {
  const client = await appClient(await seedTokenFor(email))
  const files = await valueOrFail('list files', client.listFiles())
  const existing = files.find((file) => file.path === name)
  if (!existing) throw new Error(`No existing file at ${name}`)
  if (existing.trashedAt !== undefined)
    await valueOrFail('restore file', client.restoreFile(existing.id))
  const revision = await valueOrFail(
    'write file',
    client.writeFileAtRevisionContent(existing.id, existing.revision, content),
  )
  return { id: existing.id, name, content, revision, trashedAt: null }
}

/** What the worker answered a write: `ok`, or the `ApiError` code. */
export type CloudWriteOutcome =
  | 'ok'
  | CreateFileAtContentError['kind']
  | MoveFileToError['kind']

export async function tryCreateCloudFile(
  email: string,
  name: string,
): Promise<CloudWriteOutcome> {
  const client = await appClient(await seedTokenFor(email))
  const result = settled(
    await client.createFileAtContent(
      crypto.randomUUID(),
      name,
      DEFAULT_CLOUD_FILE_CONTENT,
    ),
  )
  return result.kind === 'error' ? result.error.kind : 'ok'
}
export async function tryRenameCloudFile(
  email: string,
  id: string,
  name: string,
): Promise<CloudWriteOutcome> {
  const client = await appClient(await seedTokenFor(email))
  const result = settled(await client.moveFileTo(id, name))
  return result.kind === 'error' ? result.error.kind : 'ok'
}
export async function trashCloudFile(email: string, id: string): Promise<void> {
  await valueOrFail(
    'trash file',
    (await appClient(await seedTokenFor(email))).trashFile(id),
  )
}
export async function renameCloudFile(
  email: string,
  id: string,
  name: string,
): Promise<void> {
  await valueOrFail(
    'move file',
    (await appClient(await seedTokenFor(email))).moveFileTo(id, name),
  )
}
export async function restoreCloudFile(
  email: string,
  id: string,
  name: string,
): Promise<void> {
  const client = await appClient(await seedTokenFor(email))
  const restored = await valueOrFail('restore file', client.restoreFile(id))
  if (restored !== name)
    await valueOrFail('move restored file', client.moveFileTo(id, name))
}
export async function fetchSyncedDoc(shareId: string) {
  return valueOrFail(
    'open shared link',
    (await publicClient()).openSharedLink(shareId),
  )
}
