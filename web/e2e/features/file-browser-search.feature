Feature: The file browser can be searched

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.poentaro", "Hymns/Amazing.poentaro" and "Scratch.poentaro" and an empty folder "Empty" with "Scratch.poentaro" active

  Scenario: Typing filters the tree and keeps the folders leading to a match
    When the file tree app loads
    And I open the file tree list
    And I search the file browser for "come"
    Then the file tree shows the folder "Hymns" with a count of 2
    And the file tree shows the file "O Come"
    And the file tree does not show the file "Amazing"
    And the file tree does not show the file "Scratch"

  Scenario: A search without a match says so
    When the file tree app loads
    And I open the file tree list
    And I search the file browser for "zzz"
    Then the file browser says no files match "zzz"

  Scenario: Searching for a folder name shows everything inside it
    When the file tree app loads
    And I open the file tree list
    And I search the file browser for "hymns"
    Then the file tree shows the folder "Hymns" with a count of 2
    And the file tree shows the file "Amazing"
    And the file tree does not show the file "Scratch"

  Scenario: Escape clears the search first and closes the browser second
    When the file tree app loads
    And I open the file tree list
    And I search the file browser for "come"
    And I press Escape in the file browser
    Then the file browser search box is empty
    And the file tree shows the file "Scratch"
    When I press Escape in the file browser
    Then the file browser is closed
