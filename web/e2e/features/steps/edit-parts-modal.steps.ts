import { expect } from '@playwright/test'
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

const FOLLOW_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M], "Chords" [C]: follow [M]],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 1 5 5',
  '    [Mv1] twin- kle twin- kle',
  '    """,',
].join('\n')

const MULTI_FOLLOW_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts [',
  '    "Melody" [M]: notes,',
  '    "Melody lyrics" [Mv1]: lyrics [M],',
  '    "Harmony" [H]: notes,',
  '    "Chords" [C]: follow [M],',
  '],',
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

async function selectSoundfont(
  page: import('@playwright/test').Page,
  abbreviation: string,
  instrumentLabel: string,
) {
  await page.getByTestId(`soundfont-select-${abbreviation}`).click()
  const searchModal = page
    .getByRole('dialog')
    .filter({ hasText: 'Select soundfont' })
  await expect(searchModal).toBeVisible()
  await searchModal
    .getByRole('button', { name: instrumentLabel, exact: true })
    .click()
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

type Selection = {
  startLineNumber: number
  startColumn: number
  endLineNumber: number
  endColumn: number
}

function getSelection(page: import('@playwright/test').Page) {
  return page.evaluate(() => {
    const editors = (
      window as unknown as {
        monaco?: {
          editor?: {
            getEditors?: () => { getSelection?: () => Selection | null }[]
          }
        }
      }
    ).monaco?.editor?.getEditors?.()
    return editors?.[0]?.getSelection?.() ?? null
  })
}

function editPartsModal(page: import('@playwright/test').Page) {
  return page.getByTestId('edit-parts-modal')
}

Given('the edit-parts-modal test fixture is loaded', async ({ page }) => {
  await loadSource(page)
  await page.goto('/')
})

Given(
  'the edit-parts-modal test fixture with a follow part is loaded',
  async ({ page }) => {
    await loadSource(page, FOLLOW_SOURCE)
    await page.goto('/')
  },
)

Given(
  'the edit-parts-modal test fixture with multiple followable parts is loaded',
  async ({ page }) => {
    await loadSource(page, MULTI_FOLLOW_SOURCE)
    await page.goto('/')
  },
)

When(
  'I open the Edit Parts modal, as seen in edit parts modal',
  async ({ page }) => {
    await openEditPartsModal(page)
  },
)

Then(
  'the edit parts modal contains {string}',
  async ({ page }, text: string) => {
    await expect(editPartsModal(page)).toContainText(text)
  },
)

Then(
  'the mode select for part {string} shows {string}, as seen in edit parts modal',
  async ({ page }, abbreviation: string, mode: string) => {
    const modeSelect = page.getByTestId(`mode-select-${abbreviation}`)
    await expect(modeSelect).toContainText(mode)
  },
)

When(
  'I change the mode select for part {string} to {string}, as seen in edit parts modal',
  async ({ page }, abbreviation: string, mode: string) => {
    const modeSelect = page.getByTestId(`mode-select-${abbreviation}`)
    await modeSelect.click()
    await page.getByRole('option', { name: mode, exact: true }).click()
  },
)

Then(
  'the soundfont select for part {string} shows {string}, as seen in edit parts modal',
  async ({ page }, abbreviation: string, text: string) => {
    const soundfontSelect = page.getByTestId(`soundfont-select-${abbreviation}`)
    await expect(soundfontSelect).toContainText(text)
  },
)

When(
  'I select soundfont {string} for part {string}',
  async ({ page }, instrumentLabel: string, abbreviation: string) => {
    await selectSoundfont(page, abbreviation, instrumentLabel)
  },
)

Then(
  'the octave select for part {string} shows {string}',
  async ({ page }, abbreviation: string, value: string) => {
    const octaveSelect = page.getByTestId(`octave-select-${abbreviation}`)
    await expect(octaveSelect).toContainText(value)
  },
)

When(
  'I change the octave select for part {string} to {string}',
  async ({ page }, abbreviation: string, value: string) => {
    const octaveSelect = page.getByTestId(`octave-select-${abbreviation}`)
    await octaveSelect.click()
    await page.getByRole('option', { name: value, exact: true }).click()
  },
)

Then(
  'the volume value for part {string} shows {string}',
  async ({ page }, abbreviation: string, value: string) => {
    const volumeValue = page.getByTestId(`volume-value-${abbreviation}`)
    await expect(volumeValue).toContainText(value)
  },
)

When(
  'I focus the volume slider for part {string} and press Home',
  async ({ page }, abbreviation: string) => {
    const volumeSlider = page.getByTestId(`volume-slider-${abbreviation}`)
    await volumeSlider.locator('[role="slider"]').focus()
    await page.keyboard.press('Home')
  },
)

Then(
  'the follow target select for part {string} shows {string}',
  async ({ page }, abbreviation: string, value: string) => {
    const followTargetSelect = page.getByTestId(
      `follow-target-select-${abbreviation}`,
    )
    await expect(followTargetSelect).toContainText(value)
  },
)

When(
  'I change the follow target select for part {string} to {string}',
  async ({ page }, abbreviation: string, value: string) => {
    const followTargetSelect = page.getByTestId(
      `follow-target-select-${abbreviation}`,
    )
    await followTargetSelect.click()
    await page.getByRole('option', { name: value, exact: true }).click()
  },
)

When('I close the edit parts modal with Escape', async ({ page }) => {
  await page.keyboard.press('Escape')
  await page.getByTestId('edit-parts-modal').waitFor({ state: 'hidden' })
})

Then(
  'the editor source and stored source both contain {string}, as seen in edit parts modal',
  async ({ page }, expectedLine: string) => {
    await expect.poll(getEditorSource.bind(null, page)).toContain(expectedLine)
    await expect.poll(getStoredSource.bind(null, page)).toContain(expectedLine)
  },
)

// Navigate to SOURCE line 7 ("[M] 1 1 5 5") and select to end of line.
Given(
  'the caret is placed on line 7 of the edit-parts-modal fixture with the line selected',
  async ({ page, focusEditor }) => {
    await waitForEditor(page)
    const codeLensLink = page.locator('.codelens-decoration a', {
      hasText: 'Edit Parts',
    })
    await expect(codeLensLink).toBeVisible({ timeout: 15_000 })

    await focusEditor()
    await page.keyboard.press('Control+g')
    await page.keyboard.type('7')
    await page.keyboard.press('Enter')
    await page.keyboard.press('Shift+End')
    await page.waitForTimeout(300)
  },
)

Then(
  'the editor selection spans line 7 from column 1 to the end of the line',
  async ({ page }) => {
    const selectionBefore = await getSelection(page)
    expect(selectionBefore?.startLineNumber).toBe(7)
    expect(selectionBefore?.startColumn).toBe(1)
    expect(selectionBefore?.endLineNumber).toBe(7)
    expect(selectionBefore?.endColumn).toBeGreaterThan(1)
  },
)

let capturedSelectionBefore: Selection | null = null

Given('I record the current editor selection', async ({ page }) => {
  capturedSelectionBefore = await getSelection(page)
})

// Open the modal and change the soundfont.
When(
  'I open the Edit Parts modal via CodeLens and change soundfont {string} for part {string}',
  async ({ page }, instrumentLabel: string, abbreviation: string) => {
    const codeLensLink = page.locator('.codelens-decoration a', {
      hasText: 'Edit Parts',
    })
    await codeLensLink.click()
    await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })
    await selectSoundfont(page, abbreviation, instrumentLabel)
  },
)

Then(
  'the editor selection is unchanged from before the modal was opened',
  async ({ page }) => {
    const selectionAfter = await getSelection(page)
    expect(selectionAfter).toEqual(capturedSelectionBefore)
  },
)
