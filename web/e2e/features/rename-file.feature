Feature: Rename file via the local storage backend

  Scenario: SVG preview persists after renaming the active file
    Given a local-storage-backed file "original.poentaro" is seeded for renaming
    When the app loads the seeded rename test file
    And I double-click the active tab name to enter rename mode
    And I fill the active tab's rename input with "renamed" and press Enter
    Then the active tab shows the name "renamed"
    And the file switcher trigger shows the renamed file "renamed"
    And the SVG preview is still visible without any manual edits
