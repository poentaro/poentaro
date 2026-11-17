import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Mobile Workspace Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

Given('a seeded score and a mobile viewport', async ({ page }) => {
  // The original spec set this via `test.use({ viewport })`, which isn't
  // available from within a step body under playwright-bdd — set it directly
  // on the page instead, before the app's first navigation.
  await page.setViewportSize({ width: 375, height: 700 })

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

Then(
  'the editor pane is collapsed and the preview pane fills the mobile viewport',
  async ({ page }) => {
    const editorPane = page.locator('.pane--editor')
    const previewPane = page.locator('.pane--preview')
    const toggleIcon = page.locator('.pane-divider-toggle-icon')

    await expect(editorPane).toHaveClass(/pane--editor-collapsed/)
    await expect(previewPane).not.toHaveClass(/pane--preview-collapsed/)

    const previewHeight = await previewPane.evaluate(
      (el) => el.getBoundingClientRect().height,
    )
    expect(previewHeight).toBeGreaterThan(300)

    // Preview is below the collapsed editor, so the chevron points down.
    await expect(toggleIcon).toHaveCSS('transform', 'matrix(0, -1, 1, 0, 0, 0)')
  },
)

Then(
  'the app header overflows horizontally instead of wrapping to a second row',
  async ({ page }) => {
    const header = page.locator('.app-header')

    const { scrollWidth, clientHeight } = await header.evaluate((el) => ({
      scrollWidth: el.scrollWidth,
      clientHeight: el.getBoundingClientRect().height,
    }))

    // On a 375px-wide viewport the header's content (title, playback buttons,
    // file switcher, export controls) is wider than the viewport, so it only
    // fits without wrapping if it overflows horizontally instead.
    expect(scrollWidth).toBeGreaterThan(375)
    // A single, non-wrapped row stays well under the height a two-row wrap
    // would produce.
    expect(clientHeight).toBeLessThan(60)
  },
)

When('I click the pane-divider toggle', async ({ page }) => {
  await page.locator('.pane-divider-toggle').click()
})

When('I click the pane-divider toggle again', async ({ page }) => {
  await page.locator('.pane-divider-toggle').click()
})

Then(
  'the editor pane is shown and the preview pane is collapsed',
  async ({ page }) => {
    const editorPane = page.locator('.pane--editor')
    const previewPane = page.locator('.pane--preview')
    const toggleIcon = page.locator('.pane-divider-toggle-icon')

    await expect(editorPane).not.toHaveClass(/pane--editor-collapsed/)
    await expect(previewPane).toHaveClass(/pane--preview-collapsed/)
    await expect(page.locator('.monaco-editor')).toBeVisible()

    // Editor is now the visible pane, above the collapsed preview, so the
    // chevron points up.
    await expect(toggleIcon).toHaveCSS('transform', 'matrix(0, 1, -1, 0, 0, 0)')

    await expect
      .poll(
        async () =>
          previewPane.evaluate((el) => el.getBoundingClientRect().height),
        { timeout: 3_000 },
      )
      .toBeLessThan(2)
  },
)

Then(
  'the editor pane is collapsed and the preview pane is shown again',
  async ({ page }) => {
    const editorPane = page.locator('.pane--editor')
    const previewPane = page.locator('.pane--preview')

    await expect(editorPane).toHaveClass(/pane--editor-collapsed/)
    await expect(previewPane).not.toHaveClass(/pane--preview-collapsed/)
  },
)

Given(
  "the mobile browser's 100vh is taller than its visible area",
  async ({ page }) => {
    // On real phones `100vh` is the *large* viewport (as if the URL bar were
    // hidden), so anything sized to it overflows the visible area by the URL
    // bar's height. Desktop Chromium's mobile emulation has no URL bar, so
    // `100vh` always equals the visible height there — simulate the gap by
    // making `.app` taller than the viewport the way a phone would.
    await page.addStyleTag({
      content: '.app { height: calc(100vh + 120px) !important; }',
    })
  },
)

When(
  'I scroll the page with the pointer over the toolbars',
  async ({ page }) => {
    const box = await page.locator('.app-header').boundingBox()
    if (box == null) throw new Error('app header is not rendered')
    await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2)
    await page.mouse.wheel(0, 300)
    // Give the scroll a frame to land before asserting it didn't happen.
    await page.evaluate(
      () => new Promise((resolve) => requestAnimationFrame(resolve)),
    )
  },
)

Then('the page itself does not scroll', async ({ page }) => {
  expect(await page.evaluate(() => window.scrollY)).toBe(0)
})

Then(
  'the toolbars stay pinned at the top of the viewport',
  async ({ page }) => {
    // Every rendered bar stacked above the workspace (header, sections
    // toolbar, part toggles — whichever this score shows).
    const toolbars = page.locator('.app > :not(.workspace)')
    for (const toolbar of await toolbars.all()) {
      const { top, height } = await toolbar.evaluate((el) =>
        el.getBoundingClientRect(),
      )
      if (height === 0) continue
      expect(top).toBeGreaterThanOrEqual(0)
      await expect(toolbar).toBeInViewport({ ratio: 1 })
    }
  },
)

When('I open the Export dropdown menu', async ({ page }) => {
  const menuButton = page.getByRole('button', { name: 'Export', exact: true })
  await expect(menuButton).toBeEnabled({ timeout: 30_000 })
  await menuButton.click()
})

Then(
  'every export menu item is within the mobile viewport',
  async ({ page }) => {
    // The header scrolls horizontally on mobile (see previous test), which
    // would clip a Radix `DropdownMenu`'s vertical overflow too, trapping it
    // inside the ~48px-tall header strip instead of floating over the page.
    // Below the mobile breakpoint the menu renders as a full-screen drawer
    // (role="dialog") instead, sidestepping that entirely.
    const menu = page.getByRole('dialog')
    await expect(menu).toBeVisible()

    const items = await menu.getByRole('menuitem').all()
    expect(items.length).toBeGreaterThan(0)
    for (const item of items) {
      await expect(item).toBeInViewport()
    }
  },
)
