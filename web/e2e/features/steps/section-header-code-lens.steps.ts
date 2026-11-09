import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const COMMENTED_SOURCE = [
  '# about the song',
  'metadata { title "Test" },',
  '',
  '# about the parts',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    """,',
].join('\n')

// The score string's text is not a section, so only the real `parts` gets a link.
const PARTS_LIKE_SCORE_LINE_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    parts ["Harmony" [H]: notes],',
  '    """,',
].join('\n')

async function loadSource(page: Page, source: string) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'code-lens.poentaro',
        userFiles: { 'code-lens.poentaro': src },
        bin: {},
        fileIds: { 'code-lens.poentaro': crypto.randomUUID() },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 30_000 })
}

function codeLensLinks(page: Page, title: string) {
  return page.locator('.codelens-decoration a', { hasText: title })
}

Given(
  'the section-header CodeLens fixture with comments is loaded',
  async ({ page }) => {
    await loadSource(page, COMMENTED_SOURCE)
  },
)

Given(
  'the section-header CodeLens fixture with a parts-like score line is loaded',
  async ({ page }) => {
    await loadSource(page, PARTS_LIKE_SCORE_LINE_SOURCE)
  },
)

Then(
  'the editor shows {int} {string} CodeLens link(s)',
  async ({ page }, count: number, title: string) => {
    await expect(codeLensLinks(page, title)).toHaveCount(count, {
      timeout: 15_000,
    })
  },
)

When('I click the {string} CodeLens link', async ({ page }, title: string) => {
  await codeLensLinks(page, title).click()
})
