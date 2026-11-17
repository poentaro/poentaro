Feature: Undo/redo scoping across a file switch

  Scenario: An unsaved edit in file A survives switching to file B and back, and undo/redo only ever touch A's own edit
    Given local files "a.poentaro" and "b.poentaro" are seeded for undo-redo across file switch
    When the app loads the undo-redo file-switch test files
    And I type " 5" at the end of the editor to edit file A
    Then the stored file "a.poentaro" contains "1 2 3 4 5", as seen in undo redo across file switch
    When I switch the active tab to "b" without saving
    Then the "b" tab is the active tab
    And the editor view-lines show file B's content "5 6 7 1"
    When I switch the active tab back to "a"
    Then the "a" tab is the active tab
    And the editor view-lines show file A's edited content "1 2 3 4 5"
    When I focus the editor and press undo until file A's original content is restored
    Then the editor model value equals file A's original source
    And the stored file "a.poentaro" equals file A's original source
    And the stored file "b.poentaro" is untouched by A's undo
    When I press redo until file A's edit is restored
    Then the editor model value equals file A's edited source
    And the stored file "a.poentaro" equals file A's edited source
    And the stored file "b.poentaro" is untouched by A's redo
