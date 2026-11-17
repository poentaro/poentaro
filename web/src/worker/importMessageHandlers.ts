import { poentaroWasm } from '../poentaroWasm'
import type { WorkerRequest, WorkerResponse } from './poentaro.worker'

export function handleImportFromFile(
  msg: Extract<WorkerRequest, { type: 'importFromFile' }>,
): void {
  const bytes = new Uint8Array(msg.bytes)
  const source =
    msg.kind === 'svg'
      ? poentaroWasm().extractSourceFromSvg(bytes)
      : poentaroWasm().extractSourceFromPdf(bytes)

  if (source === undefined) {
    postMessage({ type: 'importErr', id: msg.id } satisfies WorkerResponse)
    return
  }

  postMessage({
    type: 'importOk',
    id: msg.id,
    source,
  } satisfies WorkerResponse)
}
