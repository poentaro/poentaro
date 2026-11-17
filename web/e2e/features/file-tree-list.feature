Feature: The file list is a collapsible folder tree

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.poentaro", "Hymns/Amazing.poentaro" and "Scratch.poentaro" and an empty folder "Empty" with "Scratch.poentaro" active

  Scenario: Folders start collapsed and show how many files they hold
    When the file tree app loads
    And I open the file tree list
    Then the file tree shows the folder "Hymns" with a count of 2
    And the file tree shows the folder "Empty" with a count of 0
    And the file tree shows the file "Scratch"
    And the file tree does not show the file "Amazing"

  Scenario: Expanding folders reveals nested files
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Hymns"
    Then the file tree shows the file "Amazing"
    And the file tree does not show the file "O Come"
    When I click the file tree folder "Advent"
    Then the file tree shows the file "O Come"
    When I click the file tree folder "Hymns"
    Then the file tree does not show the file "Amazing"

  Scenario: Selecting a nested file shows its folders as a muted prefix on the toolbar button
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Hymns"
    And I click the file tree folder "Advent"
    And I click the file tree file "O Come"
    Then the file switcher toolbar button shows the folder prefix "Hymns / Advent /" and the file name "O Come"

  Scenario: A flat file has no prefix on the toolbar button
    When the file tree app loads
    Then the file switcher toolbar button shows no folder prefix and the file name "Scratch"

  Scenario: Folders holding the active file are open when the list opens
    Given the file tree active file is "Hymns/Advent/O Come.poentaro"
    When the file tree app loads
    And I open the file tree list
    Then the file tree shows the file "O Come"
    And the file tree highlights the file "O Come"
    And the file tree shows the file "Amazing"

  Scenario: Expanded folders are remembered after a reload
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Hymns"
    And I reload the file tree app
    And I open the file tree list
    Then the file tree shows the file "Amazing"

  Scenario: An empty folder explains that it is empty
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Empty"
    Then the file tree shows the empty folder hint

  Scenario: Demo files are a read-only Demo row at the end of the list
    When the file tree app loads
    And I open the file tree list
    Then the file tree shows the Demo row last
    And the file tree does not show the demo file "01-pitches"
    When I click the file tree folder "Demo"
    And I click the file tree demo file "01-pitches"
    Then the file switcher toolbar button shows the folder prefix "Demo /" and the file name "01-pitches"
