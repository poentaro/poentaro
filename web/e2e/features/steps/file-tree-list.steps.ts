import { expect, type Page } from '@playwright/test'
import { fileSwitcherTrigger, openFileList } from '../../fileSwitcherHelpers'
import { Given, Then, When } from './fixtures'

const SOURCE = [
  'metadata { title "Tree Test" },',
  '',
  'parts ["Melody" [M]: notes],',
  '',
  'score """',
  '    (bpm=120 key=C4 time=4/4)',
  '    1 2 3 4',
  '    """,',
].join('\n')

interface TreeSeed {
  files: string[]
  folders: string[]
  bin?: string[]
  active: string
}

const seeds = new WeakMap<Page, TreeSeed>()

async function applySeed(page: Page, seed: TreeSeed) {
  const bin = seed.bin ?? []
  await page.addInitScript(
    ({ src, files, folders, active, bin }) => {
      // Only seed on the first load so a reload keeps what the app saved.
      if (localStorage.getItem('poentaro:files:v1')) return
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active,
          userFiles: Object.fromEntries(files.map((name) => [name, src])),
          bin: Object.fromEntries(bin.map((name) => [name, src])),
          fileIds: Object.fromEntries(
            [...files, ...bin].map((name) => [name, crypto.randomUUID()]),
          ),
          folders,
        }),
      )
    },
    {
      src: SOURCE,
      files: seed.files,
      folders: seed.folders,
      active: seed.active,
      bin,
    },
  )
}

Given(
  'the file tree is seeded with files {string}, {string} and {string} and an empty folder {string} with {string} active',
  async (
    { page },
    a: string,
    b: string,
    c: string,
    folder: string,
    active: string,
  ) => {
    seeds.set(page, {
      files: [a, b, c],
      folders: [folder],
      active,
    })
  },
)

Given(
  'the file tree active file is {string}',
  async ({ page }, active: string) => {
    const seed = seeds.get(page)
    if (!seed) throw new Error('file tree seed missing')
    seeds.set(page, { ...seed, active })
  },
)

Given(
  'the file tree also has the file {string}',
  async ({ page }, path: string) => {
    const seed = seeds.get(page)
    if (!seed) throw new Error('file tree seed missing')
    seeds.set(page, { ...seed, files: [...seed.files, path] })
  },
)

Given(
  'the file tree bin also holds {string}',
  async ({ page }, path: string) => {
    const seed = seeds.get(page)
    if (!seed) throw new Error('file tree seed missing')
    seeds.set(page, { ...seed, bin: [...(seed.bin ?? []), path] })
  },
)

When('the file tree app loads', async ({ page }) => {
  const seed = seeds.get(page)
  if (!seed) throw new Error('file tree seed missing')
  await applySeed(page, seed)
  await page.goto('/')
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When('I reload the file tree app', async ({ page }) => {
  await page.reload()
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
})

When('I open the file tree list', async ({ page }) => {
  // Already open, e.g. after a folder rename that keeps the browser up.
  if (await page.getByTestId('file-browser').isVisible()) return
  await openFileList(page)
})

Then('the file browser is open', async ({ page }) => {
  await expect(page.getByTestId('file-browser')).toBeVisible()
})

function folderRow(page: Page, name: string) {
  // Scoped to the tree: the file browser's own "New folder" toolbar button
  // would otherwise match a folder named "New folder".
  return page
    .locator('li.file-tree-row')
    .getByRole('button', { name, exact: true })
}

function fileRow(page: Page, name: string) {
  return page.locator('.file-tab-name', {
    hasText: new RegExp(`^${name}$`),
  })
}

When(
  'I click the file tree folder {string}',
  async ({ page }, name: string) => {
    await folderRow(page, name).click()
  },
)

When('I click the file tree file {string}', async ({ page }, name: string) => {
  await fileRow(page, name).click()
})

When(
  'I click the file tree demo file {string}',
  async ({ page }, name: string) => {
    await fileRow(page, name).click()
  },
)

Then(
  'the file tree shows the folder {string} with a count of {int}',
  async ({ page }, name: string, count: number) => {
    await expect(folderRow(page, name)).toBeVisible()
    await expect(folderRow(page, name).locator('.file-tree-count')).toHaveText(
      String(count),
    )
  },
)

Then(
  'the file tree shows no folder {string}',
  async ({ page }, name: string) => {
    await expect(folderRow(page, name)).toHaveCount(0)
  },
)

Then(
  'the file tree shows the file {string}',
  async ({ page }, name: string) => {
    await expect(fileRow(page, name)).toBeVisible()
  },
)

Then(
  'the file tree does not show the file {string}',
  async ({ page }, name: string) => {
    await expect(fileRow(page, name)).toHaveCount(0)
  },
)

Then(
  'the file tree does not show the demo file {string}',
  async ({ page }, name: string) => {
    await expect(fileRow(page, name)).toHaveCount(0)
  },
)

Then(
  'the file tree highlights the file {string}',
  async ({ page }, name: string) => {
    await expect(
      page.locator('.file-tab--active .file-tab-name', {
        hasText: new RegExp(`^${name}$`),
      }),
    ).toBeVisible()
  },
)

Then('the file tree shows the empty folder hint', async ({ page }) => {
  await expect(page.locator('.file-tree-empty-hint')).toHaveText('Empty folder')
})

Then('the file tree shows the Demo row last', async ({ page }) => {
  const lastRow = page.locator('ul.file-tabs[aria-label="Files"] > li').last()
  await expect(lastRow.getByRole('button', { name: 'Demo' })).toBeVisible()
  await expect(lastRow).toHaveClass(/file-tree-demo-row/)
})

Then(
  'the file switcher toolbar button shows the folder prefix {string} and the file name {string}',
  async ({ page }, prefix: string, name: string) => {
    const trigger = fileSwitcherTrigger(page)
    await expect(trigger.locator('.file-switcher-folder-prefix')).toHaveText(
      prefix,
    )
    await expect(trigger).toHaveText(`${prefix} ${name}`)
  },
)

Then(
  'the file switcher toolbar button shows no folder prefix and the file name {string}',
  async ({ page }, name: string) => {
    const trigger = fileSwitcherTrigger(page)
    await expect(trigger.locator('.file-switcher-folder-prefix')).toHaveCount(0)
    await expect(trigger).toHaveText(name)
  },
)

When(
  'I search the file browser for {string}',
  async ({ page }, text: string) => {
    await page.getByRole('searchbox', { name: 'Search files' }).fill(text)
  },
)

When('I press Escape in the file browser', async ({ page }) => {
  await page.keyboard.press('Escape')
})

Then(
  'the file browser says no files match {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('file-browser')).toContainText(
      `No files match “${text}”.`,
    )
  },
)

Then('the file browser search box is empty', async ({ page }) => {
  await expect(
    page.getByRole('searchbox', { name: 'Search files' }),
  ).toHaveValue('')
})

Then('the file browser is closed', async ({ page }) => {
  await expect(page.getByTestId('file-browser')).toBeHidden()
})
