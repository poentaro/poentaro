Feature: Tab switch force-saves unobserved edits on the local storage backend

  Scenario: Switching tabs on the local backend never loses an edit that has not been observed via the debounce timer
    Given local files "a.poentaro" and "b.poentaro" are seeded for tab-switch force-save
    And the clock is installed and never advanced before the tab switch
    When the app loads the tab-switch force-save test files
    And I type " 5" at the end of the editor before switching tabs
    Then the stored file "a.poentaro" contains "1 2 3 4 5"
    When I switch the active tab to "b"
    Then the "b" tab becomes the current tab
    And the editor view-lines contain "5 6 7 1"
    And the stored file "a.poentaro" is unchanged from before the tab switch
    When I switch the active tab to "a"
    Then the editor view-lines contain "1 2 3 4 5"
