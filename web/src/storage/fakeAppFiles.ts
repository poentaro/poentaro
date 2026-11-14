import type {
  FilesFile,
  FilesFileSummary,
  FilesVersionsVersion,
} from '../generated/protocol/app'
import type { AppFileCalls } from '../protocol/clients'
import type { CallOutcome } from '../protocol/outcome'

/**
 * An in-memory `poentaro app` for the cloud backend's tests: `files/*` and
 * `folders/*` on a map of files, following the worker's rules closely
 * enough for the client's behaviour (taken paths, revisions, the trash,
 * folder moves). Every call is recorded by its method name. `failNext`
 * makes the next call of a method answer something else, or throw; `hang`
 * makes its calls never answer.
 */

export interface FakeFile {
  id: string
  path: string
  content: string
  revision?: number
  trashedAt?: number
}

export interface RecordedCall {
  method: string
  args: unknown[]
}

type Outcome = CallOutcome<unknown, unknown>

interface StoredFile {
  id: string
  path: string
  content: string
  revision: number
  trashedAt?: number
  versions: Array<{ revision: number; savedAt: number; content: string }>
}

const value = <T>(value: T) => ({ kind: 'value', value }) as const
const error = <E>(error: E) => ({ kind: 'error', error }) as const

function summaryOf(file: StoredFile): FilesFileSummary {
  return {
    id: file.id,
    path: file.path,
    revision: file.revision,
    ...(file.trashedAt === undefined ? {} : { trashedAt: file.trashedAt }),
  }
}

/** `stem 2.ext`, `stem 3.ext`, ...: the first path no file in `taken`
 * holds. */
function freePath(path: string, taken: (path: string) => boolean): string {
  if (!taken(path)) return path
  const dot = path.lastIndexOf('.')
  const stem = dot > path.lastIndexOf('/') ? path.slice(0, dot) : path
  const extension = path.slice(stem.length)
  for (let number = 2; ; number++) {
    const candidate = `${stem} ${number}${extension}`
    if (!taken(candidate)) return candidate
  }
}

const EVERY_METHOD = '*'

