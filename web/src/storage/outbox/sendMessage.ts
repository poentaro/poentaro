import type {
  FolderMove,
  RenameMove,
  SendRequest,
  SendResult,
} from '../../poentaroWasm'
import { poentaroWasm } from '../../poentaroWasm'
import type { AppFileCalls, ClientOptions } from '../../protocol/clients'
import { settled } from '../../protocol/outcome'
import { SessionEnded } from '../../protocol/transport'
import {
  NetworkFailure,
  WorkerRequestError,
} from '../../syncedShare/workerClient'

const SEND_TIMEOUT_MS = 15_000

export interface SendMessageDeps {
  /** `poentaro app`'s file calls, reporting each response's trace ID to
   * `options.onTraceId`. */
  app(options: ClientOptions): Promise<AppFileCalls>
  timeoutMs?: number
}

/** A method error the outbox can't fix by retrying, as the sync panel
 * shows it: the status it would have had over REST and the error's kind. */
interface Refusal {
  status: number
  error: { kind: string }
}

function transient(reason: string): SendResult {
  return { tag: 'transient', val: { reason } }
}

function ok(revision: bigint | number): SendResult {
  return { tag: 'ok', val: { revision: BigInt(revision) } }
}

/** What a destination someone else holds becomes: a folder's `.keep`
 * placeholder that exists already is the folder itself, so the message is
 * done; any other path takes the next free numbered sibling (computed in
 * Rust). */
function nameTaken(request: SendRequest, path: string): SendResult {
  if (poentaroWasm().fileStoreIsPlaceholderPath(path)) {
    return ok(request.expectedRevision)
  }
  const suggestedName = poentaroWasm().fileStoreNameAfterCollision(path)
  return suggestedName === undefined
    ? { tag: 'permanent', val: { status: 409, reason: 'taken' } }
    : { tag: 'name-taken', val: { suggestedName } }
}

/** A batch refused as a whole: each taken destination gets the next free
 * numbered sibling (`nextFree`), or is dropped when it is a folder's `.keep`
 * marker that exists already. */
function namesTaken(
  paths: string[],
  nextFree: (rejected: string) => string | undefined = (rejected) =>
    poentaroWasm().fileStoreNameAfterCollision(rejected),
): SendResult {
  return {
    tag: 'names-taken',
    val: {
      adjustments: paths.map((rejected) => ({
        rejected,
        replacement: poentaroWasm().fileStoreIsPlaceholderPath(rejected)
          ? undefined
          : nextFree(rejected),
      })),
    },
  }
}

function refused(error: { kind: string }, status: number): Refusal {
  return { status, error }
}

/** Each move on its own, for a batch queued without its folder. A move
 * the server already made is a move to where the file is, which succeeds
 * again; a file gone since is skipped. */
async function moveEach(
  app: AppFileCalls,
  request: SendRequest,
  moves: RenameMove[],
): Promise<SendResult | Refusal> {
  const taken: string[] = []
  for (const move of moves) {
    const outcome = settled(await app.files.moveTo(move.fileId, move.to))
    if (outcome.kind === 'value') continue
    switch (outcome.error.kind) {
      case 'taken':
        taken.push(move.to)
        break
      case 'missing':
        break
      case 'invalidPath':
        return refused(outcome.error, 400)
      case 'wouldBeBothFileAndFolder':
        return refused(outcome.error, 409)
    }
  }
  return taken.length > 0 ? namesTaken(taken) : ok(request.expectedRevision)
}

async function moveFolder(
  app: AppFileCalls,
  request: SendRequest,
  folder: FolderMove,
): Promise<SendResult | Refusal> {
  const outcome = settled(await app.folders.moveTo(folder.from, folder.to))
  if (outcome.kind === 'value') return ok(request.expectedRevision)
  switch (outcome.error.kind) {
    // Nothing left in it: every file was moved or trashed since.
    case 'noSuchFolder':
      return ok(request.expectedRevision)
    // The destination folder exists already: move into the next numbered
    // folder instead. A file made inside it meanwhile is named on its own.
    case 'takenByOtherFiles':
      return namesTaken(outcome.error.paths, (rejected) =>
        rejected === folder.to
          ? poentaroWasm().fileStoreFolderAfterCollision(rejected)
          : poentaroWasm().fileStoreNameAfterCollision(rejected),
      )
    case 'invalidPath':
      return refused(outcome.error, 400)
    case 'wouldBeBothFileAndFolder':
      return refused(outcome.error, 409)
  }
}

async function createFile(
  app: AppFileCalls,
  request: SendRequest,
  name: string,
  content: string,
): Promise<SendResult | Refusal> {
  const id = request.fileId
  const outcome = settled(await app.files.createAtContent(id, name, content))
  if (outcome.kind === 'value') return ok(outcome.value.revision)
  switch (outcome.error.kind) {
    case 'taken':
      return nameTaken(request, name)
    // An earlier try made it and its answer was lost.
    case 'idTaken': {
      const file = settled(await app.files.read(id))
      return file.kind === 'value'
        ? ok(file.value.summary.revision)
        : refused(file.error, 409)
    }
    case 'invalidPath':
      return refused(outcome.error, 400)
    case 'wouldBeBothFileAndFolder':
      return refused(outcome.error, 409)
  }
}

