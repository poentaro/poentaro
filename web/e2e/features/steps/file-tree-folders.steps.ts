import { expect, type Page } from '@playwright/test'
import { Then, When } from './fixtures'
import { exportedPdfBytes } from './import.steps'

function dialog(page: Page, name: string) {
  return page.getByRole('dialog', { name })
}

function folderMenuButton(page: Page, folder: string) {
  return page.getByRole('button', {
    name: `Actions for folder ${folder}`,
    exact: true,
  })
}

async function openFolderMenu(page: Page, folder: string) {
  const row = page
    .locator('li.file-tree-row')
    .filter({ has: page.getByRole('button', { name: folder, exact: true }) })
  await row.hover()
  await folderMenuButton(page, folder).click()
}

When(
  'I open the actions menu of the folder {string}',
  async ({ page }, folder: string) => {
    await openFolderMenu(page, folder)
  },
)

When(
  'I choose {string} from the actions menu of the folder {string}',
  async ({ page }, item: string, folder: string) => {
    await openFolderMenu(page, folder)
    await page.getByRole('menuitem', { name: item, exact: true }).click()
  },
)

Then('the folder menu offers {string}', async ({ page }, item: string) => {
  await expect(
    page.getByRole('menuitem', { name: item, exact: true }),
  ).toBeVisible()
})

Then(
  'the folder menu does not offer {string}',
  async ({ page }, item: string) => {
    await expect(
      page.getByRole('menuitem', { name: item, exact: true }),
    ).toHaveCount(0)
  },
)

Then(
  'the folder {string} has no actions menu',
  async ({ page }, folder: string) => {
    await expect(
      page.getByRole('button', { name: folder, exact: true }),
    ).toBeVisible()
    await expect(folderMenuButton(page, folder)).toHaveCount(0)
  },
)

Then(
  'the file tree does not show the folder {string}',
  async ({ page }, name: string) => {
    await expect(page.getByRole('button', { name, exact: true })).toHaveCount(0)
  },
)

When(
  'I import the exported PDF file into the folder {string} as {string}',
  async ({ page }, folder: string, name: string) => {
    await openFolderMenu(page, folder)
    await page.getByRole('menuitem', { name: 'Import here' }).click()
    // The menu's own input: the file browser's toolbar has one too.
    await page.setInputFiles('[role=menu] input[type=file]', {
      name: `${name}.pdf`,
      mimeType: 'application/pdf',
      buffer: exportedPdfBytes(),
    })
  },
)

When('I cancel the rename dialog', async ({ page }) => {
  await dialog(page, 'Rename').getByRole('button', { name: 'Cancel' }).click()
})

When('I cancel the move dialog', async ({ page }) => {
  await dialog(page, 'Move to').getByRole('button', { name: 'Cancel' }).click()
})

Then(
  'the move dialog offers only the folders {string} and {string}',
  async ({ page }, a: string, b: string) => {
    await expect(
      dialog(page, 'Move to').locator('.move-target-name'),
    ).toHaveText([a, b])
  },
)

Then(
  'the delete folder dialog says {string}',
  async ({ page }, text: string) => {
    await expect(dialog(page, 'Delete folder')).toContainText(text)
  },
)

When('I cancel the delete folder dialog', async ({ page }) => {
  await dialog(page, 'Delete folder')
    .getByRole('button', { name: 'Cancel' })
    .click()
})

When('I confirm the delete folder dialog', async ({ page }) => {
  await dialog(page, 'Delete folder')
    .getByRole('button', { name: 'Move to bin' })
    .click()
})

Then(
  'the bin lists {string} located in {string}',
  async ({ page }, name: string, location: string) => {
    const entry = page.locator('.file-tab-bar-bin-item').filter({
      has: page.locator('.file-tab-bar-bin-name', {
        hasText: new RegExp(`^${name}$`),
      }),
    })
    await expect(entry.locator('.file-tab-bar-bin-where')).toHaveText(location)
  },
)

When('I restore {string} from the bin', async ({ page }, key: string) => {
  await page
    .locator(`.file-tab-bar-restore[aria-label="Restore ${key}"]`)
    .click()
})
