Feature: Measure highlight

  Background:
    Given the app is loaded and the editor is focused

  Scenario: Renders amber highlight rect when cursor is inside a measure
    When I jump to line 7
    Then the measure highlight rect is visible

  Scenario: Removes highlight rect when cursor moves outside all measures
    When I jump to line 7
    Then the measure highlight rect is visible
    When I jump to line 1
    Then the measure highlight rect is not visible

  Scenario: Hides the highlight rect while text is selected, and restores it once the selection collapses back to a caret
    When I jump to line 7
    Then the measure highlight rect is visible
    When I select the whole current line
    Then the measure highlight rect is not visible
    When I collapse the selection back to a caret at the end of the line
    Then the measure highlight rect is visible
