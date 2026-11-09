import { expect, type Page } from '@playwright/test'
import { NEW_ORIGIN, OLD_ORIGIN } from '../../originMigrationHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '1 2 3 4',
  '""",',
].join('\n')

/** Saves a local file store holding `name` on `origin`, under the
 * `localStorage` key `key`, before any of that origin's page scripts run. */
async function seedLocalFile(
  page: Page,
  origin: string,
  name: string,
  key: string,
) {
  await page.addInitScript(
    ({ targetOrigin, fileName, source, storageKey }) => {
      if (window.location.origin !== targetOrigin) return
      localStorage.setItem(
        storageKey,
        JSON.stringify({
          active: fileName,
          userFiles: { [fileName]: source },
          bin: {},
          fileIds: { [fileName]: crypto.randomUUID() },
        }),
      )
    },
    { targetOrigin: origin, fileName: name, source: SOURCE, storageKey: key },
  )
}

// The old origin only ever ran the app before the rename to Poentaro, so it
// saved under the legacy key.
Given(
  'the old origin has a saved local file {string}',
  async ({ page }, name: string) => {
    await seedLocalFile(page, OLD_ORIGIN, name, 'jianpu:files:v1')
  },
)

Given(
  'the new origin has a saved local file {string}',
  async ({ page }, name: string) => {
    await seedLocalFile(page, NEW_ORIGIN, name, 'poentaro:files:v1')
  },
)

Given(
  'the new origin has a local file {string} saved before the rename',
  async ({ page }, name: string) => {
    await seedLocalFile(page, NEW_ORIGIN, name, 'jianpu:files:v1')
  },
)

When('I open the new origin', async ({ page }) => {
  await page.goto(NEW_ORIGIN)
  await page.waitForSelector('.monaco-editor .view-lines', {
    timeout: 30_000,
  })
})

Then(
  'the new origin has no data saved under a legacy name',
  async ({ page }) => {
    const legacyKeys = await page.evaluate(() =>
      Object.keys(localStorage).filter((key) => key.startsWith('jianpu:')),
    )
    expect(legacyKeys).toEqual([])
  },
)

When('I open the old origin at {string}', async ({ page }, path: string) => {
  await page.goto(`${OLD_ORIGIN}${path}`)
})

Then('I land on the new origin at {string}', async ({ page }, path: string) => {
  // The handoff is claimed (and `migrate` stripped from the URL) before
  // the app renders, so wait for the editor before reading the URL.
  await page.waitForURL((url) => url.origin === NEW_ORIGIN, {
    timeout: 15_000,
  })
  await page.waitForSelector('.monaco-editor .view-lines', {
    timeout: 30_000,
  })
  // The app adds its own `file` param once loaded, so compare only what
  // the old link carried.
  const expected = new URL(path, NEW_ORIGIN)
  const landed = new URL(page.url())
  expect(landed.pathname).toBe(expected.pathname)
  expect(landed.hash).toBe(expected.hash)
  expect(landed.searchParams.has('migrate')).toBe(false)
  for (const [name, value] of expected.searchParams) {
    expect(landed.searchParams.get(name)).toBe(value)
  }
})

Then(
  'the new origin has the saved local files {string}',
  async ({ page }, names: string) => {
    await expect
      .poll(() =>
        page.evaluate(() => {
          const raw = localStorage.getItem('poentaro:files:v1')
          const store = raw
            ? (JSON.parse(raw) as { userFiles: Record<string, string> })
            : { userFiles: {} }
          return Object.keys(store.userFiles).sort().join(', ')
        }),
      )
      .toBe(names)
  },
)
