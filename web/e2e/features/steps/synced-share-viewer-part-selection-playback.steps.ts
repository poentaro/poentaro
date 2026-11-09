import { expect } from '@playwright/test'
import { tagSelector } from '../../../src/dataAttributes'
import { stableBoundingBox } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'
import {
  seedSyncedCloudFile,
  syncedShareButtonState as state,
} from './synced-share-button-state'

/**
 * Two parts, each sounding notes across two measures, rendered in a single
 * system (`max_measures_per_system = 48`) so a part-label click's "every
 * note that part sounds across the system" scope covers both measures — the
 * exact "selection spans a range of measures" shape the bug report
 * describes.
 */
const twoPartRangeSource = [
  'metadata { title "Synced Two-Part Range-Select Score", max measures per system 48 },',
  '',
  'parts ["Melody" [M]: notes, "Bass" [B]: notes],',
  '',
  'score """',
  '[M] 1 2',
  '[B] 3 4',
  '',
  '[M] 5 6',
  "[B] 7 1'",
  '""",',
].join('\n')

/** Patches `Worker.prototype.postMessage` before the app's worker is
 * created, capturing the `enabledTracks` of the most recent
 * `generateMeasureRangeAudio` request — the exact payload
 * `useMeasureAudioPlayback.playMeasureRange` sends. `__lastEnabledTracksSeen`
 * flips `true` the instant that message is posted, independent of what
 * `enabledTracks` itself holds — `undefined` is a legitimate ("every
 * currently-visible part") value for it, so the *presence* of the message
 * has to be its own flag rather than checking `enabledTracks !== undefined`,
 * which would never resolve when that's exactly the (buggy) value observed.
 * Mirrors `part-label-cmd-click-play-selection-with-hidden-part.steps.ts`'s
 * `captureEnabledTracks`, applied to the viewer page instead of the owner's. */
async function captureEnabledTracks(page: import('@playwright/test').Page) {
  await page.addInitScript(() => {
    const win = window as typeof window & {
      __lastEnabledTracks?: string[]
      __lastEnabledTracksSeen?: boolean
    }
    const origPostMessage = Worker.prototype.postMessage
    Worker.prototype.postMessage = function (
      this: Worker,
      message: unknown,
      ...rest: unknown[]
    ) {
      if (
        message &&
        typeof message === 'object' &&
        (message as { type?: string }).type === 'generateMeasureRangeAudio'
      ) {
        win.__lastEnabledTracks = (
          message as { enabledTracks?: string[] }
        ).enabledTracks
        win.__lastEnabledTracksSeen = true
      }
      // biome-ignore lint/suspicious/noExplicitAny: forwarding to the native postMessage overload
      return (origPostMessage as any).call(this, message, ...rest)
    }
  })
}

Given(
  'the file store is seeded with a two-part synced range-select score',
  async ({ page }) => {
    await seedSyncedCloudFile(
      page,
      'synced-two-part-range-test',
      twoPartRangeSource,
    )
  },
)

When(
  'a viewer opens the copied sync link in a new page, with enabled-tracks capture, and waits for measures to render',
  async ({ browser }) => {
    if (!state.syncedShareLink)
      throw new Error('syncedShareLink was not captured yet')
    state.viewerContext = await browser.newContext()
    state.viewerPage = await state.viewerContext.newPage()
    await captureEnabledTracks(state.viewerPage)
    await state.viewerPage.goto(state.syncedShareLink)
    await state.viewerPage.waitForSelector(
      tagSelector('measure', { index: 1 }),
      { timeout: 15_000 },
    )
  },
)

When('the viewer plain-clicks the Melody part label', async ({}) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  const label = state.viewerPage
    .locator(tagSelector('part-label', { sourcePartIndex: 0 }))
    .first()
  await expect(label).toBeVisible({ timeout: 5_000 })
  const box = await stableBoundingBox(label)
  if (!box) throw new Error('Could not get bounding box for the Melody label.')

  // A plain click (mousedown + mouseup at the same point, no drag) — a
  // shortcut for selecting every note that part sounds across the system
  // (see `resolvePartLabelSelection`'s doc comment), same as it is in the
  // editable app.
  await state.viewerPage.mouse.move(
    box.x + box.width / 2,
    box.y + box.height / 2,
  )
  await state.viewerPage.mouse.down()
  await state.viewerPage.mouse.up()
})

When('the viewer clicks the play button', async ({}) => {
  if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
  const playBtn = state.viewerPage.locator(
    '[data-testid="play-measure-button"]',
  )
  // The soundfont asset loads independently of the score/measures — wait
  // for it so the click actually fires `generateMeasureRangeAudio` instead
  // of silently no-opping on a still-disabled button.
  await expect(playBtn).toBeEnabled({ timeout: 30_000 })
  await playBtn.click()
})

Then(
  'the captured enabled tracks for the viewer are exactly Melody',
  async ({}) => {
    if (!state.viewerPage) throw new Error('viewerPage was not opened yet')
    await state.viewerPage.waitForFunction(
      () =>
        (window as typeof window & { __lastEnabledTracksSeen?: boolean })
          .__lastEnabledTracksSeen === true,
      undefined,
      { timeout: 10_000 },
    )
    const enabledTracks = await state.viewerPage.evaluate(
      () =>
        (window as typeof window & { __lastEnabledTracks?: string[] })
          .__lastEnabledTracks,
    )

    // The viewer selected only Melody — playback must enable exactly that
    // part, never Bass (visible, but never selected). `undefined` (the
    // "every currently-visible part" sentinel) is exactly the buggy value —
    // see this feature's regression comment.
    expect(enabledTracks).not.toBeUndefined()
    expect([...(enabledTracks ?? [])].sort()).toEqual(['M'])
  },
)
