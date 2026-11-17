import { expect, type Page } from '@playwright/test'
import { seedCloudFile } from '../../cloudFileHelpers'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { gotoCloudApp } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'

const modal = (page: Page) => page.getByTestId('share-modal')

function folderRowOf(page: Page, folder: string) {
  return page
    .locator('li.file-tree-row')
    .filter({ has: page.getByRole('button', { name: folder, exact: true }) })
}

function fileRowOf(page: Page, name: string) {
  return page.locator('li').filter({
    has: page.locator('.file-tab-name', {
      hasText: new RegExp(`^${name}$`),
    }),
  })
}

/** Deterministic pseudo-random text: it barely compresses, so a few
 * kilobytes of it push a static folder link past its size limit. */
function incompressibleText(seed: number, length: number): string {
  let value = seed
  return Array.from({ length }, () => {
    value = (value * 1664525 + 1013904223) % 4294967296
    return 'abcdefghijklmnopqrstuvwxyz0123456789'[value % 36]
  }).join('')
}

const copiedLinks = new WeakMap<Page, string>()

Given('the cloud app is open', async ({ page }) => {
  await gotoCloudApp(page)
  await expect(fileSwitcherTrigger(page)).toBeVisible({ timeout: 15_000 })
})

Given(
  'the cloud folder {string} holds {int} large unrelated files for the signed-in account',
  async ({}, folder: string, count: number) => {
    // seedCloudFile needs the email; the shared account is the default one.
    const { DEFAULT_E2E_EMAIL } = await import('../../e2eAccounts.ts')
    for (let index = 0; index < count; index++) {
      await seedCloudFile(
        DEFAULT_E2E_EMAIL,
        `${folder}/big-${index}.poentaro`,
        `score """\n    // ${incompressibleText(index + 1, 6000)}\n    1 2 3 4\n    """\n`,
      )
    }
  },
)

When(
  'I open the share dialog of the folder {string}',
  async ({ page }, folder: string) => {
    await openFileList(page)
    const row = folderRowOf(page, folder)
    await row.hover()
    await page
      .getByRole('button', { name: `Actions for folder ${folder}` })
      .click()
    await page
      .getByRole('menuitem', { name: 'Share folder', exact: true })
      .click()
  },
)

When(
  'I open the sharing settings of the folder {string}',
  async ({ page }, folder: string) => {
    await openFileList(page)
    await folderRowOf(page, folder).hover()
    await page
      .getByRole('button', { name: `Actions for folder ${folder}` })
      .click()
    await page
      .getByRole('menuitem', { name: 'Sharing settings', exact: true })
      .click()
  },
)

Then('the share dialog is titled {string}', async ({ page }, title: string) => {
  await expect(modal(page).getByRole('heading', { name: title })).toBeVisible()
})

When('I open the synced tab of the share dialog', async ({ page }) => {
  await page.getByTestId('share-modal-synced-tab').click()
})

When('I start sync in the share dialog', async ({ page }) => {
  await page.getByTestId('share-modal-start-sync').click()
})

Then('the share dialog shows a synced link', async ({ page }) => {
  await expect(page.getByTestId('share-modal-synced-link')).toHaveValue(
    /\?s=/,
    { timeout: 15_000 },
  )
})

When('I remember the synced link in the share dialog', async ({ page }) => {
  const link = await page.getByTestId('share-modal-synced-link').inputValue()
  copiedLinks.set(page, link)
})

Then(
  'the synced link in the share dialog is the remembered one',
  async ({ page }) => {
    const remembered = copiedLinks.get(page)
    if (!remembered) throw new Error('no synced link remembered')
    const identity = (link: string) => link.split('?s=')[1]?.split('--')[0]
    await expect
      .poll(async () =>
        identity(
          await page.getByTestId('share-modal-synced-link').inputValue(),
        ),
      )
      .toBe(identity(remembered))
  },
)

When('I close the share dialog', async ({ page }) => {
  await page.keyboard.press('Escape')
  await expect(modal(page)).toHaveCount(0)
})

Then(
  'the share dialog shows the file warning {string}',
  async ({ page }, text: string) => {
    await expect(
      page.getByTestId('share-modal-own-links-warning'),
    ).toContainText(text)
  },
)

Then('the share dialog shows no file warning', async ({ page }) => {
  await expect(page.getByTestId('share-modal-own-links-warning')).toHaveCount(0)
})

When('I click stop sync in the share dialog', async ({ page }) => {
  await page.getByTestId('share-modal-stop-sync').click()
})

Then('the share dialog asks to confirm stopping', async ({ page }) => {
  await expect(page.getByTestId('share-modal-stop-confirm')).toBeVisible()
})

When('I cancel stopping sync', async ({ page }) => {
  await page.getByTestId('share-modal-cancel-stop-sync').click()
})

When('I confirm stopping sync', async ({ page }) => {
  await page.getByTestId('share-modal-confirm-stop-sync').click()
})

Then('the share dialog offers to start sync', async ({ page }) => {
  await expect(page.getByTestId('share-modal-start-sync')).toBeVisible({
    timeout: 15_000,
  })
})

Then(
  'the folder {string} shows the shared marker',
  async ({ page }, folder: string) => {
    await openFileList(page)
    await expect(
      folderRowOf(page, folder).locator('.file-tree-share-marker'),
    ).toBeVisible({ timeout: 15_000 })
  },
)

Then(
  'the folder {string} shows no shared marker',
  async ({ page }, folder: string) => {
    await openFileList(page)
    await expect(folderRowOf(page, folder)).toBeVisible()
    await expect(
      folderRowOf(page, folder).locator('.file-tree-share-marker'),
    ).toHaveCount(0)
  },
)

Then(
  'the file {string} is tagged {string}',
  async ({ page }, name: string, tag: string) => {
    await expect(
      fileRowOf(page, name).locator('.file-tree-via-tag'),
    ).toHaveText(tag, { timeout: 15_000 })
  },
)

Then(
  'the folder {string} is tagged {string}',
  async ({ page }, folder: string, tag: string) => {
    await expect(
      folderRowOf(page, folder).locator('.file-tree-via-tag'),
    ).toHaveText(tag, { timeout: 15_000 })
  },
)

Then(
  'the menu of the folder {string} offers {string}',
  async ({ page }, folder: string, item: string) => {
    await openFileList(page)
    await folderRowOf(page, folder).hover()
    await page
      .getByRole('button', { name: `Actions for folder ${folder}` })
      .click()
    await expect(page.getByRole('menuitem', { name: item })).toBeVisible()
    await page.keyboard.press('Escape')
  },
)

Then(
  'the delete folder dialog warns {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('delete-folder-share-warning')).toContainText(
      text,
    )
  },
)

Then('the share dialog shows a static link', async ({ page }) => {
  await expect(page.getByTestId('share-modal-static-link')).toHaveValue(
    /#share-folder=/,
  )
})

Then('the share dialog warns the folder is too big', async ({ page }) => {
  await expect(page.getByTestId('share-modal-too-big')).toBeVisible()
})

When('I choose the synced link from the size warning', async ({ page }) => {
  await page.getByTestId('share-modal-use-synced-link').click()
})

Then('the share dialog explains links need cloud storage', async ({ page }) => {
  await expect(page.getByTestId('share-modal-synced-cloud-only')).toBeVisible()
})

Then(
  'the share dialog describes the snapshot as {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('share-modal-static-note')).toContainText(
      text,
    )
  },
)
