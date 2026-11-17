const demoFragments = import.meta.glob('../../demo/*.poentaro', {
  query: '?raw',
  import: 'default',
  eager: true,
}) as Record<string, string>

export interface DemoFile {
  name: string
  content: string
}

/** One entry per fragment under the top-level `demo/` folder, in filename
 * order (numeric prefixes keep the intended reading order) — shown to users
 * as a folder of individually-selectable, read-only reference files. */
export const DEMO_FILES: DemoFile[] = Object.entries(demoFragments)
  .sort(([a], [b]) => a.localeCompare(b))
  .map(([path, content]) => ({
    name: path.slice(path.lastIndexOf('/') + 1),
    content,
  }))

export const DEMO_FILE_NAMES: string[] = DEMO_FILES.map((file) => file.name)
