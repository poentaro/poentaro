import { expect, type Page } from '@playwright/test'
import { Given, Then, When } from './fixtures'

function sourceWithParts(partsLine: string) {
  return [
    'metadata { title "Test" },',
    '',
    `parts [${partsLine}],`,
    '',
    'score """',
    'time=4/4 key=C4 bpm=120',
    '[V] 1 4, 5 5',
    '""",',
  ].join('\n')
}

async function loadSource(page: Page, source: string) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'test.poentaro',
        userFiles: { 'test.poentaro': src },
        bin: {},
        fileIds: { 'test.poentaro': crypto.randomUUID() },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 30_000,
  })
}

Given(
  'the instrument-range fixture with a violin part is loaded',
  async ({ page }) => {
    await loadSource(
      page,
      sourceWithParts('"Violin" [V]: notes { instrument: violin }'),
    )
  },
)

Given(
  'the instrument-range fixture with a part naming no instrument is loaded',
  async ({ page }) => {
    await loadSource(page, sourceWithParts('"Violin" [V]: notes'))
  },
)

Then('the editor shows the warning {string}', async ({ page }, message) => {
  await expect(
    page.locator('.editor-warning-zone-message', { hasText: message }),
  ).toBeVisible({ timeout: 10_000 })
})

Then('the preview highlights a measure with a problem', async ({ page }) => {
  await expect(
    page.locator('.preview-page [data-testid="error-highlight"]').first(),
  ).toBeVisible({ timeout: 10_000 })
})

Then('the preview highlights no measure', async ({ page }) => {
  await expect(
    page.locator('.preview-page [data-testid="error-highlight"]'),
  ).toHaveCount(0)
})

When(
  'I change the instrument select for part {string} to {string}',
  async ({ page }, abbreviation: string, label: string) => {
    await page.getByTestId(`instrument-select-${abbreviation}`).click()
    await page.getByRole('option', { name: label, exact: true }).click()
  },
)

Given(
  'the instrument-range fixture with a part writing the range {string} is loaded',
  async ({ page }, range: string) => {
    await loadSource(page, sourceWithParts(`"Violin" [V]: notes { ${range} }`))
  },
)

When(
  'I change the range {word} select for part {string} to {string}',
  async ({ page }, bound: string, abbreviation: string, label: string) => {
    await page.getByTestId(`range-${bound}-select-${abbreviation}`).click()
    await page.getByRole('option', { name: label, exact: true }).click()
  },
)

Then(
  'the range {word} select for part {string} shows {string}',
  async ({ page }, bound: string, abbreviation: string, label: string) => {
    await expect(
      page.getByTestId(`range-${bound}-select-${abbreviation}`),
    ).toContainText(label)
  },
)
