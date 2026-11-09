import { expect, type Page } from '@playwright/test'
import createClient from 'openapi-fetch'
import type { paths } from '../../../src/generated/live-share-worker/schema'
import {
  CLOUD_WORKER_ORIGIN,
  DEFAULT_CLOUD_FILE_CONTENT,
  seedCloudFile,
  seedTokenFor,
} from '../../cloudFileHelpers'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  focusEditor,
  openFileList,
} from '../../fileSwitcherHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { gotoCloudApp } from './cloud-account-helpers'
import { installContentSaveTracking } from './cloud-content-save-tracking'
import { Given, Then, When } from './fixtures'

const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })

const EDITOR_LINES = '.monaco-editor .view-lines'
const displayName = (fileName: string) => fileName.replace(/\.poentaro$/, '')

/** The default seed with its notes line (`1 2 3 4`) replaced by `lastLine`. */
const withLastLine = (lastLine: string) =>
  DEFAULT_CLOUD_FILE_CONTENT.replace('1 2 3 4', lastLine)

// Two edits far enough apart for a line-based three-way merge to keep both:
// the first line (a comment heading) and the last line (the notes).
const TWO_LINE_BASE = `# heading
${DEFAULT_CLOUD_FILE_CONTENT}`
const FIRST_LINE_EDIT = '# heading edited here'
const SECOND_LINE_EDIT = '1 2 3 4 7'
const RESOLVED_TEXT = '1 2 3 9 5'

/** Changes the server's copy of `fileName` straight on the worker, using the
 * revision the server currently holds so the write is accepted. */
async function changeServerCopy(
  fileName: string,
  content: string,
): Promise<void> {
  const identityToken = await seedTokenFor(currentSignedInEmail())
  const { data: listed } = await worker.POST('/files/list', {
    body: { identityToken },
  })
  const file = listed?.files.find(
    (candidate) => candidate.name === fileName && candidate.trashedAt === null,
  )
  if (!file) throw new Error(`no server file named ${fileName}`)
  const { error } = await worker.POST('/files/{id}/content', {
    params: { path: { id: file.id } },
    body: { identityToken, content, expectedRevision: file.revision },
  })
  if (error) throw new Error(`server edit failed: ${JSON.stringify(error)}`)
}

/** The server's current content of `fileName` ('' when absent). */
async function serverContentOf(fileName: string): Promise<string> {
  const identityToken = await seedTokenFor(currentSignedInEmail())
  const { data } = await worker.POST('/files/list', { body: { identityToken } })
  return (
    data?.files.find(
      (file) => file.name === fileName && file.trashedAt === null,
    )?.content ?? ''
  )
}

/** Loads the app (installing save tracking first) and opens `name`. */
async function openFile(page: Page, name: string): Promise<void> {
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
}

Given(
  'a cloud file named {string} is seeded for the signed-in account with two separate lines',
  async ({}, name: string) => {
    await seedCloudFile(currentSignedInEmail(), name, TWO_LINE_BASE)
  },
)

Given('I open {string}', async ({ page }, name: string) => {
  await openFile(page, name)
})

Given(
  'the server copy of {string} is changed to {string}',
  async ({}, fileName: string, lastLine: string) => {
    await changeServerCopy(fileName, withLastLine(lastLine))
  },
)

Given(
  'the server copy of {string} has its second line changed',
  async ({}, fileName: string) => {
    await changeServerCopy(
      fileName,
      TWO_LINE_BASE.replace('1 2 3 4', SECOND_LINE_EDIT),
    )
  },
)

When('the tab becomes visible again', async ({ page }) => {
  await page.evaluate(() =>
    document.dispatchEvent(new Event('visibilitychange')),
  )
})

When('I change the first line of the open file', async ({ page }) => {
  await focusEditor(page)
  await page.evaluate((replacement) => {
    const model = window.monaco?.editor.getEditors()[0]?.getModel()
    if (!model) throw new Error('no editor model')
    model.pushEditOperations(
      [],
      [
        {
          range: model
            .getFullModelRange()
            .collapseToStart()
            .setEndPosition(1, model.getLineMaxColumn(1)),
          text: replacement,
        },
      ],
      () => null,
    )
  }, FIRST_LINE_EDIT)
  await expect(page.locator(EDITOR_LINES)).toContainText(FIRST_LINE_EDIT)
  await page.keyboard.press('ControlOrMeta+s')
})

Then(
  'the content save lands for {string} containing both changes',
  async ({}, fileName: string) => {
    await expect
      .poll(() => serverContentOf(fileName), { timeout: 20_000 })
      .toContain(FIRST_LINE_EDIT)
    await expect
      .poll(() => serverContentOf(fileName))
      .toContain(SECOND_LINE_EDIT)
  },
)

Then('a notice says {string}', async ({ page }, text: string) => {
  await expect(page.getByTestId('sync-notice')).toHaveText(text, {
    timeout: 15_000,
  })
})

Then(
  'the merge editor shows the server version {string} on the left',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('merge-editor')).toBeVisible({
      timeout: 15_000,
    })
    await expect
      .poll(() =>
        page.evaluate(
          () =>
            window.monaco?.editor
              .getDiffEditors()[0]
              ?.getOriginalEditor()
              .getValue() ?? '',
        ),
      )
      .toContain(text)
  },
)

const mergeEditorButton = (page: Page, name: string) =>
  page.getByTestId('merge-editor').getByRole('button', { name, exact: true })

Then('the Save button in the merge editor is disabled', async ({ page }) => {
  await expect(mergeEditorButton(page, 'Save')).toBeDisabled()
})

When('I resolve the conflict markers in the merge editor', async ({ page }) => {
  await page.evaluate((resolved) => {
    const editor = window.monaco?.editor
      .getDiffEditors()[0]
      ?.getModifiedEditor()
    const model = editor?.getModel()
    if (!model) throw new Error('no merge editor model')
    model.setValue(
      model
        .getValue()
        .replace(/<<<<<<<[^\n]*\n[\s\S]*?>>>>>>>[^\n]*/, resolved),
    )
  }, RESOLVED_TEXT)
  await expect(mergeEditorButton(page, 'Save')).toBeEnabled()
})

When(
  'I click {string} in the merge editor',
  async ({ page }, label: string) => {
    await mergeEditorButton(page, label).click()
  },
)

Then(
  'the content save lands for {string} containing the resolved text',
  async ({}, fileName: string) => {
    await expect
      .poll(() => serverContentOf(fileName), { timeout: 20_000 })
      .toContain(RESOLVED_TEXT)
  },
)

Then(
  'a file whose name starts with {string} is listed in the file switcher',
  async ({ page }, prefix: string) => {
    const panel = page.getByTestId('outbox-history')
    if (await panel.isVisible()) {
      await panel.getByRole('button', { name: 'Close', exact: true }).click()
      await expect(panel).toBeHidden()
    }
    await openFileList(page)
    await expect(
      page
        .getByTestId('file-browser')
        .getByRole('button', { name: prefix })
        .first(),
    ).toBeVisible({ timeout: 15_000 })
  },
)

Then(
  'the editor of {string} contains {string}',
  async ({ page }, fileName: string, text: string) => {
    await openFileList(page)
    const tab = fileTabByExactName(page, displayName(fileName))
    await tab.waitFor({ timeout: 15_000 })
    await tab.click()
    await expect(page.locator(EDITOR_LINES)).toContainText(text, {
      timeout: 10_000,
    })
  },
)
