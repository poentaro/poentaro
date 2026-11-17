Feature: Autosave on the local storage backend

  Scenario: Editing a file persists to local storage without waiting out the autosave debounce
    Given a local-storage-backed file "auto.poentaro" is seeded
    And the clock is installed and never advanced
    When the app loads the local-storage-backed file
    And I type " 5" at the end of the editor
    Then the stored source for the active file contains "1 2 3 4 5"
    And no save-status badge appears
