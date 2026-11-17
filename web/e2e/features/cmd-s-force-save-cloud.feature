Feature: Cmd/Ctrl+S force-save via the cloud storage backend

  Scenario: Cmd/Ctrl+S force-flushes a pending debounced cloud save immediately
    Given an account is signed in as "e2e-test-user@poentaro.test"
    And a cloud file named "save.poentaro" is seeded for the signed-in account
    And a fake clock is installed to prevent an autosave race with force save
    When the app loads the cloud-backed file list for a force save
    And I select the "save" tab to test the force save
    And I append " 5" to the editor to trigger a force save
    Then no content request has been sent yet before the force save
    And the force-save status badge shows "Unsaved"
    When I press Cmd/Ctrl+S
    Then the content save lands for "save.poentaro" containing "1 2 3 4 5"
    When I reload the page and reopen the file list
    Then the file list shows "save" after reload
    And the reloaded editor still contains the force-saved edit "1 2 3 4 5"
