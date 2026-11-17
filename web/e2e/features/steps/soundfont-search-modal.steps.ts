import { expect, test } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M]],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    [Mv1] twin- kle twin- kle',
  '    """,',
].join('\n')

async function loadSource(page: import('@playwright/test').Page) {
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
  }, SOURCE)
}

async function waitForEditor(page: import('@playwright/test').Page) {
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 30_000 })
}

async function openEditPartsModal(page: import('@playwright/test').Page) {
  await waitForEditor(page)
  const codeLensLink = page.locator('.codelens-decoration a', {
    hasText: 'Edit Parts',
  })
  await expect(codeLensLink).toBeVisible({ timeout: 15_000 })
  await codeLensLink.click()
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })
}

async function openSoundfontSearchModal(page: import('@playwright/test').Page) {
  await openEditPartsModal(page)
  await page.getByTestId('soundfont-select-M').click()
  const searchModal = page
    .getByRole('dialog')
    .filter({ hasText: 'Select soundfont' })
  await expect(searchModal).toBeVisible()
  return searchModal
}

function soundfontSearchModal(page: import('@playwright/test').Page) {
  return page.getByRole('dialog').filter({ hasText: 'Select soundfont' })
}

Given('the soundfont-search-modal test fixture is loaded', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
})

When(
  'I open the soundfont search modal for part {string}',
  async ({ page }, _abbreviation: string) => {
    await openSoundfontSearchModal(page)
  },
)

When(
  'I fill the soundfont search box with {string}',
  async ({ page }, query: string) => {
    await soundfontSearchModal(page).getByPlaceholder('Search...').fill(query)
  },
)

Then(
  'the soundfont search modal shows a button {string}',
  async ({ page }, name: string) => {
    await expect(
      soundfontSearchModal(page).getByRole('button', { name, exact: true }),
    ).toBeVisible()
  },
)

Then(
  'the soundfont search modal has no button {string}',
  async ({ page }, name: string) => {
    await expect(
      soundfontSearchModal(page).getByRole('button', { name, exact: true }),
    ).toHaveCount(0)
  },
)

When(
  'I click the {string} tag on the {string} row in the soundfont search modal',
  async ({ page }, tag: string, instrumentLabel: string) => {
    const row = soundfontSearchModal(page)
      .getByRole('button', { name: instrumentLabel, exact: true })
      .locator('xpath=ancestor::div[1]')
    await row.getByRole('button', { name: tag, exact: true }).click()
  },
)

Then(
  'the {string} tag on the {string} row is highlighted as active',
  async ({ page }, tag: string, instrumentLabel: string) => {
    const row = soundfontSearchModal(page)
      .getByRole('button', { name: instrumentLabel, exact: true })
      .locator('xpath=ancestor::div[1]')
    const tagButton = row.getByRole('button', { name: tag, exact: true })
    await expect(tagButton).toHaveCSS('color', 'rgb(29, 78, 216)')
  },
)

Then(
  'the {string} tag on the {string} row is not highlighted as active',
  async ({ page }, tag: string, instrumentLabel: string) => {
    const row = soundfontSearchModal(page)
      .getByRole('button', { name: instrumentLabel, exact: true })
      .locator('xpath=ancestor::div[1]')
    const tagButton = row.getByRole('button', { name: tag, exact: true })
    await expect(tagButton).not.toHaveCSS('color', 'rgb(29, 78, 216)')
  },
)

Given(
  'the scenario timeout is extended to 60 seconds, as seen in soundfont search modal',
  async () => {
    test.setTimeout(60_000)
  },
)

When(
  'I retry clicking the Preview instrument button for {string} in the soundfont search modal until it pauses',
  async ({ page }, instrumentLabel: string) => {
    const row = soundfontSearchModal(page)
      .getByRole('button', { name: instrumentLabel, exact: true })
      .locator('xpath=ancestor::div[1]')
    const previewButton = row.getByTitle('Preview instrument')

    // The button is a silent no-op until the soundfont finishes loading, so
    // retry clicking until the title actually flips to "Pause preview".
    await expect(async () => {
      await previewButton.click()
      await expect(row.getByTitle('Pause preview')).toBeVisible({
        timeout: 1_000,
      })
    }).toPass({ timeout: 30_000 })
  },
)

Then(
  'clicking Pause preview for {string} in the soundfont search modal returns it to Preview instrument',
  async ({ page }, instrumentLabel: string) => {
    const row = soundfontSearchModal(page)
      .getByRole('button', { name: instrumentLabel, exact: true })
      .locator('xpath=ancestor::div[1]')
    await row.getByTitle('Pause preview').click()
    await expect(row.getByTitle('Preview instrument')).toBeVisible({
      timeout: 5_000,
    })
  },
)
