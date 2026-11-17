Feature: The file tree highlights the folders holding the hovered row

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.poentaro", "Hymns/Amazing.poentaro" and "Scratch.poentaro" and an empty folder "Empty" with "Scratch.poentaro" active

  Scenario: Hovering a nested file highlights exactly its folders
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Hymns"
    And I click the file tree folder "Advent"
    And I hover the file tree file "O Come"
    Then the file tree highlights the ancestor folders "Hymns, Advent"

  Scenario: Hovering a nested folder highlights exactly its folders
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Hymns"
    And I hover the file tree folder "Advent"
    Then the file tree highlights the ancestor folders "Hymns"

  Scenario: Without a hovered row, the active file's folders are highlighted
    Given the file tree active file is "Hymns/Advent/O Come.poentaro"
    When the file tree app loads
    And I open the file tree list
    And I hover the file tree file "Scratch"
    Then the file tree highlights no ancestor folders
    When I move the pointer off the file tree
    Then the file tree highlights the ancestor folders "Hymns, Advent"

  Scenario: Moving between rows updates the highlight
    When the file tree app loads
    And I open the file tree list
    And I click the file tree folder "Hymns"
    And I click the file tree folder "Advent"
    And I hover the file tree file "O Come"
    Then the file tree highlights the ancestor folders "Hymns, Advent"
    When I hover the file tree file "Amazing"
    Then the file tree highlights the ancestor folders "Hymns"
