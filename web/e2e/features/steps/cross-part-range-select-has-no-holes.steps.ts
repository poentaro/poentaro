import { expect, type Page } from '@playwright/test'
import { rectVariantSelector, tagSelector } from '../../../src/dataAttributes'
import { stableBoundingBox } from '../../rangeSelectHelpers'
import { Given, Then, When } from './fixtures'

const FILE_NAME = 'cross-part-range-select-has-no-holes.poentaro'

/** The `[X]` abbreviations of the score's `parts [...]` section, in declaration
 * order — a part's position here is its `sourcePartIndex`. */
let partAbbreviations: string[] = []

function partIndex(abbreviation: string): number {
  const index = partAbbreviations.indexOf(abbreviation)
  if (index === -1) throw new Error(`No part [${abbreviation}] in the score.`)
  return index
}

/** `"3, 4 and 5"` → `[3, 4, 5]`. */
function ordinals(list: string): number[] {
  return list.split(/,\s*|\s+and\s+/).map((n) => Number.parseInt(n, 10))
}

// A note renders two sibling `[data-tag="note"]` groups (click target and
// playback cursor); only the click-target one is marked range-selected.
function noteGroup(page: Page, abbreviation: string, nth: number) {
  return page.locator(
    `${tagSelector('note', { sourcePartIndex: partIndex(abbreviation), noteId: nth - 1 })}:has(${rectVariantSelector('note-click-target')})`,
  )
}

// A syllable's `noteId` is the id of the note it is sung on.
function syllableGroup(page: Page, abbreviation: string, nth: number) {
  return page.locator(
    tagSelector('lyric', {
      sourcePartIndex: partIndex(abbreviation),
      noteId: nth - 1,
    }),
  )
}

async function clickCenterOf(
  page: Page,
  target: ReturnType<Page['locator']>,
): Promise<void> {
  const box = await stableBoundingBox(target)
  if (!box) throw new Error('Could not get a bounding box for the click.')
  await page.mouse.move(box.x + box.width / 2, box.y + box.height / 2, {
    steps: 10,
  })
  await page.mouse.down()
  await page.mouse.up()
}

Given('the score:', async ({ page }, body: string) => {
  const [partsSection = ''] = body.split('score')
  partAbbreviations = Array.from(
    partsSection.matchAll(/"([^"]+)"(?:\s*\[([^\]]+)\])?\s*:/g),
    (match) => match[2] ?? match[1] ?? '',
  )
  const source = [
    'metadata { title "cross part range test", max measures per system 48 },',
    '',
    body,
  ].join('\n')
  await page.addInitScript(
    ({ fileName, src }) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: fileName,
          userFiles: { [fileName]: src },
          bin: {},
          fileIds: { [fileName]: `${fileName}-id-001` },
        }),
      )
    },
    { fileName: FILE_NAME, src: source },
  )
  await page.goto('/')
  await page.waitForSelector('[data-testid="play-measure-button"]', {
    timeout: 15_000,
  })
  await expect(
    page.locator(rectVariantSelector('note-click-target')).first(),
  ).toBeVisible({ timeout: 10_000 })
  await page.evaluate(() => document.fonts.ready)
})

When(
  'I click note {int} of part {word}',
  async ({ page }, nth: number, abbreviation: string) => {
    await clickCenterOf(
      page,
      noteGroup(page, abbreviation, nth).locator(
        rectVariantSelector('note-click-target'),
      ),
    )
  },
)

When(
  'I click syllable {int} of part {word}',
  async ({ page }, nth: number, abbreviation: string) => {
    await clickCenterOf(page, syllableGroup(page, abbreviation, nth))
  },
)

Then(
  /^notes? ([\d, and]+) of part (\w+) (?:is|are) (not )?range-selected$/,
  async ({ page }, list: string, abbreviation: string, not?: string) => {
    for (const nth of ordinals(list)) {
      const group = expect(noteGroup(page, abbreviation, nth))
      await (not ? group.not : group).toHaveAttribute(
        'data-note-range-selected',
        '',
      )
    }
  },
)

Then(
  /^syllables? ([\d, and]+) of part (\w+) (?:is|are) (not )?range-selected$/,
  async ({ page }, list: string, abbreviation: string, not?: string) => {
    for (const nth of ordinals(list)) {
      const group = expect(syllableGroup(page, abbreviation, nth))
      await (not ? group.not : group).toHaveAttribute(
        'data-lyric-range-selected',
        '',
      )
    }
  },
)
