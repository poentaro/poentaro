import { expect, type Page } from '@playwright/test'
import { anyxLightThemeRules } from '../../../src/anyx-monaco'
import { highlightKindStyles } from '../../../src/components/editorTheme'
import type { HighlightKind } from '../../../src/poentaroWasm'
import { Given, Then } from './fixtures'

const SOURCE = [
  '## Hymn',
  'metadata {',
  '    row height 30,',
  '},',
  '',
  'parts ["Melody" [MEL]: notes { volume : 80% }, "Echo" [E]: follow [MEL]],',
  '',
  'score """',
  '    bpm=92 break',
  '    [MEL] 1 2 3 4',
  '    """,',
].join('\n')

function hexToRgb(hex: string): string {
  const [red, green, blue] = [0, 2, 4].map((index) =>
    Number.parseInt(hex.slice(index, index + 2), 16),
  )
  return `rgb(${red}, ${green}, ${blue})`
}

function anyxColor(tokenClass: string): string {
  const rule = anyxLightThemeRules.find((rule) => rule.token === tokenClass)
  if (!rule?.foreground) throw new Error(`No anyx color for ${tokenClass}`)
  return hexToRgb(rule.foreground)
}

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
  'the editor text {string} is colored as an anyx {string}',
  async ({ page }, text: string, tokenClass: string) => {
    await expect
      .poll(() => editorTokenColor(page, text))
      .toBe(anyxColor(tokenClass))
  },
)

Then(
  'the editor text {string} is not colored as an anyx {string}',
  async ({ page }, text: string, tokenClass: string) => {
    await expect.poll(() => editorTokenColor(page, text)).not.toBe('missing')
    expect(await editorTokenColor(page, text)).not.toBe(anyxColor(tokenClass))
  },
)
