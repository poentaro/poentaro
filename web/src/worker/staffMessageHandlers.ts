import type { VerovioToolkit } from 'verovio/esm'
import { type PageLayout, poentaroWasm } from '../poentaroWasm'
import { GM_INSTRUMENTS } from '../utils/gmInstruments'
import type {
  StaffEngraveResult,
  StaffRenderResult,
  WorkerRequest,
  WorkerResponse,
} from './messages'

/** A4 at Verovio's tenth-of-a-millimetre page units, matching the numbered
 * preview's page size. `condense: 'encoded'` plus the MEI's
 * `scoreDef optimize="true"` hides staves that are empty for a whole system. */
const VEROVIO_OPTIONS = {
  pageWidth: 2100,
  pageHeight: 2970,
  scale: 40,
  svgViewBox: true,
  breaks: 'smart',
  condense: 'encoded',
  condenseFirstPage: true,
  lyricSize: 4.5,
  header: 'auto',
  footer: 'none',
}

let toolkitPromise: Promise<VerovioToolkit> | null = null

/** Verovio is ~7 MB (wasm embedded), so it's only fetched the first time the
 * staff view asks for a render; later renders reuse the same toolkit. */
function loadToolkit(): Promise<VerovioToolkit> {
  toolkitPromise ??= Promise.all([
    import('verovio/wasm'),
    import('verovio/esm'),
  ])
    .then(async ([wasm, esm]) => {
      const toolkit = new esm.VerovioToolkit(await wasm.default())
      return toolkit
    })
    .catch((error: unknown) => {
      toolkitPromise = null
      throw error
    })
  return toolkitPromise
}

/** Verovio's page units are tenths of a millimetre at 100% scale; at
 * `scale: 40` one CSS pixel of rendered width is 2.5 of them. */
const PAGE_UNITS_PER_CSS_PIXEL = 100 / VEROVIO_OPTIONS.scale

/** Verovio's largest page height: tall enough for max view's single page. */
const MAX_PAGE_HEIGHT = 60000

/** A4 pages, or max view's one page as wide as the preview and as tall as
 * the score. */
function pageOptions(pageLayout: PageLayout) {
  if (pageLayout.tag === 'a4') {
    return { ...VEROVIO_OPTIONS, adjustPageHeight: false }
  }
  return {
    ...VEROVIO_OPTIONS,
    pageWidth: Math.round(pageLayout.val.widthPx * PAGE_UNITS_PER_CSS_PIXEL),
    pageHeight: MAX_PAGE_HEIGHT,
    adjustPageHeight: true,
  }
}

/** Engraves MEI (from Rust) into one SVG string per page of `pageLayout`. */
export async function engraveMei(
  mei: string,
  pageLayout: PageLayout,
): Promise<StaffEngraveResult> {
  try {
    const toolkit = await loadToolkit()
    toolkit.setOptions(pageOptions(pageLayout))
    if (!toolkit.loadData(mei)) {
      return { status: 'err', message: 'Verovio could not read the score.' }
    }
    const pages = Array.from({ length: toolkit.getPageCount() }, (_, index) =>
      toolkit.renderToSVG(index + 1),
    )
    return { status: 'ok', pages }
  } catch (error) {
    return {
      status: 'err',
      message: `Could not load the staff notation engine: ${String(error)}`,
    }
  }
}

async function renderStaff(
  msg: Extract<WorkerRequest, { type: 'renderStaff' }>,
): Promise<StaffRenderResult> {
  const mei = poentaroWasm().renderMei(
    msg.source,
    msg.visibility,
    GM_INSTRUMENTS,
    msg.clef,
  )
  if (mei.tag === 'err') {
    return {
      status: 'err',
      message: mei.val.diagnostics[0]?.message ?? 'Could not export the score.',
    }
  }
  const engraved = await engraveMei(mei.val.mei, msg.pageLayout)
  if (engraved.status === 'err') return engraved
  return {
    status: 'ok',
    score: { pages: engraved.pages, measures: mei.val.measures },
  }
}

export async function handleRenderStaff(
  msg: Extract<WorkerRequest, { type: 'renderStaff' }>,
): Promise<void> {
  postMessage({
    type: 'staffRendered',
    id: msg.id,
    result: await renderStaff(msg),
  } satisfies WorkerResponse)
}
