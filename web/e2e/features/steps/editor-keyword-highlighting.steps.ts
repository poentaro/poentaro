import { expect, type Page } from '@playwright/test'
import { highlightKindStyles } from '../../../src/components/editorTheme'
import type { HighlightKind } from '../../../src/poentaroWasm'
import { Given, Then } from './fixtures'

const SOURCE = [
  'metadata {',
  '    row height 30,',
  '    colour: red,',
  '},',
  '',
  'parts ["Melody" [M]: notes, "Echo" [E]: follow [M]],',
  '',
  'score """',
  '    bpm=92 break',
  '    [M] 1 2 3 4',
  '    """,',
].join('\n')

function hexToRgb(hex: string): string {
  const [red, green, blue] = [0, 2, 4].map((index) =>
    Number.parseInt(hex.slice(index, index + 2), 16),
  )
  return `rgb(${red}, ${green}, ${blue})`
}

const keywordColors = Object.values(highlightKindStyles).map(({ foreground }) =>
  hexToRgb(foreground),
)

/**
 * Color of the shortest run of rendered editor spans starting with `text`
 * (Monarch may still split one keyword into several spans, e.g. at `_`, and
 * an uncolored word shares its span with what follows), or `mixed` when the
 * run is not uniformly colored, `missing` when absent.
 */
async function editorTokenColor(page: Page, text: string): Promise<string> {
  return page
    .locator('.monaco-editor .view-line')
    .evaluateAll((lines, wanted) => {
      for (const line of lines) {
        const spans = [...line.querySelectorAll('span span')]
        for (const [start] of spans.entries()) {
          let joined = ''
          const colors = new Set<string>()
          for (const span of spans.slice(start)) {
            const text = (span.textContent ?? '').replaceAll('\u00a0', ' ')
            joined = (joined + text).trimStart()
            if (text.trim()) colors.add(getComputedStyle(span).color)
            if (joined.startsWith(wanted)) {
              return colors.size === 1 ? [...colors].join() : 'mixed'
            }
            if (!wanted.startsWith(joined)) break
          }
        }
      }
      return 'missing'
    }, text)
}

Given('the editor keyword-highlighting fixture is loaded', async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'highlighting.poentaro',
        userFiles: { 'highlighting.poentaro': src },
        bin: {},
        fileIds: { 'highlighting.poentaro': crypto.randomUUID() },
      }),
    )
  }, SOURCE)
  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Then(
  'the editor text {string} is colored as a {string} keyword',
  async ({ page }, text: string, kind: string) => {
    const { foreground } = highlightKindStyles[kind as HighlightKind]
    await expect
      .poll(() => editorTokenColor(page, text))
      .toBe(hexToRgb(foreground))
  },
)

Then(
  'the editor text {string} is not colored as a keyword',
  async ({ page }, text: string) => {
    // Wait until highlighting has run, so a later keyword color can't
    // still arrive.
    await expect
      .poll(() => editorTokenColor(page, 'height'))
      .toBe(hexToRgb(highlightKindStyles['metadata-key'].foreground))
    const color = await editorTokenColor(page, text)
    expect(color).not.toBe('missing')
    expect(keywordColors).not.toContain(color)
  },
)
