import type { Response } from '@playwright/test'
import { expect } from '@playwright/test'
import { publicCallTo } from '../../protocolClients'
import { AfterScenario, Then, When } from './fixtures'
import { syncedShareButtonState as state } from './synced-share-button-state'

// Not part of `SyncedShareButtonState` -- only this file's own scenario reads
// it, so it doesn't need to be threaded through the shared cross-step state.
let capturedShareResponse: Response | undefined

AfterScenario(async () => {
  capturedShareResponse = undefined
  if (state.viewerContext) {
    await state.viewerContext.close()
    state.viewerContext = undefined
  }
})

When(
  'a viewer opens the copied sync link in a new page, capturing the share response',
  async ({ browser }) => {
    if (!state.syncedShareLink)
      throw new Error('syncedShareLink was not captured yet')
    state.viewerContext = await browser.newContext()
    state.viewerPage = await state.viewerContext.newPage()

    // Watch the public protocol's document call before navigation starts it.
    const shareResponse = state.viewerPage.waitForResponse((response) =>
      publicCallTo(response.request(), 'open shared link _'),
    )

    await state.viewerPage.goto(state.syncedShareLink)
    capturedShareResponse = await shareResponse
    await state.viewerPage.waitForSelector('.preview-page', {
      timeout: 15_000,
    })
  },
)

Then(
  'the captured share response has a {string} header of {string}',
  async ({}, headerName: string, expectedValue: string) => {
    if (!capturedShareResponse)
      throw new Error('share response was not captured yet')
    const headers = capturedShareResponse.headers()
    expect(headers[headerName.toLowerCase()]).toBe(expectedValue)
  },
)
