import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SINGLE_PART_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

const MULTI_PART_SOURCE = [
  'metadata { title "Test" },',
  '',
  'parts ["Melody" [M]: notes, "Harmony" [H]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    5 6 7 1',
  '    """,',
].join('\n')

async function loadSource(
  page: import('@playwright/test').Page,
  source: string,
) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'test.poentaro',
        userFiles: { 'test.poentaro': src },
        bin: {},
        fileIds: { 'test.poentaro': crypto.randomUUID() },
      }),
    )
  }, source)
}

function exportMenuButton(page: import('@playwright/test').Page) {
  return page.getByRole('button', { name: 'Export', exact: true })
}

Given('the single-part export menu source is loaded', async ({ page }) => {
  await loadSource(page, SINGLE_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

Given('the multi-part export menu source is loaded', async ({ page }) => {
  await loadSource(page, MULTI_PART_SOURCE)
  await page.goto('/')
  await page.waitForSelector('.monaco-editor .view-lines', { timeout: 15_000 })
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When('I open the export menu, as seen in export menu ux', async ({ page }) => {
  const menuButton = exportMenuButton(page)
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()
})

When('I press Escape', async ({ page }) => {
  await page.keyboard.press('Escape')
})

When(
  'I click outside the export menu on the preview pages',
  async ({ page }) => {
    await page.locator('.preview-pages').click({ position: { x: 5, y: 5 } })
  },
)

Then('the export menu is visible', async ({ page }) => {
  await expect(page.getByRole('menu')).toBeVisible()
})

Then(
  'the export menu has no {string} section',
  async ({ page }, text: string) => {
    const menu = page.getByRole('menu')
    await expect(menu.getByText(text)).toHaveCount(0)
  },
)

Then(
  'the export menu has no {string} item',
  async ({ page }, itemName: string) => {
    const menu = page.getByRole('menu')
    await expect(menu.getByRole('menuitem', { name: itemName })).toHaveCount(0)
  },
)

Then(
  'the export menu items are exactly {string}',
  async ({ page }, commaSeparatedNames: string) => {
    const menu = page.getByRole('menu')
    const itemLabels = await menu.getByRole('menuitem').allTextContents()
    expect(itemLabels).toEqual(commaSeparatedNames.split(', '))
  },
)

Then(
  'the export menu shows a {string} section',
  async ({ page }, text: string) => {
    const menu = page.getByRole('menu')
    await expect(menu.getByText(text)).toBeVisible()
  },
)

Then(
  'the export menu shows an {string} section',
  async ({ page }, text: string) => {
    const menu = page.getByRole('menu')
    await expect(menu.getByText(text)).toBeVisible()
  },
)

Then(
  'the export menu is closed and the button is collapsed',
  async ({ page }) => {
    const menuButton = exportMenuButton(page)
    await expect(page.getByRole('menu')).toHaveCount(0)
    await expect(menuButton).toHaveAttribute('aria-expanded', 'false')
  },
)
