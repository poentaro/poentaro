Feature: Section header CodeLens links
  The Edit Parts / Edit Metadata links appear exactly on the lines the parser
  treats as `parts` / `metadata` sections.

  Scenario: A section after a comment still gets its link
    Given the section-header CodeLens fixture with comments is loaded
    Then the editor shows 1 "Edit Parts" CodeLens link
    And the editor shows 1 "Edit Metadata" CodeLens link
    When I click the "Edit Parts" CodeLens link
    Then the edit parts modal contains "Melody"

  Scenario: A parts-like line inside the score string gets no link
    Given the section-header CodeLens fixture with a parts-like score line is loaded
    Then the editor shows 1 "Edit Parts" CodeLens link
