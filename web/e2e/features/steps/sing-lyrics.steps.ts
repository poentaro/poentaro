import { expect } from '@playwright/test'
import { Given, Then } from './fixtures'

const SINGING_SOURCE = [
  'parts [',
  '    "Soprano" [S]: notes { instrument: soprano },',
  '    "Words" [w]: lyrics [S],',
  '],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120',
  '    [S] 1 2 3 4',
  '    [w] 记 得 我 们',
  '    """,',
].join('\n')

const SINGER_WASM = 'poentaro_singer_wasm.core.wasm'

let singerDownloads: string[] = []

Given('downloads of the singing voice are recorded', async ({ page }) => {
  singerDownloads = []
  page.on('request', (request) => {
    if (request.url().includes(SINGER_WASM)) singerDownloads.push(request.url())
  })
})

Given(
  'a score whose soprano part sings its lyrics is loaded',
  async ({ page }) => {
    await page.addInitScript((source) => {
      localStorage.setItem(
        'poentaro:files:v1',
        JSON.stringify({
          active: 'test.poentaro',
          userFiles: { 'test.poentaro': source },
          bin: {},
          fileIds: { 'test.poentaro': crypto.randomUUID() },
        }),
      )
    }, SINGING_SOURCE)
    await page.goto('/')
    await page.waitForSelector('.monaco-editor .view-lines', {
      timeout: 15_000,
    })
    await page.waitForSelector('.preview-page', { timeout: 15_000 })
  },
)

Then('the singing voice was downloaded', async () => {
  expect(singerDownloads).not.toHaveLength(0)
})

Then('the singing voice was never downloaded', async () => {
  expect(singerDownloads).toHaveLength(0)
})

Then(
  'the Play All button eventually shows the playing state',
  async ({ page }) => {
    // Play waits while the voice is sung, so this allows more time than the
    // instrumental `the Play All button shows the playing state`.
    await expect(page.locator('button.play-all-btn')).toHaveClass(
      /play-all-btn--playing/,
      { timeout: 50_000 },
    )
  },
)
