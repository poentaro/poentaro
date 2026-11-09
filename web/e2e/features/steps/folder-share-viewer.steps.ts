import { expect, type Locator, type Page } from '@playwright/test'
import {
  DEFAULT_CLOUD_FILE_CONTENT,
  renameCloudFile,
  seedCloudFile,
  seedTokenFor,
  trashCloudFile,
  workerRouteGlob,
} from '../../cloudFileHelpers'
import { openFileList } from '../../fileSwitcherHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { seedSignedInSession } from './cloud-account-helpers'
import { Given, Then, When } from './fixtures'
import { syncedShareButtonState as state } from './synced-share-button-state'

/** Releases the viewer's held-back `/files/list` requests; set while they are
 * held. */
let releaseViewerFileList: (() => void) | undefined

/** Ids of the cloud files this scenario seeded, by name, for the owner's
 * API-side changes (rename, delete). */
const seededIds = new Map<string, string>()

const contentTitled = (title: string) =>
  DEFAULT_CLOUD_FILE_CONTENT.replace('Cloud Backend Test', title)

function viewer(): Page {
  if (!state.viewerPage) throw new Error('the viewer page was not opened yet')
  return state.viewerPage
}

async function openViewer(
  browser: import('@playwright/test').Browser,
  url: string,
  options: {
    phone?: boolean
    localFolder?: string
    cloudAccount?: string
    holdFileList?: boolean
  } = {},
) {
  state.viewerContext = await browser.newContext(
    options.phone ? { viewport: { width: 390, height: 800 } } : {},
  )
  const page = await state.viewerContext.newPage()
  if (options.localFolder) {
    await page.addInitScript((folder: string) => {
      if (localStorage.getItem('poentaro:files:v1')) return
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: `${folder}/mine.poentaro`,
          userFiles: {
            [`${folder}/mine.poentaro`]: 'score """\n    1 2 3 4\n    """\n',
          },
          bin: {},
          fileIds: { [`${folder}/mine.poentaro`]: crypto.randomUUID() },
          folders: [],
        }),
      )
    }, options.localFolder)
  }
  if (options.cloudAccount) {
    await seedSignedInSession(page, options.cloudAccount)
    await page.addInitScript(() => {
      localStorage.setItem(
        'poentaro:storage-backend:v1',
        JSON.stringify({ backend: 'cloud' }),
      )
    })
  }
  if (options.holdFileList) {
    const released = new Promise<void>((resolve) => {
      releaseViewerFileList = resolve
    })
    await page.route(workerRouteGlob('/files/list'), async (route) => {
      await released
      await route.continue()
    })
  }
  state.viewerPage = page
  await page.goto(url)
  await page.waitForSelector('.preview-page', { timeout: 15_000 })
}

Given(
  'a cloud file named {string} titled {string} is seeded for the signed-in account',
  async ({}, name: string, title: string) => {
    const file = await seedCloudFile(
      currentSignedInEmail(),
      name,
      contentTitled(title),
    )
    seededIds.set(name, file.id)
  },
)

const syncedLinkOf = (page: Page) =>
  page.getByTestId('share-modal-synced-link').inputValue()

When(
  'a viewer opens the synced link of the share dialog',
  async ({ page, browser }) => {
    await openViewer(browser, await syncedLinkOf(page))
  },
)

When(
  'a viewer opens the synced link of the share dialog on a phone',
  async ({ page, browser }) => {
    await openViewer(browser, await syncedLinkOf(page), { phone: true })
  },
)

When(
  'a viewer who already has the local folder {string} opens the synced link of the share dialog',
  async ({ page, browser }, folder: string) => {
    await openViewer(browser, await syncedLinkOf(page), {
      localFolder: folder,
    })
  },
)

When(
  'a cloud viewer opens the synced link of the share dialog',
  async ({ page, browser }) => {
    await openViewer(browser, await syncedLinkOf(page), {
      cloudAccount: currentSignedInEmail(),
    })
  },
)

When(
  'a cloud viewer whose files are still loading opens the synced link of the share dialog',
  async ({ page, browser }) => {
    await openViewer(browser, await syncedLinkOf(page), {
      cloudAccount: currentSignedInEmail(),
      holdFileList: true,
    })
  },
)

Then('the viewer cannot import the folder yet', async () => {
  await expect(
    viewer().getByRole('button', { name: 'Import this folder' }),
  ).toBeDisabled()
})

When("the viewer's files finish loading", async () => {
  releaseViewerFileList?.()
  releaseViewerFileList = undefined
  await expect(
    viewer().getByRole('button', { name: 'Import this folder' }),
  ).toBeEnabled()
})

When(
  'a viewer opens the static link of the share dialog',
  async ({ page, browser }) => {
    const link = await page.getByTestId('share-modal-static-link').inputValue()
    await openViewer(browser, link)
  },
)

When(
  'the owner adds the cloud file {string} titled {string}',
  async ({}, name: string, title: string) => {
    const file = await seedCloudFile(
      currentSignedInEmail(),
      name,
      contentTitled(title),
    )
    seededIds.set(name, file.id)
  },
)

When(
  'the owner renames the cloud file {string} to {string}',
  async ({}, from: string, to: string) => {
    const id = seededIds.get(from)
    if (!id) throw new Error(`${from} was not seeded`)
    await renameCloudFile(currentSignedInEmail(), id, to)
  },
)

When(
  'the owner edits the cloud file {string} to be titled {string}',
  async ({}, name: string, title: string) => {
    // Seeding an existing name overwrites its content.
    await seedCloudFile(currentSignedInEmail(), name, contentTitled(title))
  },
)

