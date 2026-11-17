import { expect } from '@playwright/test'
import {
  allowContentSaves,
  expectSaveBadge,
  fastForwardPastOutboxRetryDelay,
  interceptContentSaves,
  saveStatusBadge,
} from '../../cloudOutboxHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
  typeAtEditorEnd,
} from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import {
  currentContentSaveTracker,
  installContentSaveTracking,
} from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

/** Resolves a file id to its name through the scenario's save tracker, which
 * is installed (before the app loads) by the "I open and edit" steps. */
const nameForId = (fileId: string) =>
  currentContentSaveTracker().nameForId.get(fileId)

Given(
  'every content-save request will be aborted as a network failure',
  async ({ page }) => {
    await interceptContentSaves(page, { fault: { kind: 'abort' } }, nameForId)
  },
)

Given(
  /^the first content-save request will fail with a (5\d\d) response$/,
  async ({ page }, status: string) => {
    await interceptContentSaves(
      page,
      { fault: { kind: 'status', status: Number(status) }, times: 1 },
      nameForId,
    )
  },
)

Given(
  'every content-save request for {string} will fail with a {int} response',
  async ({ page }, fileName: string, status: number) => {
    await interceptContentSaves(
      page,
      { fault: { kind: 'status', status }, fileName },
      nameForId,
    )
  },
)

Given(
  'the first content-save response will be dropped after the server applies it',
  async ({ page }) => {
    await interceptContentSaves(
      page,
      { fault: { kind: 'drop-response' }, times: 1 },
      nameForId,
    )
  },
)

/** Opens `name` from the file list (loading the app first if this is the
 * scenario's first open) and appends `suffix`, then waits for the edit. */
async function openAndEdit(
  page: Parameters<typeof gotoCloudApp>[0],
  name: string,
  suffix: string,
): Promise<void> {
  if (page.url() === 'about:blank') {
    installContentSaveTracking(page)
    await gotoCloudApp(page)
  }
  await openFileList(page)
  const tab = fileTabByExactName(page, name)
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await expect(fileSwitcherTrigger(page)).toContainText(name)
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await typeAtEditorEnd(page, suffix)
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    `1 2 3 4${suffix}`,
    { timeout: 10_000 },
  )
}

// Without a fake clock the 20s autosave debounce would make these scenarios
// slow, so the edit is flushed with Cmd/Ctrl+S, which hands it to the outbox
// straight away.
Given(
  'I open and edit {string} with suffix {string}',
  async ({ page }, name: string, suffix: string) => {
    await openAndEdit(page, name, suffix)
    await page.keyboard.press('ControlOrMeta+s')
  },
)

When(
  'I fast-forward the clock past the outbox retry delay',
  async ({ page }) => {
    await fastForwardPastOutboxRetryDelay(page)
  },
)

When('content-save requests are allowed again', async ({ page }) => {
  allowContentSaves(page)
})

Then('the save badge shows {string}', async ({ page }, text: string) => {
  await expectSaveBadge(page, text)
})

Then('no conflict is shown', async ({ page }) => {
  await expect(page.getByTestId('conflict-banner')).toHaveCount(0)
  await expect(saveStatusBadge(page)).not.toContainText('attention')
})
