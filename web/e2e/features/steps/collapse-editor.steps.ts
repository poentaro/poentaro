import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Collapse Editor Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

Given('the collapse-editor test fixture is loaded', async ({ page }) => {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'original.poentaro',
        userFiles: { 'original.poentaro': src },
        bin: {},
        fileIds: { 'original.poentaro': crypto.randomUUID() },
      }),
    )
  }, SOURCE)

  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

function editorPane(page: import('@playwright/test').Page) {
  return page.locator('.pane--editor')
}

function toggleButton(page: import('@playwright/test').Page) {
  return page.locator('.pane-divider-toggle')
}

Then('the editor pane is expanded with nonzero width', async ({ page }) => {
  await expect(editorPane(page)).not.toHaveClass(/pane--editor-collapsed/)
  const expandedWidth = await editorPane(page).evaluate(
    (el) => el.getBoundingClientRect().width,
  )
  expect(expandedWidth).toBeGreaterThan(0)
})

When('I click the pane-divider toggle button', async ({ page }) => {
  await toggleButton(page).click()
})

Then('the editor pane is collapsed', async ({ page }) => {
  await expect(editorPane(page)).toHaveClass(/pane--editor-collapsed/)
})

Then('the editor pane is expanded', async ({ page }) => {
  await expect(editorPane(page)).not.toHaveClass(/pane--editor-collapsed/)
})

Then(
  'the pane-divider toggle button title is {string}',
  async ({ page }, title: string) => {
    await expect(toggleButton(page)).toHaveAttribute('title', title)
  },
)

Then(
  'the editor pane width shrinks to less than 2 pixels',
  async ({ page }) => {
    await expect
      .poll(
        async () =>
          editorPane(page).evaluate((el) => el.getBoundingClientRect().width),
        { timeout: 3_000 },
      )
      .toBeLessThan(2)
  },
)

Then('the editor pane width grows to more than 50 pixels', async ({ page }) => {
  await expect
    .poll(
      async () =>
        editorPane(page).evaluate((el) => el.getBoundingClientRect().width),
      { timeout: 3_000 },
    )
    .toBeGreaterThan(50)
})

Then('the Monaco editor is visible', async ({ page }) => {
  await expect(page.locator('.monaco-editor')).toBeVisible()
})