export function createFakeAppFiles(seed: FakeFile[] = []) {
  const files = new Map<string, StoredFile>()
  const calls: RecordedCall[] = []
  const failures = new Map<string, Array<Outcome | Error>>()
  const hanging = new Set<string>()
  let clock = 1_000
  let latencyMs = 0
  let inFlight = 0
  let maxInFlight = 0

  const live = () => [...files.values()].filter((file) => !file.trashedAt)
  const holder = (path: string) => live().find((file) => file.path === path)

  function seedFiles(seeded: FakeFile[]): void {
    for (const file of seeded) {
      files.set(file.id, {
        id: file.id,
        path: file.path,
        content: file.content,
        revision: file.revision ?? 0,
        trashedAt: file.trashedAt,
        versions: [],
      })
    }
  }
  seedFiles(seed)

  /** Records the call, then answers the queued failure or runs `body`. */
  async function call<T>(
    method: string,
    args: unknown[],
    body: () => CallOutcome<T, unknown>,
  ): Promise<never> {
    calls.push({ method, args })
    if (hanging.has(method) || hanging.has(EVERY_METHOD)) {
      return new Promise(() => undefined)
    }
    inFlight++
    maxInFlight = Math.max(maxInFlight, inFlight)
    try {
      if (latencyMs > 0) {
        await new Promise((resolve) => setTimeout(resolve, latencyMs))
      }
      const failure = failures.get(method)?.shift()
      if (failure instanceof Error) throw failure
      return (failure ?? body()) as never
    } finally {
      inFlight--
    }
  }

  function save(file: StoredFile, content: string): number {
    file.versions.unshift({
      revision: file.revision,
      savedAt: clock++,
      content: file.content,
    })
    file.content = content
    file.revision += 1
    return file.revision
  }

  const app: AppFileCalls = {
    files: {
      list: () =>
        call('files/list', [], () => value([...files.values()].map(summaryOf))),
      read: (id) =>
        call('files/read _', [id], () => {
          const file = files.get(id)
          return file
            ? value<FilesFile>({
                summary: summaryOf(file),
                content: file.content,
              })
            : error({ kind: 'missing' })
        }),
      writeAtRevisionContent: (id, revision, content) =>
        call(
          'files/write _ at revision _ content _',
          [id, revision, content],
          () => {
            const file = files.get(id)
            if (!file) return error({ kind: 'missing' })
            if (file.trashedAt) return error({ kind: 'trashed' })
            if (file.revision !== revision) {
              return error({
                kind: 'conflictAtRevisionContent',
                revision: file.revision,
                content: file.content,
              })
            }
            return value(save(file, content))
          },
        ),
      createAtContent: (id, path, content) =>
        call('files/create _ at _ content _', [id, path, content], () => {
          if (files.has(id)) return error({ kind: 'idTaken' })
          if (holder(path)) return error({ kind: 'taken' })
          const file = { id, path, content, revision: 0, versions: [] }
          files.set(id, file)
          return value(summaryOf(file))
        }),
      moveTo: (id, path) =>
        call('files/move _ to _', [id, path], () => {
          const file = files.get(id)
          if (!file) return error({ kind: 'missing' })
          const other = holder(path)
          if (other && other.id !== id) return error({ kind: 'taken' })
          file.path = path
          return value(undefined)
        }),
      trash: (id) =>
        call('files/trash _', [id], () => {
          const file = files.get(id)
          if (!file) return error({ kind: 'missing' })
          file.trashedAt ??= clock++
          return value(undefined)
        }),
      restore: (id) =>
        call('files/restore _', [id], () => {
          const file = files.get(id)
          if (!file) return error({ kind: 'missing' })
          if (!file.trashedAt) return error({ kind: 'notTrashed' })
          file.path = freePath(file.path, (path) => !!holder(path))
          file.trashedAt = undefined
          return value(file.path)
        }),
      versions: {
        list: (id) =>
          call('files/versions/list _', [id], () => {
            const file = files.get(id)
            return file
              ? value<FilesVersionsVersion[]>(
                  file.versions.map(({ revision, savedAt }) => ({
                    revision,
                    savedAt,
                  })),
                )
              : error({ kind: 'missing' })
          }),
        readAtRevision: (id, revision) =>
          call('files/versions/read _ at revision _', [id, revision], () => {
            const file = files.get(id)
            if (!file) return error({ kind: 'missing' })
            const version = file.versions.find(
              (saved) => saved.revision === revision,
            )
            return version
              ? value(version.content)
              : error({ kind: 'noSuchVersion' })
          }),
      },
    },
    batch: (build) => Promise.all(build(app)),
    folders: {
      moveTo: (from, to) =>
        call('folders/move _ to _', [from, to], () => {
          const inside = (folder: string) => (file: StoredFile) =>
            file.path.startsWith(`${folder}/`)
          const moving = live().filter(inside(from))
          if (moving.length === 0) {
            return live().some(inside(to))
              ? value(live().filter(inside(to)).map(summaryOf))
              : error({ kind: 'noSuchFolder' })
          }
          if (live().some(inside(to))) {
            return error({ kind: 'takenByOtherFiles', paths: [to] })
          }
          for (const file of moving) {
            file.path = `${to}${file.path.slice(from.length)}`
          }
          return value(moving.map(summaryOf))
        }),
    },
  }

  return {
    /** Stands in for `appClient`; ignores the trace ID option. */
    app: async () => app,
    calls,
    /** The calls made to `method`, oldest first. */
    callsTo: (method: string) => calls.filter((made) => made.method === method),
    lastCall: () => calls.at(-1),
    /** The stored file `id`, as the server holds it. */
    file: (id: string) => files.get(id),
    /** The live file at `path`. */
    fileAt: (path: string) => holder(path),
    seed: seedFiles,
    /** The next call of `method` answers `outcome`, or throws it. */
    failNext(method: string, outcome: Outcome | Error): void {
      failures.set(method, [...(failures.get(method) ?? []), outcome])
    },
    /** Every later call of `method`, or of every method, never answers. */
    hang(method: string = EVERY_METHOD): void {
      hanging.add(method)
    },
    /** Every later call answers after `ms`. */
    slow(ms: number): void {
      latencyMs = ms
    },
    /** The most calls that were ever awaiting an answer at once. */
    maxInFlight: () => maxInFlight,
  }
}

export type FakeAppFiles = ReturnType<typeof createFakeAppFiles>
