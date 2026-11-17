Feature: Signing in for cloud storage

  Scenario: Signing in switches to cloud storage automatically
    Given an account exists for "e2e-sign-in@poentaro.test"
    When the app loads for the account sign-in flow
    And I open the storage settings modal for sign-in
    And I select the "Cloud storage" storage option
    And I click "Sign in" in the storage settings modal
    And I enter the email "e2e-sign-in@poentaro.test" in the account dialog
    And I enter the e2e password in the account dialog
    And I click "Sign in" in the account dialog
    Then the storage settings modal shows connected as "e2e-sign-in@poentaro.test"
    And the "Cloud storage" storage option is checked
    And the stored storage-backend preference is set to cloud

  Scenario: Signing out reverts to the local backend and stops saving to the cloud
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And a cloud file named "song.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list for disconnect
    And I select the "song" tab before disconnecting
    When I click the account chip
    And I click "Sign out" in the profile popover
    Then the stored account auth is cleared
    When I open the storage settings modal for sign-in
    Then the app no longer shows as connected
    And the "This browser" storage option is checked
    When I close the storage settings modal and attempt to edit and force-save
    Then no content request is sent to the worker after disconnecting
