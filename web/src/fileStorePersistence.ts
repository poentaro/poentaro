import { DEMO_FILE_NAMES } from './defaultSource'
import {
  DEFAULT_FILE_STORE,
  FILE_STORE_KEY,
  type FileStoreState,
  generateFileId,
  STORAGE_KEY,
} from './fileStore'

function ensureFileIds(
  userFiles: Record<string, string>,
  bin: Record<string, string>,
  existing: Record<string, string> | undefined,
): Record<string, string> {
  const fileIds = { ...existing }
  for (const name of Object.keys(userFiles)) {
    if (!fileIds[name]) fileIds[name] = generateFileId()
  }
  for (const name of Object.keys(bin)) {
    if (!fileIds[name]) fileIds[name] = generateFileId()
  }
  return fileIds
}

function normalizeState(parsed: Partial<FileStoreState>): FileStoreState {
  const userFiles = { ...parsed.userFiles }
  const bin = { ...parsed.bin }
  for (const name of DEMO_FILE_NAMES) {
    delete userFiles[name]
    delete bin[name]
  }

  const state: FileStoreState = {
    active: parsed.active ?? DEMO_FILE_NAMES[0] ?? '',
    userFiles,
    bin,
    fileIds: ensureFileIds(userFiles, bin, parsed.fileIds),
    // Optional and absent in stores saved before folders existed: no version
    // bump, old stores load with no explicit folders.
    ...(Array.isArray(parsed.folders) && parsed.folders.length > 0
      ? { folders: parsed.folders.filter((f) => typeof f === 'string') }
      : {}),
  }
  const names = [...DEMO_FILE_NAMES, ...Object.keys(userFiles)]
  return {
    ...state,
    active: names.includes(state.active)
      ? state.active
      : (DEMO_FILE_NAMES[0] ?? ''),
  }
}

function fileIdsNeedMigration(
  stored: Partial<FileStoreState>,
  normalized: FileStoreState,
): boolean {
  if (!stored.fileIds) return true
  for (const name of [
    ...Object.keys(normalized.userFiles),
    ...Object.keys(normalized.bin),
  ]) {
    if (!stored.fileIds[name]) return true
  }
  return false
}

function persistFileStoreMigration(
  raw: string,
  normalized: FileStoreState,
): void {
  try {
    const stored = JSON.parse(raw) as Partial<FileStoreState>
    if (fileIdsNeedMigration(stored, normalized)) {
      localStorage.setItem(FILE_STORE_KEY, JSON.stringify(normalized))
    }
  } catch {
    // ignore migration write failures
  }
}

export function parseStoredFileStore(raw: string): FileStoreState | null {
  try {
    const parsed = JSON.parse(raw) as Partial<FileStoreState>
    if (parsed && typeof parsed.active === 'string' && parsed.userFiles) {
      return normalizeState({
        ...parsed,
        bin: parsed.bin ?? {},
      })
    }
  } catch {
    // ignore corrupt storage
  }
  return null
}

function readLegacyFileStore(): FileStoreState | null {
  try {
    const legacy = localStorage.getItem(STORAGE_KEY)
    if (legacy != null) {
      const userFiles = { 'untitled.poentaro': legacy }
      return {
        active: 'untitled.poentaro',
        userFiles,
        bin: {},
        fileIds: ensureFileIds(userFiles, {}, undefined),
      }
    }
  } catch {
    // ignore
  }
  return null
}

export function readInitialFileStore(): FileStoreState {
  try {
    const raw = localStorage.getItem(FILE_STORE_KEY)
    if (raw != null) {
      const parsed = parseStoredFileStore(raw)
      if (parsed) {
        persistFileStoreMigration(raw, parsed)
        return parsed
      }
    }
  } catch {
    // ignore
  }

  return readLegacyFileStore() ?? DEFAULT_FILE_STORE
}

export function deserializeFileStore(raw: string): FileStoreState {
  const parsed = parseStoredFileStore(raw)
  if (parsed) {
    persistFileStoreMigration(raw, parsed)
    return parsed
  }
  return readLegacyFileStore() ?? DEFAULT_FILE_STORE
}
