Feature: File operations on the cloud (D1) storage backend

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: Creating a file persists it to the cloud backend
    When the app loads the cloud backend with no files
    And I click the "New" button to create a file
    Then the active tab becomes "untitled"
    When I reload the page and reopen the file list
    Then the file list shows "untitled" after reload

  Scenario: Renaming a file persists the new name to the cloud backend
    Given a cloud file named "before-rename.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I rename the active file to "renamed"
    Then the active tab becomes "renamed"
    When I reload the page and reopen the file list
    Then the file list shows "renamed" after reload
    And the file list does not show "before-rename"

  Scenario: Deleting a file moves it to the bin and it survives reload
    Given a cloud file named "todelete.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I choose Delete from the file actions menu of the active file
    And I confirm the delete file dialog
    Then the file list no longer shows "todelete"
    And the bin shows "todelete"
    When I reload the page and reopen the file list
    Then the bin shows "todelete" after reload

  Scenario: Delete asks for confirmation before moving the file to the bin
    Given a cloud file named "askfirst.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I choose Delete from the file actions menu of the active file
    Then the delete file dialog is titled 'Delete file "askfirst.poentaro"?'
    When I cancel the delete file dialog
    Then the file list still shows "askfirst"

  Scenario: Cancelling the delete file dialog keeps the file
    Given a cloud file named "keepme.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I choose Delete from the file actions menu of the active file
    And I cancel the delete file dialog
    Then the file list still shows "keepme"
    And the bin does not show "keepme"

  Scenario: Duplicating a file creates a second cloud-backed file with the same content
    Given a cloud file named "source.poentaro" with content "1 2 3" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I duplicate the active file
    Then the active tab becomes "source 2"
    When I reload the page and reopen the file list
    Then the editor for "source 2" contains "1 2 3"

  Scenario: Restoring a file that collides with an active file renames it via the cloud backend
    Given the signed-in account has "original.poentaro" active and "original.poentaro" binned via a separate delete, for a restore collision
    When the app loads the cloud-backed file list for the collision test
    And I open the bin
    And I click the restore button for "original.poentaro"
    Then a "original 2" tab appears within 5 seconds
    And both "original" and "original 2" tabs exist exactly once each
    When I reload the page after the collision restore
    Then both "original" and "original 2" tabs exist exactly once each after reload

  Scenario: Editing a file schedules a debounced autosave to the cloud storage backend
    Given a cloud file named "auto.poentaro" with content "1 2 3 4" is seeded for the signed-in account
    And a fake clock is installed before navigating to test autosave
    When the app loads the cloud-backed file list for autosave
    And I select the "auto" tab to test autosave
    And I append " 5" to the editor to trigger an autosave
    Then no content request has been sent yet for the debounced edit
    And the autosave status badge shows "Unsaved (autosaving in"
    When I fast-forward the clock past the autosave debounce interval to trigger it
    Then the content save lands for "auto.poentaro" containing "1 2 3 4 5"
    And the autosave status badge shows "Saved"
    When I reload the page after the autosave
    Then the reloaded editor still contains the autosaved edit "1 2 3 4 5"
