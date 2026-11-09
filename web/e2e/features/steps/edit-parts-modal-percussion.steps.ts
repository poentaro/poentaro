import { expect, test } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M], "Chords" [C]: chords],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    [Mv1] twin- kle twin- kle',
  '    [C] 1 - - -',
  '    """,',
].join('\n')

const PERCUSSION_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M], "Drums" [D]: percussion],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    [Mv1] twin- kle twin- kle',
  '    """,',
].join('\n')

async function loadSource(
  page: import('@playwright/test').Page,
  source: string = SOURCE,
) {
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

async function getEditorSource(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const editors = (
      window as unknown as {
        monaco?: {
          editor?: {
            getEditors?: () => { getValue?: () => string }[]
          }
        }
      }
    ).monaco?.editor?.getEditors?.()
    return editors?.[0]?.getValue?.() ?? ''
  })
}

async function getStoredSource(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const raw = localStorage.getItem('poentaro:files:v1')
    if (!raw) return ''
    const store = JSON.parse(raw) as {
      active: string
      userFiles: Record<string, string>
    }
    return store.userFiles[store.active] ?? ''
  })
}

function percussionSearchModal(page: import('@playwright/test').Page) {
  return page.getByRole('dialog').filter({ hasText: 'Select percussion sound' })
}

Given(
  'the edit-parts-modal-percussion test fixture is loaded',
  async ({ page }) => {
    await loadSource(page)
    await page.goto('/')
  },
)

Given(
  'the edit-parts-modal-percussion test fixture with a percussion part is loaded',
  async ({ page }) => {
    await loadSource(page, PERCUSSION_SOURCE)
    await page.goto('/')
  },
)

When('I open the Edit Parts modal', async ({ page }) => {
  await openEditPartsModal(page)
})

Then(
  'the mode select for part {string} shows {string}',
  async ({ page }, abbreviation: string, mode: string) => {
    const modeSelect = page.getByTestId(`mode-select-${abbreviation}`)
    await expect(modeSelect).toContainText(mode)
  },
)

When(
  'I change the mode select for part {string} to {string}',
  async ({ page }, abbreviation: string, mode: string) => {
    const modeSelect = page.getByTestId(`mode-select-${abbreviation}`)
    await modeSelect.click()
    await page.getByRole('option', { name: mode, exact: true }).click()
  },
)

Then(
  'the editor source and stored source both contain {string}, as seen in edit parts modal percussion',
  async ({ page }, expectedLine: string) => {
    await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
    await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
  },
)

When(
  'I click the soundfont select for part {string}',
  async ({ page }, abbreviation: string) => {
    await page.getByTestId(`soundfont-select-${abbreviation}`).click()
  },
)

Then('the percussion sound search modal is visible', async ({ page }) => {
  await expect(percussionSearchModal(page)).toBeVisible()
})

Then(
  'the percussion sound search modal shows a button {string}',
  async ({ page }, name: string) => {
    await expect(
      percussionSearchModal(page).getByRole('button', { name, exact: true }),
    ).toBeVisible()
  },
)

Then(
  'the percussion sound search modal has no button {string}',
  async ({ page }, name: string) => {
    await expect(
      percussionSearchModal(page).getByRole('button', { name, exact: true }),
    ).toHaveCount(0)
  },
)

When(
  'I click the {string} button in the percussion sound search modal',
  async ({ page }, name: string) => {
    await percussionSearchModal(page)
      .getByRole('button', { name, exact: true })
      .click()
  },
)

Then(
  'the soundfont select for part {string} shows {string}',
  async ({ page }, abbreviation: string, text: string) => {
    const soundfontSelect = page.getByTestId(`soundfont-select-${abbreviation}`)
    await expect(soundfontSelect).toContainText(text)
  },
)

Given('the scenario timeout is extended to 60 seconds', async () => {
  test.setTimeout(60_000)
})

When(
  'I retry clicking the Preview instrument button for {string} in the percussion search modal until it pauses',
  async ({ page }, name: string) => {
    const snareRow = percussionSearchModal(page)
      .getByRole('button', { name, exact: true })
      .locator('xpath=ancestor::div[1]')
    const previewButton = snareRow.getByTitle('Preview instrument')

    // The button is a silent no-op until the soundfont finishes loading, so
    // retry clicking until the title actually flips to "Pause preview".
    await expect(async () => {
      await previewButton.click()
      await expect(snareRow.getByTitle('Pause preview')).toBeVisible({
        timeout: 1_000,
      })
    }).toPass({ timeout: 30_000 })
  },
)

Then(
  'clicking Pause preview for {string} in the percussion search modal returns it to Preview instrument',
  async ({ page }, name: string) => {
    const snareRow = percussionSearchModal(page)
      .getByRole('button', { name, exact: true })
      .locator('xpath=ancestor::div[1]')
    await snareRow.getByTitle('Pause preview').click()
    await expect(snareRow.getByTitle('Preview instrument')).toBeVisible({
      timeout: 5_000,
    })
  },
)