async function saveContent(
  app: AppFileCalls,
  request: SendRequest,
  content: string,
): Promise<SendResult | Refusal> {
  const outcome = settled(
    await app.files.writeAtRevisionContent(
      request.fileId,
      Number(request.expectedRevision),
      content,
    ),
  )
  if (outcome.kind === 'value') return ok(outcome.value)
  switch (outcome.error.kind) {
    // The server holding exactly our text means the earlier answer was
    // lost, not the write.
    case 'conflictAtRevisionContent': {
      const current = outcome.error
      return current.content === content
        ? ok(current.revision)
        : {
            tag: 'conflict',
            val: { currentRevision: BigInt(current.revision) },
          }
    }
    case 'missing':
      return refused(outcome.error, 404)
    case 'trashed':
      return refused(outcome.error, 410)
  }
}

async function renameFile(
  app: AppFileCalls,
  request: SendRequest,
  to: string,
): Promise<SendResult | Refusal> {
  const outcome = settled(await app.files.moveTo(request.fileId, to))
  if (outcome.kind === 'value') return ok(request.expectedRevision)
  switch (outcome.error.kind) {
    case 'taken':
      return nameTaken(request, to)
    case 'missing':
      return refused(outcome.error, 404)
    case 'invalidPath':
      return refused(outcome.error, 400)
    case 'wouldBeBothFileAndFolder':
      return refused(outcome.error, 409)
  }
}

async function restoreFile(
  app: AppFileCalls,
  request: SendRequest,
): Promise<SendResult | Refusal> {
  const outcome = settled(await app.files.restore(request.fileId))
  if (outcome.kind === 'value') return ok(request.expectedRevision)
  switch (outcome.error.kind) {
    // An earlier try restored it and its answer was lost.
    case 'notTrashed':
      return ok(request.expectedRevision)
    case 'missing':
      return refused(outcome.error, 404)
    case 'wouldBeBothFileAndFolder':
      return refused(outcome.error, 409)
  }
}

/** Sends `request` and maps each method error to what the outbox does. */
async function deliver(
  app: AppFileCalls,
  request: SendRequest,
): Promise<SendResult | Refusal> {
  const { message } = request
  switch (message.tag) {
    case 'create-file':
      return createFile(app, request, message.val.name, message.val.content)
    case 'save-content':
      return saveContent(app, request, message.val.content)
    case 'rename-file':
      return renameFile(app, request, message.val.to)
    case 'rename-files':
      return message.val.folder
        ? moveFolder(app, request, message.val.folder)
        : moveEach(app, request, message.val.moves)
    // Trashing a file twice, or one gone since, leaves it trashed.
    case 'trash-file':
      settled(await app.files.trash(request.fileId))
      return ok(request.expectedRevision)
    case 'restore-file':
      return restoreFile(app, request)
  }
}

function isRefusal(result: SendResult | Refusal): result is Refusal {
  return 'error' in result
}

/** Appends the worker's trace ID, so a failed save in the sync panel and
 * its history can be looked up in `app_log`. */
function withTraceId(reason: string, traceId: string | null): string {
  return traceId ? `${reason}, trace ID ${traceId}` : reason
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

function timeout(ms: number): { promise: Promise<never>; cancel(): void } {
  let timer: ReturnType<typeof setTimeout> | undefined
  const promise = new Promise<never>((_, reject) => {
    timer = setTimeout(
      () => reject(new NetworkFailure(`no answer in ${ms}ms`)),
      ms,
    )
  })
  return { promise, cancel: () => clearTimeout(timer) }
}

/**
 * Delivers one outbox message and classifies the outcome into the
 * `SendResult` the Rust state machine consumes. Classification only -- what
 * to do about each outcome (retry, halt, merge) is decided in Rust.
 */
export async function sendMessage(
  request: SendRequest,
  deps: SendMessageDeps,
): Promise<SendResult> {
  let traceId: string | null = null
  const options: ClientOptions = {
    onTraceId: (id) => {
      traceId = id
    },
  }
  const timer = timeout(deps.timeoutMs ?? SEND_TIMEOUT_MS)
  try {
    const app = await deps.app(options)
    const result = await Promise.race([deliver(app, request), timer.promise])
    if (!isRefusal(result)) return result
    return {
      tag: 'permanent',
      val: {
        status: result.status,
        reason: withTraceId(result.error.kind, traceId),
      },
    }
  } catch (error) {
    if (error instanceof SessionEnded) return { tag: 'unauthorized' }
    if (error instanceof WorkerRequestError) {
      return transient(
        withTraceId(
          `status ${error.response.status}`,
          error.traceId ?? traceId,
        ),
      )
    }
    return transient(withTraceId(errorMessage(error), traceId))
  } finally {
    timer.cancel()
  }
}
