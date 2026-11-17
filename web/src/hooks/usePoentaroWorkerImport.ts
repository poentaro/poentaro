import type { RefObject } from 'react'
import { useCallback } from 'react'
import type { WorkerRequest } from '../worker/poentaro.worker'

interface UsePoentaroWorkerImportParams {
  workerRef: RefObject<Worker | null>
  importRequestIdRef: RefObject<number>
  pendingImportsRef: RefObject<
    Map<
      number,
      { resolve: (source: string) => void; reject: (error: Error) => void }
    >
  >
}

function kindFromFileName(name: string): 'svg' | 'pdf' {
  return name.toLowerCase().endsWith('.pdf') ? 'pdf' : 'svg'
}

/**
 * Recovers the `.poentaro` source embedded in a previously exported SVG/PDF
 * file by round-tripping the file's bytes through the worker (see
 * `extract_source_from_svg`/`extract_source_from_pdf` in `poentaro-wasm`).
 */
export function usePoentaroWorkerImport({
  workerRef,
  importRequestIdRef,
  pendingImportsRef,
}: UsePoentaroWorkerImportParams) {
  const importFromFile = useCallback(
    async (file: File): Promise<string> => {
      const worker = workerRef.current
      if (!worker) throw new Error('Worker is not ready.')

      const bytes = await file.arrayBuffer()
      const kind = kindFromFileName(file.name)

      return new Promise<string>((resolve, reject) => {
        const id = ++importRequestIdRef.current
        pendingImportsRef.current.set(id, { resolve, reject })
        worker.postMessage(
          { type: 'importFromFile', id, bytes, kind } satisfies WorkerRequest,
          { transfer: [bytes] },
        )
      })
    },
    [workerRef, importRequestIdRef, pendingImportsRef],
  )

  return { importFromFile }
}
