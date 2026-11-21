import { expect, type Page } from '@playwright/test'
import { appCallsOf, routeAppCall } from '../../appCalls'
import { seedCloudFile } from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileActions,
  openFileList,
} from '../../fileSwitcherHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { BeforeScenario, Given, Then, When } from './fixtures'

const displayName = (name: string) => name.replace(/\.poentaro$/, '')

let seededIdByName = new Map<string, string>()
let readIds: string[] = []
let recorded = new WeakSet<Page>()

BeforeScenario(() => {
  seededIdByName = new Map()
  readIds = []
  recorded = new WeakSet()
})

function idOf(name: string): string {
  const id = seededIdByName.get(name)
  if (!id) throw new Error(`"${name}" was not seeded in this scenario`)
  return id
}

/** Records the file id of every `read file` call the page makes. */
function recordReadFiles(page: Page): void {
  if (recorded.has(page)) return
  recorded.add(page)
  page.on('request', (request) => {
    void appCallsOf(request).then((calls) => {
      for (const call of calls) {
        if (call.method === 'read file _')
          readIds.push(String(call.arguments[0]))
      }
    })
  })
}

async function loadWithFileOpen(page: Page, name: string): Promise<void> {
  recordReadFiles(page)
  await page.addInitScript(() => {
    localStorage.setItem(
      'poentaro:storage-backend:v1',
      JSON.stringify({ backend: 'cloud' }),
    )
  })
  await page.goto(`/?file=${encodeURIComponent(name)}`)
}

Given(
  'a cloud file named {string} is seeded for the signed-in account with the text {string}',
  async ({}, name: string, text: string) => {
    const file = await seedCloudFile(currentSignedInEmail(), name, text)
    seededIdByName.set(name, file.id)
  },
)

Given('the app loads with {string} open', async ({ page }, name: string) => {
  await loadWithFileOpen(page, name)
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
})

When('the app reloads with {string} open', async ({ page }, name: string) => {
  readIds = []
  await page.goto(`/?file=${encodeURIComponent(name)}`)
})

When(
  'the user opens {string} from the file list',
  async ({ page }, name: string) => {
    await openFileList(page)
    await fileTabByExactName(page, displayName(name)).click()
  },
)

Given(
  'reading the text of {string} is delayed by {int} seconds',
  async ({ page }, name: string, seconds: number) => {
    const id = idOf(name)
    await routeAppCall(page, 'read file _', async (route, call) => {
      if (call.arguments[0] === id) {
        await new Promise((resolve) => setTimeout(resolve, seconds * 1000))
      }
      await route.fallback()
    })
  },
)

Given(
  '{string} is changed on the server to {string}',
  async ({}, name: string, text: string) => {
    await seedCloudFile(currentSignedInEmail(), name, text)
  },
)

Then(
  'the file list shows {string}, {string} and {string}',
  async ({ page }, first: string, second: string, third: string) => {
    await openFileList(page)
    for (const name of [first, second, third]) {
      await expect(fileTabByExactName(page, name)).toBeVisible()
    }
  },
)

Then(
  'the server was asked for the text of {string} only',
  async ({}, name: string) => {
    await expect.poll(() => [...new Set(readIds)]).toEqual([idOf(name)])
  },
)

Then(
  'the server was asked for the text of {string} once',
  async ({}, name: string) => {
    const id = idOf(name)
    await expect
      .poll(() => readIds.filter((read) => read === id))
      .toHaveLength(1)
  },
)

Then(
  'the server was not asked for the text of {string} after the reload',
  async ({ page }, name: string) => {
    await expect(page.locator('.monaco-editor .view-lines')).toBeVisible()
    expect(readIds).not.toContain(idOf(name))
  },
)

Then('the file switcher shows a loading spinner', async ({ page }) => {
  await expect(
    fileSwitcherTrigger(page).locator('.file-tab-bar-spinner'),
  ).toBeVisible()
})

When('the text of {string} arrives', async ({ page }, _name: string) => {
  await expect(
    fileSwitcherTrigger(page).locator('.file-tab-bar-spinner'),
  ).toHaveCount(0, { timeout: 10_000 })
})

const editorReadOnly = (page: Page) =>
  page.evaluate(
    () =>
      window.monaco?.editor.getEditors()[0]?.getRawOptions().readOnly === true,
  )

Then('the editor is read-only', async ({ page }) => {
  await expect.poll(() => editorReadOnly(page)).toBe(true)
})

Then('the editor is editable', async ({ page }) => {
  await expect.poll(() => editorReadOnly(page)).toBe(false)
})

Given(
  'the user has opened {string} from the file list',
  async ({ page }, name: string) => {
    await openFileList(page)
    await fileTabByExactName(page, displayName(name)).click()
    // The URL names the file once its text has been read and it is open.
    await expect(page).toHaveURL(new RegExp(encodeURIComponent(name)))
  },
)

When('the user duplicates the open file', async ({ page }) => {
  await openFileActions(page)
  await page.getByRole('menuitem', { name: 'Duplicate' }).click()
})
