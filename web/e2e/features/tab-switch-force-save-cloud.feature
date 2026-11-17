Feature: Tab switch force-save via the cloud storage backend

  Scenario: Switching the active file tab force-flushes a pending debounced cloud save
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And a cloud file named "a.poentaro" is seeded for the signed-in account
    And a cloud file named "b.poentaro" with content "5 6 7 1" is seeded for the signed-in account
    And a fake clock is installed to prevent an autosave race with the tab switch
    When the app loads the cloud-backed file list for a tab-switch save
    And I select the "a" tab to test the tab switch
    And I append " 5" to the editor to trigger a tab-switch save
    Then no content request has been sent yet before the tab switch
    And the tab-switch status badge shows "Unsaved"
    When I switch to the "b" tab from the file list
    Then the content save lands for "a.poentaro" containing "1 2 3 4 5"
    When I switch to the "a" tab from the file list
    And I reload the page and reopen the file list
    Then the file list shows "a" after reload
    And the reloaded editor still contains the tab-switch-saved edit "1 2 3 4 5"
