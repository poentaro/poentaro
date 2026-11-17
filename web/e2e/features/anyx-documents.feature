Feature: Documents written in anyx expressions

  # The fixture has `#` comments between sections, a `//` comment inside the
  # score string, and two measures whose note lines are lines 10 and 13.

  Background:
    Given the anyx document fixture is loaded

  Scenario: The document renders without diagnostics
    Then the preview shows the anyx document's 2 measures
    And the editor has no anyx diagnostics

  Scenario: Section headers get their CodeLens links
    Then the editor shows 1 "Edit Parts" CodeLens link
    And the editor shows 1 "Edit Metadata" CodeLens link

  Scenario: A caret in the score string finds its measure
    When I jump to line 13 and wait for the selection debounce
    Then the play-measure button shows "Measure 2"

  Scenario: Selecting a note line highlights its notes
    When I select the whole of line 13 in the anyx document
    Then 4 notes are range-selected in the anyx document

  Scenario: Clicking a measure in the preview selects its text in the score string
    When I Cmd/Ctrl-click measure 1 in the anyx document
    Then the editor selects "6 6 5 0 lit- tle star" in the anyx document

  Scenario: Editing metadata keeps the comments
    When I open the Edit Metadata modal
    And I fill the title field with "New Title"
    And I close the metadata modal with Escape
    Then the editor source contains 'title "New Title"'
    And the anyx document still has its comments

  Scenario: Editing a part keeps the comments
    When I click the "Edit Parts" CodeLens link
    And I change the octave select for part "M" to "-1"
    And I close the anyx document's edit parts modal
    Then the editor source contains "octave -1"
    And the anyx document still has its comments

  Scenario: Formatting keeps the comments and the anyx form
    When I click the format-score toolbar toggle
    Then the editor source contains 'metadata {'
    And the editor source contains 'score """'
    And the anyx document still has its comments

  Scenario: A mistake in the score is reported on its own line
    When I replace "6 6 5 0" with "6 6 5 ?" in the anyx document
    Then the editor has an anyx diagnostic on line 13

  Scenario: A mistake outside the score keeps the preview
    When I replace "[M]: notes" with "[M]: nots" in the anyx document
    Then the editor has an anyx diagnostic on line 5
    And the preview still shows a page for the anyx document
