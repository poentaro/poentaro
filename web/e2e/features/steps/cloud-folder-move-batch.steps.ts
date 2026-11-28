import type { Page } from '@playwright/test'
import { expect } from '@playwright/test'
import { answerOf, appCallTo, routeAppCall } from '../../appCalls'
import { seedTokenFor } from '../../cloudFileHelpers'
import { appClient, valueOrFail } from '../../protocolClients'
import { currentSignedInEmail } from './cloud-account.steps'
import { Given, Then, When } from './fixtures'

const FOLDER_MOVE = 'move folder _ to _'
const DELIVERY_TIMEOUT_MS = 25_000

/** What the worker answered folder moves with, per page: `returned`,
 * or the method error in words. */
const batchAnswers = new WeakMap<Page, string[]>()

/** Removes the route that keeps folder moves from the cloud, per page. */
const unrouteFolderMoves = new WeakMap<Page, () => Promise<void>>()

Given('I record the batch renames the cloud answers', async ({ page }) => {
  const answers: string[] = []
  batchAnswers.set(page, answers)
  page.on('response', async (response) => {
    if (await appCallTo(response.request(), FOLDER_MOVE)) {
      const answer = await answerOf(FOLDER_MOVE, response)
      answers.push(answer === 'value' ? 'moved' : answer)
    }
  })
})

Then(
  'the cloud answered the batch renames with {string}',
  async ({ page }, statuses: string) => {
    await expect
      .poll(() => (batchAnswers.get(page) ?? []).join(', '), {
        timeout: DELIVERY_TIMEOUT_MS,
      })
      .toBe(statuses)
  },
)

When('folder moves cannot reach the cloud', async ({ page }) => {
  unrouteFolderMoves.set(
    page,
    await routeAppCall(page, FOLDER_MOVE, (route) =>
      route.abort('connectionrefused'),
    ),
  )
})

When('folder moves can reach the cloud again', async ({ page }) => {
  await unrouteFolderMoves.get(page)?.()
  await page.evaluate(() => window.dispatchEvent(new Event('online')))
})

Then(
  '{string} does not exist on the server for the signed-in account',
  async ({}, fileName: string) => {
    const identityToken = await seedTokenFor(currentSignedInEmail())
    await expect
      .poll(
        async () => {
          const files = await valueOrFail(
            'list files',
            (await appClient(identityToken)).listFiles(),
          )
          return files.some(
            (file) => file.path === fileName && file.trashedAt === null,
          )
        },
        { timeout: DELIVERY_TIMEOUT_MS },
      )
      .toBe(false)
  },
)
