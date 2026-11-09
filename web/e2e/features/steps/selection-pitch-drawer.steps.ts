import type { Locator, Page } from '@playwright/test'
import { expect } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import { focusEditor } from '../../fileSwitcherHelpers'
import {
  clickThenStableClick,
  stableBoundingBox,
} from '../../rangeSelectHelpers'
import { gotoShareUrl } from '../../shareUrlHelper'
import { Given, Then, When } from './fixtures'

// Each scenario builds its own score inline, so remember its data lines to
// map a token like "5," on the M line to the note id the preview gives it.
// Note ids count every note/rest/chord of a part in order, across measures;
// a chord's `-` extensions are not notes.
let loadedLines: { label: string; tokens: string[] }[] = []

function scoreSource(key: string, partDeclaration: string, body: string) {
  return [
    'metadata { title "pitch drawer test", max measures per system 48 },',
    '',
    `parts [${partDeclaration}],`,
    '',
    'score """',
    `bpm=120 key=${key} time=4/4`,
    body,
    '""",',
  ].join('\n')
}

function rememberLines(body: string) {
  loadedLines = body
    .split('\n')
    .filter((line) => line.startsWith('['))
    .map((line) => {
      const [label = '', ...tokens] = line.split(/\s+/)
      return {
        label: label.slice(1, -1),
        tokens: tokens.filter((token) => token !== '-'),
      }
    })
}

function noteIdOf(label: string, token: string) {
  const tokens = loadedLines
    .filter((line) => line.label === label)
    .flatMap((line) => line.tokens)
  const noteId = tokens.indexOf(token)
  if (noteId < 0) throw new Error(`No "${token}" on the ${label} line.`)
  return noteId
}

async function loadScore(page: Page, source: string) {
  await page.addInitScript((src) => {
    localStorage.setItem(
      'poentaro:files:v1',
      JSON.stringify({
        active: 'pitch-drawer-test.poentaro',
        userFiles: { 'pitch-drawer-test.poentaro': src },
        bin: {},
        fileIds: { 'pitch-drawer-test.poentaro': crypto.randomUUID() },
      }),
    )
  }, source)
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await expect(
    page.locator(rectVariantSelector('note-click-target')).first(),
  ).toBeVisible({ timeout: 10_000 })
  const isMobile = (page.viewportSize()?.width ?? 1280) <= 768
  if (isMobile) {
    // The editor pane starts collapsed on a phone, so it can't be focused
    // for the priming below; just give `listNoteSpans` time to land.
    await page.waitForTimeout(1_000)
    return
  }
  // Primes the editor/worker round-trip (same as the other click-select
  // specs) so `noteSpans` has settled before hit-testing, and leaves the
  // editor focused so Escape reaches Monaco.
  await focusEditor(page)
  await page.keyboard.press('Control+g')
  await page.keyboard.type('7')
  await page.keyboard.press('Enter')
  await expect(page.locator('button.play-measure-btn')).toHaveText(/Measure/, {
    timeout: 5_000,
  })
}

async function loadBody(
  page: Page,
  key: string,
  partDeclaration: string,
  body: string,
) {
  rememberLines(body)
  await loadScore(page, scoreSource(key, partDeclaration, body))
}

function noteGroup(page: Page, partIndex: number, noteId: number) {
  // Every note renders two `[data-tag="note"]` groups with the same ids (see
  // `pending-second-click-affordance.steps.ts`); `:has()` picks the one
  // holding the click target.
  return page
    .locator(
      `${tagSelector('note', { sourcePartIndex: partIndex, noteId })}:has(${rectVariantSelector('note-click-target')})`,
    )
    .first()
}

function clickTarget(page: Page, noteId: number) {
  // Every score here has a single part, so it's always part index 0.
  return noteGroup(page, 0, noteId).locator(
    rectVariantSelector('note-click-target'),
  )
}

async function selectNotes(page: Page, fromNoteId: number, toNoteId: number) {
  await clickThenStableClick(
    page,
    clickTarget(page, fromNoteId),
    clickTarget(page, toNoteId),
  )
  // Off the preview, so no hover styling lingers over the drawer.
  await page.mouse.move(0, 0)
}

