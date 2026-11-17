import { expect } from '@playwright/test'
import { routeAppCall } from '../../appCalls'
import { seedTokenFor } from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { appClient, valueOrFail } from '../../protocolClients'
import { currentSignedInEmail } from './cloud-account.steps'
import { gotoCloudApp, lastSeededSessionToken } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

// Covers session-ended.feature: a session that ends while the page still
// holds it signs the browser out at the first refused request.

Given(
  'I open {string} from the cloud file list',
  async ({ page }, name: string) => {
    await gotoCloudApp(page)
    await openFileList(page)
    const tab = fileTabByExactName(page, name)
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(fileSwitcherTrigger(page)).toContainText(name)
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Given(
  'the worker refuses the share overview as unauthorized',
  async ({ page }) => {
    await routeAppCall(page, 'list shared links', (route) =>
      route.fulfill({ status: 401, body: 'Your session has ended.' }),
    )
  },
)

When('the app loads with cloud storage', async ({ page }) => {
  await gotoCloudApp(page)
})

// Ends the page's session the way a sign-out on another device would: the
// page is not told, it only finds out from its next request.
When('the session is ended on the worker', async () => {
  await valueOrFail(
    'sign-out',
    (await appClient(lastSeededSessionToken())).signOut(),
  )
})

// Cmd/Ctrl+S hands the edit to the outbox at once instead of waiting out
// the autosave debounce.
When(
  'I append {string} to the editor and save',
  async ({ page }, text: string) => {
    await typeAtEditorEnd(page, text)
    await page.keyboard.press('ControlOrMeta+s')
  },
)

// Read straight from the worker: the page also sends the save its session
// had refused, so only the stored content shows it was accepted.
Then(
  'the cloud file {string} contains {string}',
  async ({}, name: string, text: string) => {
    const identityToken = await seedTokenFor(currentSignedInEmail())
    await expect
      .poll(
        async () => {
          const client = await appClient(identityToken)
          const files = await valueOrFail('list files', client.listFiles())
          const file = files.find((file) => file.path === name)
          return file
            ? (await valueOrFail('read file', client.readFile(file.id))).content
            : undefined
        },
        { timeout: 15_000 },
      )
      .toEqual(expect.stringContaining(text))
  },
)
