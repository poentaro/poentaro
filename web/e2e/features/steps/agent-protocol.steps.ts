import { expect, type Locator, type Page } from '@playwright/test'
import createClient from 'openapi-fetch'
import type { paths } from '../../../src/generated/live-share-worker/schema'
import {
  CLOUD_WORKER_ORIGIN,
  seedCloudFile,
  seedTokenFor,
} from '../../cloudFileHelpers'
import { appClient, valueOrFail } from '../../protocolClients'
import { currentSignedInEmail } from './cloud-account.steps'
import { BeforeScenario, Given, Then, When } from './fixtures'

// `agent-protocol.feature`: the signed-in owner's API key, used from Node
// the way an AI agent uses it, by sending anyx requests to `POST /agent`.

const worker = createClient<paths>({ baseUrl: CLOUD_WORKER_ORIGIN })

interface CreatedKey {
  id: string
  secret: string
  /** What the AI agents dialog shows of the key. */
  prefix: string
}

interface AgentAnswer {
  status: number
  body: string
}

let key: CreatedKey | null = null
let lastAnswer: AgentAnswer | null = null
const firstSeenRevisions = new Map<string, number>()

BeforeScenario(() => {
  key = null
  lastAnswer = null
  firstSeenRevisions.clear()
})

/** The key in a prompt the worker wrote: what the agent saves to its key file. */
function keyInPrompt(prompt: string): string {
  const secret = /'(poe_[^'\s]+)'/.exec(prompt)?.[1]
  if (!secret) throw new Error(`found no key in the prompt ${prompt}`)
  return secret
}

async function sendAsAgent(request: string): Promise<void> {
  if (!key) throw new Error('no API key was created in this scenario')
  const response = await fetch(`${CLOUD_WORKER_ORIGIN}/agent`, {
    method: 'POST',
    headers: { Authorization: `Bearer ${key.secret}` },
    body: request,
  })
  lastAnswer = { status: response.status, body: await response.text() }
}

/** A valid score whose notes are `lastLine`. Not the seeded content, which
 * has errors the app tolerates but the agent's saves are rejected for. */
function withLastLine(lastLine: string): string {
  return [
    'parts ["Melody" [M]: notes],',
    '',
    'score """',
    `    [M] ${lastLine}`,
    '    """,',
  ].join('\n')
}

async function ownersFile(path: string) {
  const identityToken = await seedTokenFor(currentSignedInEmail())
  const { data } = await worker.POST('/files/list', {
    body: { identityToken },
  })
  const file = data?.files.find((listed) => listed.name === path)
  if (!file) throw new Error(`the signed-in account has no file ${path}`)
  return file
}

/** The revision the agent first saw `path` at in this scenario, so a later
 * write at it is a write at a stale revision. */
async function firstSeenRevision(path: string): Promise<number> {
  const seen = firstSeenRevisions.get(path)
  if (seen !== undefined) return seen
  const { revision } = await ownersFile(path)
  firstSeenRevisions.set(path, revision)
  return revision
}

async function writeAsAgent(path: string, content: string): Promise<void> {
  const revision = await firstSeenRevision(path)
  await sendAsAgent(
    `files/write ${JSON.stringify(path)} at revision ${revision} content ${JSON.stringify(content)}`,
  )
}

Given('the signed-in account has an API key', async () => {
  const identityToken = await seedTokenFor(currentSignedInEmail())
  const created = await valueOrFail(
    'creating an API key',
    (await appClient(identityToken)).keys.create(),
  )
  key = {
    id: created.key.id,
    secret: keyInPrompt(created.prompt),
    prefix: created.key.prefix,
  }
})

Given(
  'a cloud file named {string} is seeded for {string}',
  async ({}, name: string, email: string) => {
    await seedCloudFile(email, name)
  },
)

When(
  'the agent writes {string} at the revision it first saw with the last line {string}',
  async ({}, path: string, lastLine: string) => {
    await writeAsAgent(path, withLastLine(lastLine))
  },
)

When(
  'the agent writes {string} at the revision it first saw with the content {string}',
  async ({}, path: string, content: string) => {
    await writeAsAgent(path, content)
  },
)

When('the agent sends {string}', async ({}, request: string) => {
  await sendAsAgent(request)
})

Then("the agent's last result is not an error", async () => {
  expect(lastAnswer?.status).toBe(200)
  expect(lastAnswer?.body).not.toMatch(/ERROR|NOT RUN|INVALID REQUEST/)
})

Then(
  "the agent's last result starts with {string}",
  async ({}, start: string) => {
    expect(lastAnswer?.status).toBe(200)
    expect(
      lastAnswer?.body.trimStart().startsWith(start),
      lastAnswer?.body,
    ).toBe(true)
  },
)

Then("the agent's last result contains {string}", async ({}, part: string) => {
  expect(lastAnswer?.body).toContain(part)
})

Then('the agent is refused for its key', async () => {
  expect(lastAnswer?.status).toBe(401)
  expect(lastAnswer?.body).toContain('Bearer poe_')
})

When(
  'I click {string} in the AI agents dialog',
  async ({ page }, label: string) => {
    await page
      .getByTestId('api-keys-modal')
      .getByRole('button', { name: label })
      .click()
  },
)

Then(
  'the AI agents dialog shows a prompt that saves the new key and installs the Poentaro skill',
  async ({ page }) => {
    const prompt = page.getByTestId('api-key-prompt')
    await expect(prompt).toContainText("printf '%s' 'poe_")
    await expect(prompt).toContainText('~/.agents/skills/poentaro/SKILL.md')
    await expect(prompt).toContainText(`curl -s ${CLOUD_WORKER_ORIGIN}/agent`)
    await expect(prompt).toContainText('Start each session by sending `SCHEMA`')
  },
)

/** The dialog's row of this scenario's key; other scenarios' keys may
 * share the account. */
function rowOfTheAgentsKey(page: Page): Locator {
  if (!key) throw new Error('no API key was created in this scenario')
  return page
    .getByTestId('api-keys-modal')
    .getByTestId('api-key-row')
    .filter({ hasText: key.prefix })
}

When(
  "I click Revoke on the agent's key in the AI agents dialog",
  async ({ page }) => {
    await rowOfTheAgentsKey(page)
      .getByRole('button', { name: 'Revoke' })
      .click()
  },
)

When('I confirm revoking the key', async ({ page }) => {
  await page
    .getByTestId('revoke-api-key-dialog')
    .getByRole('button', { name: 'Revoke' })
    .click()
})

When('I cancel revoking the key', async ({ page }) => {
  const dialog = page.getByTestId('revoke-api-key-dialog')
  await dialog.getByRole('button', { name: 'Cancel' }).click()
  await expect(dialog).toHaveCount(0)
})

Then("the AI agents dialog lists the agent's key", async ({ page }) => {
  await expect(rowOfTheAgentsKey(page)).toHaveCount(1)
})

Then(
  "the AI agents dialog no longer lists the agent's key",
  async ({ page }) => {
    await expect(rowOfTheAgentsKey(page)).toHaveCount(0)
  },
)

Then(
  'the AI agents dialog lists the new key, expiring in 30 days',
  async ({ page }) => {
    // Formatted in the page, in the browser's locale, as the dialog does.
    const expires = await page.evaluate(() =>
      new Date(Date.now() + 30 * 24 * 60 * 60 * 1000).toLocaleDateString(),
    )
    await expect(page.getByTestId('api-key-row').first()).toContainText(
      `Never used · Expires ${expires}`,
    )
  },
)
