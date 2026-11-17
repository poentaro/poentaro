import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Bass" [B]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 2 3 4',
  '    [B] 5 6 7 1',
  '    """,',
].join('\n')

const FOLLOW_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Chords" [C]: follow [M]],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    [M] 1 2 3 4',
  '    """,',
].join('\n')

async function loadSource(page: import('@playwright/test').Page, src: string) {
  await page.addInitScript((source) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'test.poentaro',
        userFiles: { 'test.poentaro': source },
        bin: {},
        fileIds: { 'test.poentaro': crypto.randomUUID() },
      }),
    )
  }, src)
}

async function waitForEditor(page: import('@playwright/test').Page) {
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 30_000 })
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

Given(
  'the melody-bass fixture is loaded and the app has navigated home',
  async ({ page }) => {
    await loadSource(page, SOURCE)
    await page.goto('/')
  },
)

Given(
  'the melody-follow-chords fixture is loaded and the app has navigated home',
  async ({ page }) => {
    await loadSource(page, FOLLOW_SOURCE)
    await page.goto('/')
  },
)

Given(
  'I open the Edit Parts modal, as seen in shift part octave toolbar',
  async ({ page }) => {
    await waitForEditor(page)
    const codeLensLink = page.locator('.codelens-decoration a', {
      hasText: 'Edit Parts',
    })
    await expect(codeLensLink).toBeVisible({ timeout: 15_000 })
    await codeLensLink.click()
    await page.getByTestId('edit-parts-modal').waitFor({ state: 'visible' })
  },
)

When(
  'I click the notation octave-up control for part {string}',
  async ({ page }, part: string) => {
    await page.getByTestId(`notation-octave-up-${part}`).click()
  },
)

Then('the editor source contains {string}', async ({ page }, text: string) => {
  await expect.poll(getEditorSource.bind(null, page)).toContain(text)
})

Then('the stored source contains {string}', async ({ page }, text: string) => {
  await expect.poll(getStoredSource.bind(null, page)).toContain(text)
})

Then(
  'the editor source still contains {string}',
  async ({ page }, text: string) => {
    const source = await getEditorSource(page)
    expect(source).toContain(text)
  },
)

Then(
  'the notation octave-up control for part {string} is visible',
  async ({ page }, part: string) => {
    await expect(page.getByTestId(`notation-octave-up-${part}`)).toBeVisible()
  },
)

Then(
  'there is no notation octave-up control for part {string}',
  async ({ page }, part: string) => {
    await expect(page.getByTestId(`notation-octave-up-${part}`)).toHaveCount(0)
  },
)
