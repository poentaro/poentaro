import { expect } from '@playwright/test'
import {
  seedCloudFile,
  trashCloudFile,
  type WorkerSchemas,
  workerRouteGlob,
} from '../../cloudFileHelpers'
import {
  fileTabByExactName,
  openBin,
  openFileList,
} from '../../fileSwitcherHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { gotoCloudApp } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

// Restore-collision coverage for the cloud storage backend, split out of
// `files-cloud-backend.steps.ts` to stay under this repo's 400-line cap.

const COLLISION_ACTIVE_CONTENT = [
  'metadata { title "Existing Active File" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    5 6 7 1',
  '    """,',
].join('\n')

const COLLISION_RESTORED_CONTENT = [
  'metadata { title "Restored From Bin" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

Given(
  'the signed-in account has "original.poentaro" active and "original.poentaro" binned via a separate delete, for a restore collision',
  async ({ page }) => {
    const email = currentSignedInEmail()
    await seedCloudFile(email, 'original.poentaro', COLLISION_ACTIVE_CONTENT)
    // The real schema enforces one globally-unique name per owner across
    // *both* active and trashed files (`idx_files_owner_name` in migration
    // 0003) -- unlike the deleted GitHub backend, where `scores/original.poentaro`
    // and `trash/original.poentaro` were genuinely different paths, a trashed
    // and an active row can never really share a name here. This file is
    // seeded under a distinct real name and trashed for real, then the
    // `/files/list` response the client's own `load()` receives is
    // relabelled below -- reproducing exactly the state `fileStore.ts`'s
    // pure `restoreFile` needs to exercise its own local collision-avoidance
    // (picking "original 2") without lying to the database itself. The
    // restore the client then issues targets this row's real id with the
    // client-computed "original 2.poentaro" name, which lands for real.
    const trashSource = await seedCloudFile(
      email,
      'original-collision-source.poentaro',
      COLLISION_RESTORED_CONTENT,
    )
    await trashCloudFile(email, trashSource.id)

    await page.route(workerRouteGlob('/files/list'), async (route) => {
      const response = await route.fetch()
      const json = (await response.json()) as WorkerSchemas['ListFilesResponse']
      for (const file of json.files) {
        // Only while still trashed -- this route stays registered across
        // the post-restore reload too (`page.route` handlers are
        // page-scoped, not one-shot), and by then the restore has for
        // real renamed this row to "original 2.poentaro" server-side.
        // Relabelling it back to "original.poentaro" unconditionally would
        // clobber that real name every time the list is re-fetched,
        // colliding with the *other*, still-genuinely-active
        // "original.poentaro" row and losing "original 2" from the list
        // entirely.
        if (file.id === trashSource.id && file.trashedAt !== null) {
          file.name = 'original.poentaro'
        }
      }
      await route.fulfill({ response, json })
    })
  },
)

When(
  'the app loads the cloud-backed file list for the collision test',
  async ({ page }) => {
    await gotoCloudApp(page)
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

When('I open the bin', async ({ page }) => {
  await openBin(page)
})

When(
  'I click the restore button for {string}',
  async ({ page }, name: string) => {
    const restoreButton = page.locator(
      `.file-tab-bar-restore[aria-label="Restore ${name}"]`,
    )
    await restoreButton.click()
    // `BinModal.tsx` only auto-closes once *every* bin entry is gone --
    // this shared, real `e2e-test-user` account (see
    // `currentSeededFileDisplayName`'s doc comment) can have some other,
    // concurrently-running scenario's file sitting in the bin at the same
    // moment, which would keep the modal open forever waiting on this
    // step's own item alone. Waiting for just this item's own restore
    // button to leave the DOM instead confirms the restore landed,
    // regardless of what else is in the bin.
    await restoreButton.waitFor({ state: 'detached', timeout: 5_000 })
    // If the modal *did* auto-close (this was the only entry), proceeding
    // immediately can still find Radix's dialog overlay (and the
    // `aria-hidden` sibling marker it stamps on the rest of the page while
    // open) mid-teardown, which intercepts the next click -- so wait for it
    // to actually leave the DOM. If it's still open (another scenario's
    // file remains), force it closed instead of waiting on a `binNames`
    // count that may never reach zero here.
    const modal = page.locator('[data-testid="bin-modal"]')
    if (await modal.count()) {
      await page.keyboard.press('Escape')
    }
    await modal.waitFor({ state: 'detached', timeout: 5_000 })
  },
)

Then(
  'a {string} tab appears within 5 seconds',
  async ({ page }, name: string) => {
    // The restored tab lives in the "Files" dropdown, which the bin-restore
    // flow above never opens (the restore button lives in the separate Bin
    // *modal*, which auto-closes once its last entry is restored -- see
    // `BinModal.tsx`) -- and like every Radix `DropdownMenu.Content`, its
    // items aren't even mounted in the DOM while closed, so a locator here
    // would find nothing regardless of whether the restore itself
    // succeeded.
    await openFileList(page)
    await fileTabByExactName(page, name).waitFor({ timeout: 5_000 })
  },
)

Then(
  'both {string} and {string} tabs exist exactly once each',
  async ({ page }, nameA: string, nameB: string) => {
    await openFileList(page)
    // Exact-text match for `nameA` -- display names drop the `.poentaro`
    // suffix, so "original" is a substring of "original 2"; a plain
    // substring match would match both tabs.
    await expect(
      page.locator('.file-tabs .file-tab-name', {
        hasText: new RegExp(`^${nameA}$`),
      }),
    ).toHaveCount(1)
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: nameB }),
    ).toHaveCount(1)
  },
)

When('I reload the page after the collision restore', async ({ page }) => {
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
  await openFileList(page)
  await page
    .locator('.file-tab-name', { hasText: /^original$/ })
    .waitFor({ timeout: 15_000 })
})

Then(
  'both {string} and {string} tabs exist exactly once each after reload',
  async ({ page }, nameA: string, nameB: string) => {
    await expect(
      page.locator('.file-tabs .file-tab-name', {
        hasText: new RegExp(`^${nameA}$`),
      }),
    ).toHaveCount(1)
    await expect(
      page.locator('.file-tabs .file-tab-name', { hasText: nameB }),
    ).toHaveCount(1)
  },
)
