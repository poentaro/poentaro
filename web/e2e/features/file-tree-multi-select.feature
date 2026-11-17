Feature: Selecting several files and folders in the file browser to rename, move or delete them

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.poentaro", "Hymns/Amazing.poentaro" and "Scratch.poentaro" and an empty folder "Empty" with "Scratch.poentaro" active
    And the file tree active file is "Hymns/Amazing.poentaro"

  Scenario: Select mode shows a selection bar that Done and Escape leave
    When the file tree app loads
    And I open the file tree list
    Then the file browser has no selection bar
    When I start selecting files
    Then the selection bar says "Nothing selected"
    When I click "Done" in the selection bar
    Then the file browser has no selection bar
    When I start selecting files
    And I press Escape in the file browser
    Then the file browser has no selection bar
    And the file tree shows the file "Scratch"

  Scenario: Clicking anywhere on a row ticks it
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I tick the file "Scratch"
    Then the selection bar says "1 selected"
    And the file "Scratch" is ticked
    When I tick the file "Scratch"
    Then the selection bar says "Nothing selected"
    And the file "Scratch" is not ticked

  Scenario: A folder ticks everything inside it and shows a mixed state when partly ticked
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I tick the file "Amazing"
    Then the checkbox of the folder "Hymns" is "mixed"
    When I tick the folder "Hymns"
    Then the selection bar says "2 selected"
    And the checkbox of the folder "Hymns" is "checked"
    When I tick the folder "Hymns"
    Then the selection bar says "Nothing selected"
    And the checkbox of the folder "Hymns" is "unchecked"

  Scenario: Select all and Clear
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I click "Select all" in the selection bar
    Then the selection bar says "3 selected"
    And the checkbox of the folder "Empty" is "unchecked"
    When I click "Clear" in the selection bar
    Then the selection bar says "Nothing selected"

  Scenario: Rename is only available for a single file or folder
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    Then the selection action "Move to…" is disabled
    And the selection action "Delete" is disabled
    When I tick the file "Scratch"
    Then the selection action "Rename" is enabled
    When I tick the file "Amazing"
    Then the selection action "Rename" is disabled
    And the selection action "Move to…" is enabled
    And the selection action "Delete" is enabled

  Scenario: Renaming the single ticked file
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I tick the file "Scratch"
    And I click "Rename" in the selection bar
    Then the rename dialog holds "Scratch"
    When I type "Renamed" in the rename dialog and confirm
    Then the file tree shows the file "Renamed"
    And the file tree does not show the file "Scratch"
    And the file browser has no selection bar

  Scenario: Moving the ticked files into a folder
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I tick the file "Scratch"
    And I tick the file "Amazing"
    And I click "Move to…" in the selection bar
    And I pick the folder "Empty" in the move dialog
    And I confirm the move dialog
    Then the file tree shows the folder "Empty" with a count of 2
    And the file browser has no selection bar

  Scenario: Moving a fully ticked folder moves it with its structure
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I tick the folder "Hymns"
    And I click "Move to…" in the selection bar
    And I pick the folder "Empty" in the move dialog
    And I confirm the move dialog
    Then the file tree shows the folder "Empty" with a count of 2
    And the file tree shows the folder "Hymns" nested 2 levels deep

  Scenario: A move that would clash is explained and cannot be confirmed
    Given the file tree also has the file "Empty/Scratch.poentaro"
    When the file tree app loads
    And I open the file tree list
    And I start selecting files
    And I tick the file "Scratch"
    And I click "Move to…" in the selection bar
    And I pick the folder "Empty" in the move dialog
    Then the move dialog says "already exists"
    And the move dialog cannot be confirmed

  Scenario: Deleting lists what goes to the bin and can be cancelled
    When the file tree app loads
    And I open the file tree list
    When I start selecting files
    And I tick the file "Scratch"
    And I tick the file "Amazing"
    And I click "Delete" in the selection bar
    Then the delete selection dialog lists "Hymns/Amazing.poentaro" and "Scratch.poentaro"
    When I cancel the delete selection dialog
    Then the selection bar says "2 selected"
    When I click "Delete" in the selection bar
    And I confirm the delete selection dialog
    Then the file tree does not show the file "Scratch"
    And the file tree does not show the file "Amazing"
    And the file browser has no selection bar
