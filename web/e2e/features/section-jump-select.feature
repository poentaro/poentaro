Feature: Section jump select

  Background:
    Given a two-section source with sections "A" and "B" is loaded

  Scenario: Clicking section A button focuses the editor
    When I click the section jump button labeled "A"
    Then the Monaco editor gains focus

  Scenario: Clicking section A button selects measures 0-1
    When I click the section jump button labeled "A"
    Then the section jump selected measure range is "0-1"

  Scenario: Clicking section B button selects measures 2-3
    When I click the section jump button labeled "B"
    Then the section jump selected measure range is "2-3"

  Scenario: Clicking section A button highlights lines 6-9 in the Monaco editor
    When I click the section jump button labeled "A"
    Then the section jump selected measure range is "0-1"
    And the section jump Monaco selection spans lines 6 to 9

  Scenario: Clicking section B button highlights lines 11-14 in the Monaco editor
    When I click the section jump button labeled "B"
    Then the section jump selected measure range is "2-3"
    And the section jump Monaco selection spans lines 11 to 14

  Scenario: Dragging from section A button to section B button selects the merged range
    When I drag from the section jump button labeled "A" to the one labeled "B"
    And I release the mouse button
    Then the section jump selected measure range is "0-3"
    And the section jump Monaco selection spans lines 6 to 14

  Scenario: Dragging from section B button to section A button selects the merged range
    When I drag from the section jump button labeled "B" to the one labeled "A"
    And I release the mouse button
    Then the section jump selected measure range is "0-3"
    And the section jump Monaco selection spans lines 6 to 14

  Scenario: Dragging between section buttons highlights them while dragging
    When I drag from the section jump button labeled "A" to the one labeled "B"
    Then the section jump buttons labeled "A" and "B" are both highlighted as dragging
    When I release the mouse button
    Then the section jump buttons labeled "A" and "B" are both highlighted as dragging

  Scenario: Touch-dragging from section A button to section B button selects the merged range
    When I touch-drag from the section jump button labeled "A" to the one labeled "B"
    Then the section jump selected measure range is "0-3"
    And the section jump Monaco selection spans lines 6 to 14

  Scenario: Clicking section A label in the SVG preview highlights lines 6-9 in the Monaco editor
    When I click the section label "A" in the SVG preview
    Then the section jump selected measure range is "0-1"
    And the section jump Monaco selection spans lines 6 to 9

  Scenario: Clicking section B label in the SVG preview highlights lines 11-14 in the Monaco editor
    When I click the section label "B" in the SVG preview
    Then the section jump selected measure range is "2-3"
    And the section jump Monaco selection spans lines 11 to 14

  Scenario: Clicking a bar line then a section label in the SVG preview still jumps to that section
    When I click a bar line in the SVG preview
    And I click the section label "B" in the SVG preview
    Then the section jump selected measure range is "2-3"
    And the section jump Monaco selection spans lines 11 to 14
