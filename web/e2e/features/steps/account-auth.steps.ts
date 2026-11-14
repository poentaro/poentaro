import { expect, type Page } from '@playwright/test'
import {
  E2E_PASSWORD,
  isSessionTokenAccepted,
  resetE2eAccounts,
  signInE2eAccount,
} from '../../e2eAccounts.ts'
import { answersCall, publicClient, valueOrFail } from '../../protocolClients'
import { lastSeededSessionToken } from './cloud-account-helpers'
import { BeforeScenario, Given, Then, When } from './fixtures'

// Covers the account dialog (sign in / create account), the change
// password dialog and signing out (account-auth.feature). The account
// dialog steps are shared with every feature that signs in through the UI.

// The session of "another device" in the change password scenario: a
// sign-in made straight against the worker, never seen by the page.
let otherDeviceToken: string | null = null

BeforeScenario(async () => {
  otherDeviceToken = null
})

function accountDialog(page: Page) {
  return page.getByTestId('account-auth-modal')
}

function changePasswordDialog(page: Page) {
  return page.getByTestId('change-password-modal')
}

Given('the account {string} does not exist', async ({}, email: string) => {
  await resetE2eAccounts([email])
})

Given('the account {string} is reset', async ({}, email: string) => {
  await resetE2eAccounts([email])
})

Given('an account exists for {string}', async ({}, email: string) => {
  await signInE2eAccount(email)
})

Given('{string} is signed in on another device', async ({}, email: string) => {
  otherDeviceToken = await signInE2eAccount(email)
})

When('the app loads for the account flow', async ({ page }) => {
  await page.goto('/')
})

When(
  'I switch the account dialog to {string}',
  async ({ page }, label: string) => {
    await accountDialog(page)
      .getByRole('button', { name: label, exact: true })
      .click()
  },
)

When(
  'I enter the email {string} in the account dialog',
  async ({ page }, email: string) => {
    await accountDialog(page).getByLabel('Email', { exact: true }).fill(email)
  },
)

When(
  'I enter the password {string} in the account dialog',
  async ({ page }, password: string) => {
    await accountDialog(page)
      .getByLabel('Password', { exact: true })
      .fill(password)
  },
)

When('I enter the e2e password in the account dialog', async ({ page }) => {
  await accountDialog(page)
    .getByLabel('Password', { exact: true })
    .fill(E2E_PASSWORD)
})

When(
  'I enter the new password {string} twice in the account dialog',
  async ({ page }, password: string) => {
    const dialog = accountDialog(page)
    await dialog.getByLabel('Password', { exact: true }).fill(password)
    await dialog.getByLabel('Confirm password', { exact: true }).fill(password)
  },
)

When(
  'I click {string} in the account dialog',
  async ({ page }, label: string) => {
    await accountDialog(page)
      .getByRole('button', { name: label, exact: true })
      .click()
  },
)

When(
  'I sign in with the password {string} {int} times in the account dialog',
  async ({ page }, password: string, times: number) => {
    const dialog = accountDialog(page)
    for (let attempt = 0; attempt < times; attempt++) {
      // A refused attempt clears the field, so each one refills it. One at
      // a time: each must reach the worker before the next starts.
      await dialog.getByLabel('Password', { exact: true }).fill(password)
      await Promise.all([
        page.waitForResponse((response) =>
          answersCall(response, 'account/sign in'),
        ),
        dialog.getByRole('button', { name: 'Sign in', exact: true }).click(),
      ])
    }
  },
)

Then('the account dialog is closed', async ({ page }) => {
  await expect(accountDialog(page)).toHaveCount(0)
})

Then(
  'the account dialog shows the error {string}',
  async ({ page }, text: string) => {
    await expect(accountDialog(page).getByRole('alert')).toContainText(text)
  },
)

Then(
  'the account dialog offers to sign in as {string}',
  async ({ page }, email: string) => {
    const dialog = accountDialog(page)
    await expect(
      dialog.getByRole('button', { name: 'Sign in', exact: true }),
    ).toBeVisible()
    await expect(dialog.getByLabel('Confirm password')).toHaveCount(0)
    await expect(dialog.getByLabel('Email', { exact: true })).toHaveValue(email)
  },
)

Then(
  'the account dialog says sign-in is paused for this email',
  async ({ page }) => {
    await expect(accountDialog(page).getByRole('alert')).toContainText(
      /Too many attempts for this email\. Try again in \d+:\d\d\./,
    )
  },
)

Then("the account dialog's sign-in button is disabled", async ({ page }) => {
  await expect(
    accountDialog(page).getByRole('button', { name: 'Sign in', exact: true }),
  ).toBeDisabled()
})

Then(
  'the worker refuses the right password for {string} while it is locked',
  async ({}, email: string) => {
    const outcome = await (await publicClient()).account.signInPassword(
      email,
      E2E_PASSWORD,
    )
    expect(outcome).toMatchObject({
      kind: 'error',
      error: { kind: 'lockedForSeconds' },
    })
  },
)

When(
  'I change the password from the e2e password to {string}',
  async ({ page }, password: string) => {
    const dialog = changePasswordDialog(page)
    await dialog
      .getByLabel('Current password', { exact: true })
      .fill(E2E_PASSWORD)
    await dialog.getByLabel('New password', { exact: true }).fill(password)
    await dialog
      .getByLabel('Confirm new password', { exact: true })
      .fill(password)
    await dialog
      .getByRole('button', { name: 'Change password', exact: true })
      .click()
  },
)

Then('the change password dialog is closed', async ({ page }) => {
  await expect(changePasswordDialog(page)).toHaveCount(0)
})

Then("the other device's session is no longer accepted", async () => {
  if (!otherDeviceToken) throw new Error('no other device signed in')
  expect(await isSessionTokenAccepted(otherDeviceToken)).toBe(false)
})

Then("this browser's session is still accepted", async () => {
  expect(await isSessionTokenAccepted(lastSeededSessionToken())).toBe(true)
})

Then('the signed-out session is no longer accepted', async () => {
  const token = lastSeededSessionToken()
  // Sign-out tells the worker without waiting for it, so give it a moment.
  await expect.poll(() => isSessionTokenAccepted(token)).toBe(false)
})

Then(
  '{string} can sign in with the password {string}',
  async ({}, email: string, password: string) => {
    const session = await valueOrFail(
      `sign-in for ${email}`,
      (await publicClient()).account.signInPassword(email, password),
    )
    expect(session.email).toBe(email)
  },
)

Then(
  'the profile popover has a {string} button',
  async ({ page }, label: string) => {
    await expect(
      page
        .getByTestId('account-popover')
        .getByRole('button', { name: label, exact: true }),
    ).toBeVisible()
  },
)
