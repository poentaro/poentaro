import type { Page } from '@playwright/test'
import { expect } from '@playwright/test'
import createClient from 'openapi-fetch'
import type { paths } from '../../../src/generated/live-share-worker/schema'
import { answerOf, appCallTo, routeAppCall } from '../../appCalls'
import { CLOUD_WORKER_ORIGIN, seedTokenFor } from '../../cloudFileHelpers'
import { currentSignedInEmail } from './cloud-account.steps'
import { Given, Then, When } from './fixtures'

const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })
const FOLDER_MOVE = 'folders/move _ to _'
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
          const { data } = await worker.POST('/files/list', {
            body: { identityToken },
          })
          return (
            data?.files.some(
              (file) => file.name === fileName && file.trashedAt === null,
            ) ?? false
          )
        },
        { timeout: DELIVERY_TIMEOUT_MS },
      )
      .toBe(false)
  },
)
