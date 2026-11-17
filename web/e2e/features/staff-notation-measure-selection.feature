Feature: Selecting measures in the staff notation view

  # The staff view selects by measure, through the editor's selection like
  # the numbered view: a section button or a click-and-click on the staff
  # pages sets the editor's selection, and the staff view highlights
  # whatever measures that selection covers.

  Background:
    Given a long two-section score with measure-audio capture is shown in the staff notation view

  Scenario: Clicking a section button highlights its measures and scrolls to them
    When I click the section jump button labeled "B"
    Then the section jump selected measure range is "24-25"
    And the highlighted staff measures are 24 to 25
    And staff measure 25 is scrolled into view

  Scenario: Hovering a staff measure tints it and shows a pointer
    When I hover staff measure 2
    Then staff measure 2 shows the hover state
    And staff measure 3 does not show the hover state

  Scenario: Click-and-click on two staff measures selects the range between them
    When I click staff measure 1
    Then the pending-second-click banner is shown in the staff view
    When I click staff measure 3
    Then the section jump selected measure range is "1-3"
    And the highlighted staff measures are 1 to 3

  Scenario: Hovering between the two clicks previews the range, and Escape cancels it
    When I click staff measure 1
    And I hover staff measure 4
    Then the highlighted staff measures are 1 to 4
    When I press Escape
    Then the highlighted staff measures are 1 to 1
    And the section jump selected measure range is "1-1"

  Scenario: Playing after a staff selection plays exactly the selected measures
    When I click staff measure 1
    And I click staff measure 2
    Then the section jump selected measure range is "1-2"
    And the play-measure button becomes enabled once the soundfont loads
    When I click the play-measure button
    Then the requested measure audio covers measures 1 to 2
