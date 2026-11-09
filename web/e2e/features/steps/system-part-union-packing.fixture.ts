import { expect } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { focusEditor } from '../../fileSwitcherHelpers'

/**
 * Part declaration order used throughout this feature:
 *   Melody [M] -> source_part_index 0
 *   Harmony [H] -> source_part_index 1
 *   Bass [B] -> source_part_index 2
 */
export const PART_INDEX: Record<string, number> = {
  Melody: 0,
  Harmony: 3,
  Bass: 4,
}

export type MeasureSpec = {
  /** Parts with plain notes in this measure, e.g. ['Melody', 'Harmony']. */
  notesFor: string[]
  /** Melody note tokens per lyric verse, e.g. [['do', 're'], ['fa', 'sol']]. */
  melodyVerses?: string[][]
  /** Directive line content for this measure group, e.g. 'merge_duplicate_measures_across_parts=no'. */
  directive?: string
  /** When set, every part in `notesFor` uses this part's note tokens
   * verbatim instead of its own default — used by the merge-scenario Given
   * steps, which need Melody and Harmony to have genuinely identical
   * content (the whole point of that scenario). */
  sameNotesAs?: string
}

/** Per-scenario accumulated fixture state, reset at the start of every
 * scenario's first `Given` step (see `resetState`). Subsequent `Given`
 * steps mutate it; `When the score is laid out` renders it into a
 * `.poentaro` source string and loads it. */
export let state: {
  maxMeasuresPerSystem: number
  hideRestingParts: boolean | null
  measures: MeasureSpec[]
} = { maxMeasuresPerSystem: 4, hideRestingParts: null, measures: [] }

export function resetState() {
  state = { maxMeasuresPerSystem: 4, hideRestingParts: null, measures: [] }
}

export function ensureMeasure(index: number): MeasureSpec {
  while (state.measures.length <= index) {
    state.measures.push({ notesFor: [] })
  }
  const measure = state.measures[index]
  if (!measure) throw new Error(`Measure ${index} was not created.`)
  return measure
}

/** Default note tokens written for a part in a measure with plain notes.
 * Deliberately distinct per part so two parts' rows never accidentally
 * collapse into one merged unison row (see
 * `merge_duplicate_measures_across_parts` in syntax.md) except in the one
 * scenario that wants that — which supplies identical tokens explicitly via
 * `measure.directive`. */
const NOTE_TOKENS: Record<string, string> = {
  Melody: '1 2 3 4',
  Harmony: '5 6 7 1',
  Bass: '3 3 3 3',
}
export const NOTE_TOKEN_COUNT = 4

/** Builds the `.poentaro` source from accumulated state. Also returns the
 * 1-based line number of the first measure's first content line, so the
 * "go to line" priming dance (see `primeMeasureSpans`) can put the cursor
 * somewhere guaranteed to fall inside a real measure span — a fixed line
 * number would be wrong across scenarios since the header's size (and thus
 * where the score content starts) varies with `hide_resting_parts`. */
function buildSource(): { source: string; firstMeasureLine: number } {
  const lines: string[] = [
    'metadata {',
    '  title "system part union packing test",',
    `  max measures per system ${state.maxMeasuresPerSystem},`,
  ]
  if (state.hideRestingParts !== null) {
    lines.push(
      `  hide resting parts: ${state.hideRestingParts ? 'yes' : 'no'},`,
    )
  }
  lines.push('},')
  lines.push('')
  lines.push('parts [')
  lines.push('  "Melody" [M]: notes,')
  lines.push('  "Melody verse 1" [Mv1]: lyrics [M],')
  lines.push('  "Melody verse 2" [Mv2]: lyrics [M],')
  lines.push('  "Harmony" [H]: notes,')
  lines.push('  "Bass" [B]: notes,')
  lines.push('],')
  lines.push('')
  lines.push('score """')

  const firstMeasureLine = lines.length + 1

  const groups: string[] = state.measures.map((measure) => {
    const groupLines: string[] = []
    if (measure.directive) {
      groupLines.push(measure.directive)
    }
    for (const part of ['Melody', 'Harmony', 'Bass']) {
      if (part === 'Melody' && measure.melodyVerses) {
        groupLines.push(`[M] ${NOTE_TOKENS.Melody}`)
        measure.melodyVerses.forEach((verse, verseIndex) => {
          groupLines.push(`[Mv${verseIndex + 1}] ${verse.join(' ')}`)
        })
      } else if (measure.notesFor.includes(part)) {
        const abbrev = part === 'Melody' ? 'M' : part === 'Harmony' ? 'H' : 'B'
        // The merge scenario deliberately wants Melody and Harmony to have
        // *identical* note content (that's what makes them mergeable), so it
        // sets `sameNotesAs` to reuse another part's tokens verbatim.
        const tokens = measure.sameNotesAs
          ? NOTE_TOKENS[measure.sameNotesAs]
          : NOTE_TOKENS[part]
        groupLines.push(`[${abbrev}] ${tokens}`)
      }
    }
    return groupLines.join('\n')
  })

  lines.push(groups.join('\n\n'))
  lines.push('""",')
  return { source: lines.join('\n'), firstMeasureLine }
}

export async function loadFixture(
  page: import('@playwright/test').Page,
): Promise<number> {
  const { source, firstMeasureLine } = buildSource()
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'system-part-union-packing-test.poentaro',
        userFiles: { 'system-part-union-packing-test.poentaro': src },
        bin: {},
        fileIds: {
          'system-part-union-packing-test.poentaro':
            'system-part-union-packing-test-id-001',
        },
      }),
    )
  }, source)
  return firstMeasureLine
}

/** Waits for measureSpans to be primed (same priming dance the measure-select
 * specs use) so the SVG has settled before hit-testing. Jumps the cursor
 * (via CodeMirror's built-in "go to line" on Ctrl/Cmd+g) to `line` — a line
 * known to fall inside the first measure's own source span, since the
 * button/highlight only activate once the cursor position resolves to a
 * real measure. */
export async function primeMeasureSpans(
  page: import('@playwright/test').Page,
  line: number,
) {
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type(String(line))
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
  await expect(
    page.locator('.preview-page [data-testid="measure-highlight"]').first(),
  ).toBeVisible({ timeout: 5_000 })
}

export function partLabelsFor(
  page: import('@playwright/test').Page,
  part: string,
) {
  return page.locator(
    tagSelector('part-label', { sourcePartIndex: PART_INDEX[part] }),
  )
}

export function partLabelAt(
  page: import('@playwright/test').Page,
  part: string,
  measureIndexStart: number,
) {
  return page.locator(
    tagSelector('part-label', {
      sourcePartIndex: PART_INDEX[part],
      measureIndexStart,
    }),
  )
}
