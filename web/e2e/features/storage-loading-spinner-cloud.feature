Feature: Storage settings modal loading spinner when switching to the cloud backend

  Scenario: Switching to Cloud storage shows a loading spinner while files load
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And a cloud file named "backend-switch-loading.poentaro" is seeded for the signed-in account
    And the cloud file-list request is delayed by 1 second when switching backend
    When the app loads on the local backend with the editor ready
    And I open the storage settings modal to switch backend
    And I select the "Cloud storage" storage option
    Then the cloud loading spinner is visible
    And the cloud loading spinner disappears once loading finishes
    And the storage settings modal shows connected as "e2e-test-user@poentaro.test"
