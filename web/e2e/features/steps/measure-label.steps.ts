import { expect } from '@playwright/test'
import { Given, Then, When } from './fixtures'

/**
 * The default demo file (demo/01-pitches.poentaro, opened on first load) has
 * the following Monaco line numbers (1-based):
 *
 *   1  metadata { title "Pitches", max measures per system 1 },
 *   ...
 *   5  score """
 *   6      label="Scale degrees & rest"
 *   7      [M] 1 2 3 0    ← first note line → measure 1
 */

Given(
  'the Chinese-lyric measure-label test fixture is loaded',
  async ({ page }) => {
    /**
     * Regression: exercises measure detection when the cursor is at the end of
     * a multi-byte-UTF-8 (CJK) lyric line that is also the last line of the
     * measure.
     *
     * The lyric line "白陽旗旛在大道盛宏" is line 13 (1-based) in the source
     * below. Pressing End places the cursor after "宏" (3-byte UTF-8 char).
     */
    const source = [
      'metadata { title "abc", author "author" },',
      '',
      'parts [',
      '    "Chord" [C]: chords,',
      '    "Alto 1 & Tenor" ["A1,T"]: notes,',
      '    "Alto 1 & Tenor lyrics" ["A1,Tv1"]: lyrics ["A1,T"],',
      '],',
      '',
      'score """',
      '    bpm=80 key=C4 time=4/4 label="Verse 1"',
      '    [C] 1 - - -',
      '    [A1,T] 5_ 5_ 5_ 5= 5= 5_ 3_ 2_ (3_',
      '    [A1,Tv1] 白陽旗旛在大道盛宏',
      '',
      '    [C] 6m/3',
      '    [A1,T] 3_) (1_1-) 0_ 1= 1=',
      '    [A1,Tv1] 昌花花',
      '    """,',
    ].join('\n')

    await page.addInitScript((src) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'chinese-test.poentaro',
          userFiles: { 'chinese-test.poentaro': src },
          bin: {},
          fileIds: { 'chinese-test.poentaro': crypto.randomUUID() },
        }),
      )
    }, source)

    await page.goto('/')
    await page.waitForSelector('[data-testid="play-measure-button"]', {
      timeout: 15_000,
    })
  },
)

When(
  'I jump to line {int} and wait for the selection debounce',
  async ({ page, focusEditor }, line: number) => {
    // Focus the Monaco editor.
    await focusEditor()

    // Use Monaco's "Go to Line" command (Ctrl+G) to jump to the given line.
    await page.keyboard.press('Control+g')
    await page.keyboard.type(String(line))
    await page.keyboard.press('Enter')

    // Allow the 300 ms debounce in notifySelection plus worker round-trip.
    await page.waitForTimeout(700)
  },
)

When(
  'I jump to line {int}, press End, and wait for the selection debounce',
  async ({ page, focusEditor }, line: number) => {
    // Focus the Monaco editor.
    await focusEditor()

    // Navigate to the given line and press End to put the cursor after the
    // last character of that line.
    await page.keyboard.press('Control+g')
    await page.keyboard.type(String(line))
    await page.keyboard.press('Enter')
    await page.keyboard.press('End')

    // Allow the 300 ms debounce in notifySelection plus worker round-trip.
    await page.waitForTimeout(700)
  },
)

Then(
  'the play-measure button shows {string}',
  async ({ page }, text: string) => {
    await expect(page.getByTestId('play-measure-button')).toContainText(text, {
      timeout: 3_000,
    })
  },
)
