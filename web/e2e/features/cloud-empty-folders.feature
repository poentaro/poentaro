Feature: Empty folders in the cloud are kept as hidden placeholder files

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: A new empty folder is stored as a placeholder and survives a reload
    When the app loads the cloud-backed file list
    And I create a new folder from the file actions menu
    And I name the new folder "CloudEmptyA"
    And I open the file tree list
    Then "CloudEmptyA/.keep" exists on the server for the signed-in account
    When I reload the page and reopen the file list
    Then the file tree shows the folder "CloudEmptyA"
    And the file tree does not show the file ".keep"
    And the bin does not list ".keep"

  Scenario: An empty folder made on another device appears and is not a file
    Given a cloud file named "CloudEmptyB/.keep" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I click the file tree folder "CloudEmptyB"
    Then the file tree shows the empty folder hint
    And the file tree does not show the file ".keep"

  Scenario: Deleting an empty folder removes it for good and leaves nothing in the bin
    Given a cloud file named "CloudEmptyC/.keep" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I choose "Delete folder" from the actions menu of the folder "CloudEmptyC"
    And I confirm the delete folder dialog
    Then the file tree does not show the folder "CloudEmptyC"
    When I reload the page and reopen the file list
    Then the file tree does not show the folder "CloudEmptyC"
    And the bin does not list ".keep"

  Scenario: Renaming an empty folder moves its placeholder
    Given a cloud file named "CloudEmptyD/.keep" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I choose "Rename" from the actions menu of the folder "CloudEmptyD"
    And I type "CloudEmptyD2" in the rename dialog and confirm
    Then "CloudEmptyD2/.keep" exists on the server for the signed-in account
    When I reload the page and reopen the file list
    Then the file tree shows the folder "CloudEmptyD2"
    And the file tree does not show the folder "CloudEmptyD"

  Scenario: A file cannot be given the reserved placeholder name
    Given a cloud file named "CloudEmptyE/song.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I open the cloud file "CloudEmptyE/song" from the file list
    And I type "CloudEmptyE/.keep" in the active file's rename input in the list
    Then the list rename input says "reserved name"
