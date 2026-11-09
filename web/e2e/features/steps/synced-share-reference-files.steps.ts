import { expect } from '@playwright/test'
import {
  fetchSyncedDoc,
  restoreCloudFile,
  seedCloudFile,
  trashCloudFile,
} from '../../cloudFileHelpers'
import { DEFAULT_E2E_EMAIL } from '../../e2eAccounts.ts'
import {
  fileSwitcherTrigger,
  fileTabByExactName,
  openFileList,
} from '../../fileSwitcherHelpers'
import { BeforeScenario, Given, Then, When } from './fixtures'
import { syncedShareButtonState as sharedState } from './synced-share-button-state'

// Steps for synced-share-reference-files.feature. The share/viewer steps
// themselves are reused from synced-share-button.steps.ts; this file only
// adds the second cloud file and the direct worker-side checks.

/** The second cloud file "another cloud file titled ... is seeded" creates,
 * under a per-scenario unique name (extension stripped) -- see
 * `seedSyncedCloudFile` for why names must not collide across scenarios. */
let otherFileName: string | undefined

BeforeScenario(async () => {
  otherFileName = undefined
})

function scoreTitled(title: string): string {
  return [
    `metadata { title "${title}" },`,
    '',
    'parts ["Melody": notes],',
    '',
    'score """',
    '    (time=4/4 key=C4 bpm=120)',
    '    5 6 7 1',
    '    """,',
  ].join('\n')
}

Given(
  'another cloud file titled {string} is seeded',
  async ({}, title: string) => {
    otherFileName = `synced-other-${crypto.randomUUID().slice(0, 8)}`
    await seedCloudFile(
      DEFAULT_E2E_EMAIL,
      `${otherFileName}.poentaro`,
      scoreTitled(title),
    )
  },
)

When('the owner switches to the other cloud file', async ({ page }) => {
  if (!otherFileName) throw new Error('no other cloud file was seeded')
  // The share modal's overlay blocks the header while it's open.
  const modal = page.getByTestId('share-modal')
  if (await modal.count()) {
    await page.keyboard.press('Escape')
    await modal.waitFor({ state: 'hidden' })
  }
  await openFileList(page)
  const tab = fileTabByExactName(page, otherFileName)
  await tab.waitFor({ timeout: 15_000 })
  await tab.click()
  await expect(fileSwitcherTrigger(page)).toContainText(otherFileName)
  await expect(page.locator('.monaco-editor .view-lines')).toContainText(
    'Other Score',
  )
})

// Replaces the whole editor model, same as "the owner edits the synced
// score's title to ..." -- `setValue` fires the same change event a real
// edit does, so the edit goes through the ordinary autosave path.
When(
  "the owner edits the other file's title to {string}",
  async ({ page }, title: string) => {
    await page.evaluate((value) => {
      const editor = (
        window as unknown as {
          monaco?: typeof import('monaco-editor')
        }
      ).monaco?.editor.getEditors()[0]
      editor?.getModel()?.setValue(value)
    }, scoreTitled(title))
  },
)

When('the shared cloud file is moved to the bin', async () => {
  if (!sharedState.ownerFileId) throw new Error('no synced file was seeded')
  await trashCloudFile(DEFAULT_E2E_EMAIL, sharedState.ownerFileId)
})

When('the shared cloud file is restored from the bin', async () => {
  if (!sharedState.ownerFileId || !sharedState.ownerFileName) {
    throw new Error('no synced file was seeded')
  }
  await restoreCloudFile(
    DEFAULT_E2E_EMAIL,
    sharedState.ownerFileId,
    sharedState.ownerFileName,
  )
})

// Reads the worker's anonymous `GET /shares/{share_id}` directly rather
// than through a viewer page -- the viewer UI hides an ended share's score
// either way, but the point here is that the content never leaves the
// server at all once the owner stops sharing.
Then(
  'the raw share document for the copied link has an empty filename and content',
  async () => {
    const link = sharedState.originalSyncedLink
    if (!link) throw new Error('originalSyncedLink was not captured yet')
    const shareId = new URL(link).searchParams.get('s')?.slice(0, 11)
    if (!shareId) throw new Error(`not a synced share link: ${link}`)
    const doc = await fetchSyncedDoc(shareId)
    expect(doc.filename).toBe('')
    expect(doc.content).toBe('')
  },
)
