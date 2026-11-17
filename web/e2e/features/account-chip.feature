Feature: Account chip in the header

  Scenario: Signed out, the header's Sign in button opens the account dialog
    Given an account exists for "e2e-sign-in@poentaro.test"
    When the app loads for the account chip flow
    Then the header shows a "Sign in" button and no account chip
    When I click "Sign in" in the header
    And I enter the email "e2e-sign-in@poentaro.test" in the account dialog
    And I enter the e2e password in the account dialog
    And I click "Sign in" in the account dialog
    Then the header shows the account chip labeled "e2e-sign-in@poentaro.test"
    And the header no longer shows a "Sign in" button

  Scenario: Signed in, opening the chip shows the email, Change password and Sign out
    Given an account is signed in as "e2e-test-user@poentaro.test"
    When the app loads for the account chip flow
    And I click the account chip
    Then the profile popover shows "e2e-test-user@poentaro.test"
    And the profile popover has a "Change password" button
    And the profile popover has a "Sign out" button

  Scenario: A file currently syncing shows a live indicator on the chip
    Given clipboard permissions are granted
    And the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    And the header account chip shows the syncing indicator

  Scenario: Signing out from the profile popover clears the account everywhere at once
    Given clipboard permissions are granted
    And the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When I close the share modal
    And I click the account chip
    And I click "Sign out" in the profile popover
    Then the header shows a "Sign in" button and no account chip
    And the signed-out session is no longer accepted
    When the owner reopens the share modal
    Then the sign-in prompt is shown
    When I close the share modal
    And I open the storage settings modal for sign-in
    Then the "This browser" storage option is checked
    And the storage settings modal hides the sign-in prompt
    When I select the "Cloud storage" storage option
    Then the storage settings modal shows the sign-in prompt

  Scenario: Signing out while cloud storage is active falls back to local storage and stops saving to the cloud
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And a cloud file named "song.poentaro" is seeded for the signed-in account
    And the stored storage-backend preference is "cloud"
    When the app loads the cloud-backed file list for the account chip flow
    And I select the "song" tab
    And I click the account chip
    And I click "Sign out" in the profile popover
    Then the stored account auth is cleared
    And the stored storage-backend preference is set to local
    When I edit and force-save
    Then no content request is sent to the worker after signing out