When('the owner deletes the cloud file {string}', async ({}, name: string) => {
  const id = seededIds.get(name)
  if (!id) throw new Error(`${name} was not seeded`)
  await trashCloudFile(currentSignedInEmail(), id)
})

When('the owner stops sharing the folder', async ({ page }) => {
  await page.getByTestId('share-modal-stop-sync').click()
  await page.getByTestId('share-modal-confirm-stop-sync').click()
  await expect(page.getByTestId('share-modal-start-sync')).toBeVisible({
    timeout: 15_000,
  })
})

When('the viewer opens the file list', async () => {
  await viewer()
    .getByTestId('shared-folder-switcher')
    .getByRole('button')
    .click()
})

When('the viewer expands the folder {string}', async ({}, folder: string) => {
  await viewer().getByRole('button', { name: folder, exact: true }).click()
})

When('the viewer selects the file {string}', async ({}, name: string) => {
  await viewer()
    .locator('.file-tab-name', { hasText: new RegExp(`^${name}$`) })
    .click()
})

When('the viewer imports the folder', async () => {
  await viewer().getByRole('button', { name: 'Import this folder' }).click()
})

When('the viewer clicks Discard', async () => {
  await viewer().getByRole('button', { name: 'Discard' }).click()
})

Then('the viewer banner says {string}', async ({}, text: string) => {
  await expect(viewer().locator('.shared-preview-banner')).toContainText(text)
})

Then('the viewer banner does not say {string}', async ({}, text: string) => {
  const banner = viewer().locator('.shared-preview-banner')
  await expect(banner).toBeVisible()
  await expect(banner).not.toContainText(text)
})

Then('the viewer sees no shared folder banner', async () => {
  await expect(viewer().locator('.shared-preview-banner')).toHaveCount(0)
})

Then(
  "the viewer's file list shows the file {string}",
  async ({}, name: string) => {
    await expect(
      viewer().locator('.file-tab-name', { hasText: new RegExp(`^${name}$`) }),
    ).toBeVisible()
  },
)

Then(
  "the viewer's file list shows no file {string}",
  async ({}, name: string) => {
    await expect(
      viewer().locator('.file-tab-name', { hasText: new RegExp(`^${name}$`) }),
    ).toHaveCount(0)
  },
)

Then("the viewer's file list has no Demo folder", async () => {
  await expect(
    viewer().getByRole('button', { name: 'Demo', exact: true }),
  ).toHaveCount(0)
})

Then("the viewer's file list offers no editing actions", async () => {
  await expect(
    viewer().getByRole('button', { name: /^Actions for/ }),
  ).toHaveCount(0)
})

Then('the viewer has no editor', async () => {
  await expect(viewer().locator('.monaco-editor')).toHaveCount(0)
})

Then("the viewer's page URL has the synced share param", async () => {
  expect(new URL(viewer().url()).searchParams.has('s')).toBe(true)
})

Then("the viewer's page URL has no synced share param", async () => {
  await expect
    .poll(() => new URL(viewer().url()).searchParams.has('s'))
    .toBe(false)
})

Then("the viewer's page URL has no share hash", async () => {
  await expect.poll(() => new URL(viewer().url()).hash).toBe('')
})

Then('the viewer sees no shared viewer notice', async () => {
  await expect(viewer().getByTestId('shared-viewer-notice')).toHaveCount(0)
})

Then('the viewer sees the "no longer shared" screen', async () => {
  await expect(viewer().getByTestId('shared-viewer-notice')).toContainText(
    'no longer shared',
  )
})

When('the viewer resizes to a phone screen', async () => {
  await viewer().setViewportSize({ width: 390, height: 800 })
})

async function expectFullyInsideScreen(locator: Locator) {
  await expect(locator).toBeVisible()
  const box = await locator.boundingBox()
  const width = viewer().viewportSize()?.width ?? 0
  expect(box).not.toBeNull()
  expect(box?.x ?? -1).toBeGreaterThanOrEqual(0)
  expect((box?.x ?? 0) + (box?.width ?? 0)).toBeLessThanOrEqual(width)
}

Then("the viewer's export button is fully inside the screen", async () => {
  await expectFullyInsideScreen(
    viewer().getByRole('button', { name: 'Export', exact: true }),
  )
})

Then("the viewer's folder picker is fully inside the screen", async () => {
  await expectFullyInsideScreen(viewer().getByTestId('shared-folder-switcher'))
})

Then("the viewer's import button is after the export button", async () => {
  const exportBox = await viewer()
    .getByRole('button', { name: 'Export', exact: true })
    .boundingBox()
  const importBox = await viewer()
    .locator('.shared-preview-import-btn')
    .boundingBox()
  expect(exportBox).not.toBeNull()
  expect(importBox).not.toBeNull()
  expect(importBox?.x ?? 0).toBeGreaterThan(
    (exportBox?.x ?? 0) + (exportBox?.width ?? 0),
  )
})

Then(
  'the cloud account holds the file {string}',
  async ({ page }, name: string) => {
    await expect
      .poll(
        async () => {
          const response = await page.request.post(
            'http://localhost:8797/files/list',
            {
              data: {
                identityToken: await seedTokenFor(currentSignedInEmail()),
              },
            },
          )
          const body = (await response.json()) as {
            files: { name: string; trashedAt: string | null }[]
          }
          return body.files.some(
            (file) => file.name === name && file.trashedAt === null,
          )
        },
        { timeout: 20_000 },
      )
      .toBe(true)
  },
)

Then(
  "the viewer's folder list has the folder {string}",
  async ({}, name: string) => {
    await openFileList(viewer())
    await expect(
      viewer().getByRole('button', { name, exact: true }),
    ).toBeVisible({ timeout: 15_000 })
  },
)