const drawer = (page: Page) => page.getByTestId('pitch-drawer')
const handle = (page: Page) => page.getByTestId('pitch-drawer-handle')

Given(
  'a score in key {word} with {string} is loaded',
  async ({ page }, key: string, body: string) => {
    await loadBody(page, key, '"Melody" [M]: notes', body)
  },
)

Given(
  'a shared link to a score in key {word} with {string} is opened',
  async ({ page }, key: string, body: string) => {
    rememberLines(body)
    await page.addInitScript(() => {
      localStorage.clear()
    })
    await gotoShareUrl(
      page,
      'pitch-drawer-test.poentaro',
      scoreSource(key, '"Melody" [M]: notes', body),
    )
    await expect(page.getByText('Import this score')).toBeVisible({
      timeout: 15_000,
    })
    await expect(
      page.locator(rectVariantSelector('note-click-target')).first(),
    ).toBeVisible({ timeout: 10_000 })
    // No editor to prime the worker round-trip through, so just give
    // `listNoteSpans` time to land before hit-testing.
    await page.waitForTimeout(1_000)
  },
)

Given(
  'a score in key {word} with {string} on a chords part is loaded',
  async ({ page }, key: string, body: string) => {
    await loadBody(page, key, '"Chords" [C]: chords', body)
  },
)

Given(
  'a score whose measure 1 is in key C4 and measure 2 is in key G4',
  async ({ page }) => {
    const body = '[M] 1 2 3 4\n\nkey=G4\n[M] 1 2 3 4'
    await loadBody(page, 'C4', '"Melody" [M]: notes', body)
  },
)

Given('the viewport is a mobile phone', async ({ page }) => {
  await page.setViewportSize({ width: 375, height: 700 })
})

When(
  /^I click-and-click select the (?:note|chord|rest) "(.*)" on the (\w+) line$/,
  async ({ page }, token: string, label: string) => {
    const noteId = noteIdOf(label, token)
    await selectNotes(page, noteId, noteId)
  },
)

When(
  'I click-and-click select the note {string} in measure {int}',
  async ({ page }, token: string, measure: number) => {
    // Four notes per measure in that fixture.
    const [firstLine] = loadedLines
    if (!firstLine) throw new Error('No source fixture has been loaded.')
    const noteId = (measure - 1) * 4 + firstLine.tokens.indexOf(token)
    await selectNotes(page, noteId, noteId)
  },
)

When(
  /^I click-and-click select from note "(.*)" to note "(.*)" on the (\w+) line$/,
  async ({ page }, from: string, to: string, label: string) => {
    await selectNotes(page, noteIdOf(label, from), noteIdOf(label, to))
  },
)

async function handleDragPoints(page: Page, fractionOfHeight: number) {
  await expect(drawer(page)).toHaveAttribute('data-state', 'open')
  const drawerBox = await stableBoundingBox(drawer(page))
  const handleBox = await handle(page).boundingBox()
  if (!drawerBox || !handleBox) throw new Error('The drawer is not laid out.')
  const x = handleBox.x + handleBox.width / 2
  const y = handleBox.y + handleBox.height / 2
  return { x, fromY: y, toY: y + drawerBox.height * fractionOfHeight }
}

async function touchDrag(page: Page, fractionOfHeight: number) {
  const { x, fromY, toY } = await handleDragPoints(page, fractionOfHeight)
  const client = await page.context().newCDPSession(page)
  await client.send('Input.dispatchTouchEvent', {
    type: 'touchStart',
    touchPoints: [{ x, y: fromY }],
  })
  const steps = 10
  for (let step = 1; step <= steps; step += 1) {
    await client.send('Input.dispatchTouchEvent', {
      type: 'touchMove',
      touchPoints: [{ x, y: fromY + ((toY - fromY) * step) / steps }],
    })
  }
  await client.send('Input.dispatchTouchEvent', {
    type: 'touchEnd',
    touchPoints: [],
  })
}

