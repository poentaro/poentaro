import {
  poentaroWasm,
  type SendRequest,
  type SendResult,
} from '../../poentaroWasm'
import {
  callWorker,
  type createWorkerClient,
  WorkerRequestError,
} from '../../syncedShare/workerClient'

const SEND_TIMEOUT_MS = 15_000

export interface SendMessageDeps {
  client: ReturnType<typeof createWorkerClient>
  identityToken: string
  /** The server's current text of the file, used to recognise a save whose
   * acknowledgement was lost (the server already holds our content). */
  fetchServerContent(fileId: string): Promise<string>
  /** The file's current local name; a `RestoreFile` message carries none. */
  restoreName(fileId: string): string
  timeoutMs?: number
}

/** Create and content saves answer with the file (and its new revision);
 * rename, trash and restore answer with an empty body. */
interface MaybeRevisionBody {
  revision?: number
}

function transient(reason: string): SendResult {
  return { tag: 'transient', val: { reason } }
}

/** Appends the worker's trace ID, so a failed save in the sync panel and
 * its history can be looked up in `app_log`. */
function withTraceId(reason: string, error: WorkerRequestError): string {
  return error.traceId ? `${reason}, trace ID ${error.traceId}` : reason
}

function errorMessage(error: unknown): string {
  return error instanceof Error ? error.message : String(error)
}

/** The name a `NameTaken` suggestion should be derived from, for the message
 * kinds that carry one. */
function rejectedName(
  request: SendRequest,
  deps: SendMessageDeps,
): string | null {
  const { message } = request
  switch (message.tag) {
    case 'create-file':
      return message.val.name
    case 'rename-file':
      return message.val.to
    case 'restore-file':
      return deps.restoreName(request.fileId)
    default:
      return null
  }
}

async function post(
  request: SendRequest,
  deps: SendMessageDeps,
  signal: AbortSignal,
): Promise<MaybeRevisionBody | undefined> {
  const { client, identityToken } = deps
  const { fileId: id, message } = request
  switch (message.tag) {
    case 'create-file':
      return callWorker(
        client.POST('/files', {
          signal,
          body: {
            identityToken,
            id,
            name: message.val.name,
            content: message.val.content,
          },
        }),
      )
    case 'save-content':
      return callWorker(
        client.POST('/files/{id}/content', {
          signal,
          params: { path: { id } },
          body: {
            identityToken,
            content: message.val.content,
            expectedRevision: Number(request.expectedRevision),
          },
        }),
      )
    case 'rename-file':
      return callWorker(
        client.POST('/files/{id}/rename', {
          signal,
          params: { path: { id } },
          body: { identityToken, name: message.val.to },
        }),
      )
    case 'rename-files':
      return callWorker(
        client.POST('/files/rename-batch', {
          signal,
          body: {
            identityToken,
            moves: message.val.moves.map(({ fileId, to }) => ({
              id: fileId,
              name: to,
            })),
          },
        }),
      )
    case 'trash-file':
      return callWorker(
        client.POST('/files/{id}/delete', {
          signal,
          params: { path: { id } },
          body: { identityToken },
        }),
      )
    case 'restore-file':
      return callWorker(
        client.POST('/files/{id}/restore', {
          signal,
          params: { path: { id } },
          body: { identityToken, name: deps.restoreName(id) },
        }),
      )
  }
}

/** A revision conflict on a save is not a conflict when the server already
 * holds exactly our text: the earlier response was lost, not the write. */
async function classifyRevisionConflict(
  request: SendRequest,
  currentRevision: number,
  deps: SendMessageDeps,
): Promise<SendResult> {
  const { message } = request
  if (message.tag !== 'save-content') {
    return {
      tag: 'permanent',
      val: { status: 409, reason: 'revision_conflict' },
    }
  }
  const serverContent = await deps.fetchServerContent(request.fileId)
  return serverContent === message.val.content
    ? { tag: 'ok', val: { revision: BigInt(currentRevision) } }
    : { tag: 'conflict', val: { currentRevision: BigInt(currentRevision) } }
}

async function classifyRequestError(
  error: WorkerRequestError,
  request: SendRequest,
  deps: SendMessageDeps,
): Promise<SendResult> {
  const { apiError } = error
  const { status } = error.response
  if (apiError?.code === 'unauthorized' || status === 401) {
    return { tag: 'unauthorized' }
  }
  if (apiError?.code === 'revision_conflict') {
    return classifyRevisionConflict(request, apiError.currentRevision, deps)
  }
  if (apiError?.code === 'names_taken') {
    // The batch was refused as a whole. Each taken destination gets the next
    // free numbered sibling in its folder (computed in Rust); a folder's
    // `.keep` marker that already exists is the folder itself, so that move
    // is dropped instead.
    return {
      tag: 'names-taken',
      val: {
        adjustments: apiError.names.map((rejected) => ({
          rejected,
          replacement: poentaroWasm().fileStoreIsPlaceholderPath(rejected)
            ? undefined
            : poentaroWasm().fileStoreNameAfterCollision(rejected),
        })),
      },
    }
  }
  if (apiError?.code === 'name_taken') {
    const name = rejectedName(request, deps)
    // A folder's `.keep` placeholder already in the cloud (another device
    // made the same folder) is the folder itself: never rename it to a
    // visible `.keep 2.poentaro`.
    if (name !== null && poentaroWasm().fileStoreIsPlaceholderPath(name)) {
      return { tag: 'ok', val: { revision: request.expectedRevision } }
    }
    // The next free numbered sibling in the same folder, computed in Rust.
    const suggestedName =
      name === null
        ? undefined
        : poentaroWasm().fileStoreNameAfterCollision(name)
    return suggestedName === undefined
      ? { tag: 'permanent', val: { status: 409, reason: 'name_taken' } }
      : { tag: 'name-taken', val: { suggestedName } }
  }
  // A trash or restore the server already applied (its answer was lost) is
  // gone or back already, so the retry finds nothing to act on.
  if (
    status === 404 &&
    (request.message.tag === 'trash-file' ||
      request.message.tag === 'restore-file')
  ) {
    return { tag: 'ok', val: { revision: request.expectedRevision } }
  }
  if (status >= 500 || status === 429) {
    return transient(withTraceId(`status ${status}`, error))
  }
  return {
    tag: 'permanent',
    val: {
      status,
      reason: withTraceId(apiError?.code ?? `status ${status}`, error),
    },
  }
}

async function classifyError(
  error: unknown,
  request: SendRequest,
  deps: SendMessageDeps,
): Promise<SendResult> {
  try {
    if (error instanceof WorkerRequestError) {
      return await classifyRequestError(error, request, deps)
    }
  } catch (fetchError) {
    // Only the server-content fetch behind a revision conflict can throw.
    return transient(errorMessage(fetchError))
  }
  return transient(errorMessage(error))
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
  const controller = new AbortController()
  const timer = setTimeout(
    () => controller.abort(),
    deps.timeoutMs ?? SEND_TIMEOUT_MS,
  )
  try {
    const body = await post(request, deps, controller.signal)
    // An empty body leaves the file's revision where the queue recorded it.
    const revision = body?.revision ?? Number(request.expectedRevision)
    return { tag: 'ok', val: { revision: BigInt(revision) } }
  } catch (error) {
    return await classifyError(error, request, deps)
  } finally {
    clearTimeout(timer)
  }
}
