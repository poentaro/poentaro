Feature: File switcher loading spinner for the cloud storage backend

  Scenario: Header file switcher shows a loading spinner while cloud files load
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And a cloud file named "loading.poentaro" is seeded for the signed-in account
    And the cloud file-list request is delayed by 5 seconds for the file switcher
    When the app loads with the editor ready while cloud files load
    Then the file switcher trigger shows a loading spinner
    And opening the file list shows the hint "Loading files from the cloud…"
    When the cloud file-list request resolves
    Then the file switcher trigger spinner is gone and the caret is visible
    And the file list shows "loading"
