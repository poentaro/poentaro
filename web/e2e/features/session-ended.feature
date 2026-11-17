Feature: A session that ends while the app is open

  # A session can end while a tab still holds it: a password change or a
  # sign-out on another device ends every other session. Whichever request
  # the worker refuses first signs this browser out, so the header offers
  # "Sign in" instead of a chip for a session that no longer exists.

  Scenario: A save refused because the session ended signs this browser out
    Given an account is signed in as "e2e-session-ended@poentaro.test"
    And a cloud file named "session-ended.poentaro" is seeded for the signed-in account
    And I open "session-ended" from the cloud file list
    When the session is ended on the worker
    And I append " 5" to the editor and save
    Then the header shows a "Sign in" button and no account chip

  Scenario: The edit made after the session ended is saved once signed in again
    Given an account is signed in as "e2e-session-ended@poentaro.test"
    And a cloud file named "session-ended-resume.poentaro" is seeded for the signed-in account
    And I open "session-ended-resume" from the cloud file list
    When the session is ended on the worker
    And I append " 5" to the editor and save
    And I click "Sign in" in the header
    And I enter the email "e2e-session-ended@poentaro.test" in the account dialog
    And I enter the e2e password in the account dialog
    And I click "Sign in" in the account dialog
    Then the cloud file "session-ended-resume.poentaro" contains "1 2 3 4 5"

  Scenario: A share overview refused because the session ended signs this browser out
    Given an account is signed in as "e2e-session-ended@poentaro.test"
    And the worker refuses the share overview as unauthorized
    When the app loads with cloud storage
    Then the header shows a "Sign in" button and no account chip
