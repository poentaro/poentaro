import { expect } from '@playwright/test'
import { BeforeScenario, Then, When } from './fixtures'

// Covers request-trace-id.feature: the trace ID the worker answers a failed
// request with is the one the app shows.

const TRACE_ID_PATTERN = /^[0-9a-f]{16}$/

let recordedTraceId: string | null = null

BeforeScenario(async () => {
  recordedTraceId = null
})

When(
  "I sign in with the password {string} and record the worker's trace ID",
  async ({ page }, password: string) => {
    const dialog = page.getByTestId('account-auth-modal')
    await dialog.getByLabel('Password', { exact: true }).fill(password)
    const [response] = await Promise.all([
      page.waitForResponse((response) =>
        response.url().endsWith('/auth/sign-in'),
      ),
      dialog.getByRole('button', { name: 'Sign in', exact: true }).click(),
    ])
    expect(response.status()).toBe(401)
    const traceId = response.headers()['x-trace-id'] ?? null
    expect(traceId).toMatch(TRACE_ID_PATTERN)
    expect((await response.json()).traceId).toBe(traceId)
    recordedTraceId = traceId
  },
)

Then('the account dialog shows the recorded trace ID', async ({ page }) => {
  await expect(
    page.getByTestId('account-auth-modal').getByTestId('trace-id'),
  ).toContainText(recordedTraceId ?? 'no trace ID recorded')
})
