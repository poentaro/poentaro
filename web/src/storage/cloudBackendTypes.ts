import type { FileStoreState } from '../fileStore'
import type { WorkerSchemas } from '../syncedShare/workerClient'
import type { BaseSnapshots } from './outbox/baseSnapshots'
import type { ConflictFlow } from './outbox/conflictFlow'
import type { OutboxLooper } from './outbox/outboxLooper'
import type { StorageBackend } from './types'

export interface CloudBackendConfig {
  /** The unified account sign-in token (`accountAuth.ts`) — the same
   * token already used to prove
   * ownership of a Synced Share link, sent as `identityToken` on every
   * `/files/*` request body (never as a header/query param — matches this
   * worker's `FileShareRequest` convention). */
  token: string
  /** The signed-in account's email, which scopes the on-device outbox. */
  account: string
  /** Bare host (no scheme), same shape as `useSyncedShareOwner.ts`'s
   * `VITE_SYNCED_SHARE_HOST` — turned into an origin via
   * `syncedShareWorkerOrigin`. */
  workerHost: string
  /** Called when the worker refuses a queued change because this session
   * has ended (signed out, expired, or a password change elsewhere), so the
   * app can drop the dead token. Not called for changes still waiting from
   * an earlier session, which this one picks up. */
  onSessionEnded?: () => void
}

/**
 * Typed detail behind a `'error'`/`'offline'` `SaveStatus`, mirroring
 * `GithubBackendError`'s role for the old GitHub backend. `'conflict'`
 * carries the server's current revision (for `forceOverwrite`/"Discard
 * mine"); `'auth'` is new here (401 -- the account session was
 * signed out or expired), replacing GitHub's `'rate-limited'` (the worker
 * rate-limits only sign-in and sign-up, never the `/files/*` routes).
 */
export type CloudBackendError =
  | { kind: 'conflict'; currentRevision: number }
  | { kind: 'auth' }
  | { kind: 'network' }
  | { kind: 'unknown'; message: string }

/** A local file renamed because the outbox had to pick another name after
 * the server reported the requested one as taken. */
export interface LocalRename {
  fileId: string
  from: string
  to: string
}

export interface CloudBackend extends StorageBackend {
  readonly kind: 'cloud'
  /** Detail behind the most recent `'error'`/`'offline'` status, if any. */
  lastError(): CloudBackendError | null
  /**
   * Used by `storageSettingsModalHelpers.ts`'s "Overwrite mine": realigns
   * the tracked revision for `state.active` to the server's last-reported
   * current revision (from a `'conflict'` `lastError`), then retries the
   * save with that revision -- so the CAS write that previously lost the
   * race now succeeds and re-pushes the caller's in-memory edit.
   */
  forceOverwrite(state: FileStoreState): Promise<void>
  /** Subscribes to renames the outbox made after a `name_taken` answer; the
   * caller applies them to its file state (see `withRenamedKey`). */
  onLocalRename(listener: (change: LocalRename) => void): () => void
  /** The outbox that carries this backend's content saves. */
  outbox(): OutboxLooper
  /** Releases the leader lock and timers. The backend starts again by itself
   * the next time it is used, so a React effect cleanup may call this. */
  dispose(): void
  /** The last text the server is known to have, per file. */
  bases(): BaseSnapshots
  /** Settles `NeedsMerge` lanes; null until the first `load()`/save has
   * started the outbox. */
  conflicts(): ConflictFlow | null
  /** Subscribes to one-line notices such as "Merged with changes from
   * another device". */
  onNotice(listener: (message: string) => void): () => void
  /** The server's current revision and content of `fileId`, or `undefined`
   * when the server does not list it. */
  fetchServerFile(fileId: string): Promise<ServerFile | undefined>
  /** The earlier versions of `fileId`, newest first. */
  listVersions(fileId: string): Promise<FileVersionSummary[]>
  /** One earlier version of `fileId`, with the diff restoring it would make. */
  fetchVersion(fileId: string, revision: number): Promise<FileVersion>
}

export type FileVersionSummary = WorkerSchemas['FileVersionSummary']
export type FileVersion = WorkerSchemas['FileVersionResponse']

export interface ServerFile {
  revision: number
  content: string
}
