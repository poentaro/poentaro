Feature: Import a shared score via the cloud storage backend

  Scenario: Importing a shared score persists via the cloud storage backend
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And the cloud storage backend preference is set
    When I navigate to the share URL for "shared-test.poentaro"
    Then the shared-preview banner is visible
    When I click the "Import this score" button
    Then the active tab becomes "shared-test"
    And the shared-preview banner is gone
    When I reload the page and reopen the file list
    Then the file list shows "shared-test" after reload
