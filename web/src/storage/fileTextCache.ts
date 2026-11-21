import type { OutboxStore } from './outbox/outboxStore'

const TEXT_KEY_PREFIX = 'text:'

const textKey = (fileId: string): string => `${TEXT_KEY_PREFIX}${fileId}`

interface CachedText {
  revision: number
  text: string
}

/** The text a file had at a server revision, kept on this device so a file
 * that has not changed is not downloaded again. A hit needs the exact
 * revision: any other revision means the server's text may differ. */
export interface FileTextCache {
  read(fileId: string, revision: number): Promise<string | undefined>
  write(fileId: string, revision: number, text: string): Promise<void>
  drop(fileId: string): Promise<void>
}

function parseCached(value: string): CachedText | undefined {
  try {
    const parsed: unknown = JSON.parse(value)
    if (
      typeof parsed === 'object' &&
      parsed !== null &&
      'revision' in parsed &&
      typeof parsed.revision === 'number' &&
      'text' in parsed &&
      typeof parsed.text === 'string'
    ) {
      return { revision: parsed.revision, text: parsed.text }
    }
  } catch {
    // A record that does not parse is a miss.
  }
  return undefined
}

export function createFileTextCache(store: OutboxStore): FileTextCache {
  return {
    async read(fileId, revision) {
      const key = textKey(fileId)
      const record = (await store.loadAll()).find((item) => item.key === key)
      const cached = record ? parseCached(record.value) : undefined
      return cached?.revision === revision ? cached.text : undefined
    },
    write: (fileId, revision, text) =>
      store.apply({
        puts: [
          { key: textKey(fileId), value: JSON.stringify({ revision, text }) },
        ],
        deletes: [],
      }),
    drop: (fileId) => store.apply({ puts: [], deletes: [textKey(fileId)] }),
  }
}