When(
  "I touch-drag the pitch drawer's handle down past half the drawer's height",
  async ({ page }) => {
    await touchDrag(page, 0.75)
  },
)

When(
  "I touch-drag the pitch drawer's handle down by less than half the drawer's height",
  async ({ page }) => {
    await touchDrag(page, 0.25)
  },
)

When(
  "I mouse-drag the pitch drawer's handle down past half the drawer's height",
  async ({ page }) => {
    const { x, fromY, toY } = await handleDragPoints(page, 0.75)
    await page.mouse.move(x, fromY)
    await page.mouse.down()
    await page.mouse.move(x, toY, { steps: 10 })
    await page.mouse.up()
  },
)

// The drawer is open once it is visible and has finished sliding up flush
// with the bottom of the preview pane.
async function expectFullyOpen(page: Page) {
  await expect(drawer(page)).toHaveAttribute('data-state', 'open', {
    timeout: 10_000,
  })
  await expect(drawer(page)).toBeVisible()
  const pane = page.locator('.pane--preview')
  await expect(async () => {
    const drawerBox = await drawer(page).boundingBox()
    const paneBox = await pane.boundingBox()
    if (!drawerBox || !paneBox) throw new Error('Not laid out yet.')
    expect(
      Math.abs(drawerBox.y + drawerBox.height - (paneBox.y + paneBox.height)),
    ).toBeLessThan(1)
  }).toPass({ timeout: 3_000 })
}

Then('the pitch drawer slides up', async ({ page }) => {
  await expectFullyOpen(page)
})

Then('the pitch drawer snaps back fully open', async ({ page }) => {
  await expectFullyOpen(page)
})

Then('the pitch drawer slides down and is hidden', async ({ page }) => {
  await expect(drawer(page)).toHaveAttribute('data-state', 'closed')
  await expect(drawer(page)).toBeHidden()
})

Then('the pitch drawer is hidden', async ({ page }) => {
  // The selection itself must have landed first, or "hidden" would pass
  // before the drawer ever had a chance to open.
  await expect(
    page.locator(`${tagSelector('note')}[data-note-range-selected]`).first(),
  ).toBeVisible({ timeout: 5_000 })
  // Longer than the worker round-trip plus the slide-up animation.
  await page.waitForTimeout(1_000)
  await expect(drawer(page)).toBeHidden()
})

async function expectText(locator: Locator, text: string) {
  await expect(locator).toHaveText(text, { timeout: 10_000 })
}

Then(
  'the pitch drawer shows the letter names {string}',
  async ({ page }, names: string) => {
    await expectText(page.getByTestId('pitch-drawer-letter-names'), names)
  },
)

Then(
  'the pitch drawer shows the chord name {string}',
  async ({ page }, name: string) => {
    await expectText(page.getByTestId('pitch-drawer-chord-name'), name)
  },
)

Then(
  'the pitch drawer shows the bass note {string}',
  async ({ page }, note: string) => {
    await expectText(page.getByTestId('pitch-drawer-bass-note'), note)
  },
)

Then(
  'the pitch drawer shows a keyboard with tones {string}',
  async ({ page }, tones: string) => {
    await expect(
      page.getByTestId('pitch-drawer-piano-diagram').locator('svg'),
    ).toHaveAttribute('data-piano-tones', tones, { timeout: 10_000 })
  },
)

When(
  'I press the keyboard key with function {string}',
  async ({ page }, toneFunction: string) => {
    await page
      .locator(`[data-tone-function="${toneFunction}"]`)
      .dispatchEvent('pointerdown')
  },
)

Then(
  'the pitch drawer names the pressed keyboard key {string}',
  async ({ page }, toneFunction: string) => {
    await expectText(
      page.getByTestId('pitch-drawer-piano-function'),
      toneFunction,
    )
  },
)

Then(
  'the note {string} on the M line is still range-selected',
  async ({ page }, token: string) => {
    const noteId = noteIdOf('M', token)
    await expect(
      page
        .locator(
          `${tagSelector('note', { sourcePartIndex: 0, noteId })}[data-note-range-selected]`,
        )
        .first(),
    ).toBeAttached()
  },
)
