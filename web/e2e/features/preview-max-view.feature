Feature: Preview max view

  Background:
    Given a score several A4 pages long is loaded for the max view

  Scenario: Max view lays the score out on one page as wide as the preview
    Then the preview shows more than 1 page
    When I switch the preview to "max" view
    Then the preview shows 1 page as wide as the preview

  Scenario: Max view re-wraps the score when the preview narrows
    Given the window is 1600 pixels wide
    When I switch the preview to "max" view
    And I count the preview's systems
    And the window narrows to 900 pixels
    Then the preview shows more systems than it counted

  Scenario: Max view is remembered after a reload
    When I switch the preview to "max" view
    And I reload the page
    Then the page view toggle reads "Max"
    When I switch the preview to "pages" view
    Then the preview shows more than 1 page
