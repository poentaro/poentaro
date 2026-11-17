Feature: Email and password accounts

  # Accounts are an email and a password, kept by the worker. Every scenario
  # uses its own account (see `e2e/e2eAccounts.ts`), so parallel scenarios
  # never share one, and resets it first when the scenario changes it in a
  # way that would carry over to the next run.

  Scenario: Creating an account signs this browser in
    Given the account "e2e-sign-up@poentaro.test" does not exist
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I switch the account dialog to "Create one"
    And I enter the email "e2e-sign-up@poentaro.test" in the account dialog
    And I enter the new password "a-long-enough-password" twice in the account dialog
    And I click "Create account" in the account dialog
    Then the account dialog is closed
    And the header shows the account chip labeled "e2e-sign-up@poentaro.test"

  Scenario: Creating an account with an email that already has one offers to sign in instead
    Given an account exists for "e2e-sign-in@poentaro.test"
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I switch the account dialog to "Create one"
    And I enter the email "e2e-sign-in@poentaro.test" in the account dialog
    And I enter the new password "another-long-password" twice in the account dialog
    And I click "Create account" in the account dialog
    Then the account dialog shows the error "An account with this email already exists."
    When I click "Sign in instead" in the account dialog
    Then the account dialog offers to sign in as "e2e-sign-in@poentaro.test"

  Scenario: A password shorter than 12 characters is refused
    Given the account "e2e-weak-password@poentaro.test" does not exist
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I switch the account dialog to "Create one"
    And I enter the email "e2e-weak-password@poentaro.test" in the account dialog
    And I enter the new password "too-short" twice in the account dialog
    And I click "Create account" in the account dialog
    Then the account dialog shows the error "at least 12 characters"
    And the header shows a "Sign in" button and no account chip

  Scenario: Signing in with the right password signs this browser in
    Given an account exists for "e2e-sign-in@poentaro.test"
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I enter the email "e2e-sign-in@poentaro.test" in the account dialog
    And I enter the e2e password in the account dialog
    And I click "Sign in" in the account dialog
    Then the account dialog is closed
    And the header shows the account chip labeled "e2e-sign-in@poentaro.test"

  Scenario: A wrong password and an unknown email get the same answer
    Given the account "e2e-wrong-password@poentaro.test" is reset
    And the account "e2e-nobody@poentaro.test" does not exist
    And an account exists for "e2e-wrong-password@poentaro.test"
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I enter the email "e2e-wrong-password@poentaro.test" in the account dialog
    And I enter the password "not-the-right-password" in the account dialog
    And I click "Sign in" in the account dialog
    Then the account dialog shows the error "Incorrect email or password."
    When I enter the email "e2e-nobody@poentaro.test" in the account dialog
    And I click "Sign in" in the account dialog
    Then the account dialog shows the error "Incorrect email or password."
    And the header shows a "Sign in" button and no account chip

  Scenario: Five wrong passwords pause sign-in for that email
    Given the account "e2e-lockout@poentaro.test" is reset
    And an account exists for "e2e-lockout@poentaro.test"
    When the app loads for the account flow
    And I click "Sign in" in the header
    And I enter the email "e2e-lockout@poentaro.test" in the account dialog
    And I sign in with the password "not-the-right-password" 5 times in the account dialog
    Then the account dialog says sign-in is paused for this email
    And the account dialog's sign-in button is disabled
    And the worker refuses the right password for "e2e-lockout@poentaro.test" while it is locked

  Scenario: Changing the password signs the account out on its other devices
    Given the account "e2e-change-password@poentaro.test" is reset
    And "e2e-change-password@poentaro.test" is signed in on another device
    And an account is signed in as "e2e-change-password@poentaro.test"
    When the app loads for the account flow
    And I click the account chip
    And I click "Change password" in the profile popover
    And I change the password from the e2e password to "a-brand-new-password"
    Then the change password dialog is closed
    And the header shows the account chip labeled "e2e-change-password@poentaro.test"
    And the other device's session is no longer accepted
    And this browser's session is still accepted
    And "e2e-change-password@poentaro.test" can sign in with the password "a-brand-new-password"

  Scenario: Signing out ends the session on the worker too
    Given an account is signed in as "e2e-sign-out@poentaro.test"
    When the app loads for the account flow
    And I click the account chip
    And I click "Sign out" in the profile popover
    Then the header shows a "Sign in" button and no account chip
    And the signed-out session is no longer accepted
