import { expect } from '@playwright/test'
import { Given, Then } from './fixtures'

const SINGING_SOURCE = [
  'parts [',
  '    "Melody" [M]: notes,',
  '    "Verse" [v]: lyrics [M] { voice: female },',
  '],',
  '',
  'score """',
  '    time=4/4 key=C4 bpm=120',
  '    [M] 1 2 3 4',
  '    [v] 记 得 我 们',
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
  'a score whose lyric part sings in a female voice is loaded',
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
