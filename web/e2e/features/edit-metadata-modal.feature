Feature: Edit Metadata modal

  Scenario: CodeLens Edit Metadata link opens the modal
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    Then the edit metadata modal contains "Edit Metadata"
    And the first text input in the metadata modal has value "Test"

  Scenario: Editing the title field updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the title field with "New Title"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "title \"New Title\""

  Scenario: Editing a numeric field updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Row Height numeric field with "30"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "row height 30"

  Scenario: Clearing an optional field removes it from the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I clear the second text input in the metadata modal
    And I close the metadata modal with Escape
    Then the editor source and stored source no longer contain "subtitle"

  Scenario: Unchecking merge_duplicate_measures_across_parts writes = no to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck the merge_duplicate_measures_across_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "merge duplicate measures across parts: no"

  Scenario: Re-checking merge_duplicate_measures_across_parts writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck then re-check the merge_duplicate_measures_across_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "merge duplicate measures across parts: yes"

  Scenario: Unchecking hide_resting_parts writes = no to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck the hide_resting_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide resting parts: no"

  Scenario: Re-checking hide_resting_parts writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I uncheck then re-check the hide_resting_parts checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide resting parts: yes"

  Scenario: Checking hide_system_dividers writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I check the hide_system_dividers checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide system dividers: yes"

  Scenario: Unchecking hide_system_dividers writes = no to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I check then uncheck the hide_system_dividers checkbox
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "hide system dividers: no"

  Scenario: Editing the Part Label Width updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Part Label Width" field with "60"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "part label width pt 60"

  Scenario: Editing the Page Margin updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Page Margin" field with "40"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "page margin pt 40"

  Scenario: Editing the Measure Number Style's Font Size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Measure Number Style Font Size" field with "14"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "measure number style { font size 14 }"

  Scenario: Editing the Section Label Style's Font Size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Section Label Style Font Size" field with "16"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "section label style { font size 16 }"

  Scenario: Editing the Part Label Style's Font Size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Part Label Style Font Size" field with "18"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "part label style { font size 18 }"

  Scenario: Editing the Page Number Style's Font Size updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Page Number Style Font Size" field with "20"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "page number style { font size 20 }"

  Scenario: Editing the Lyrics Style's V. Padding updates the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Lyrics Style V. Padding" field with "20"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "lyrics style { vertical padding pt 20 }"

  Scenario: Choosing a style's Font Family writes its source keyword
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I choose "Sans Serif" in the "Title Style Font Family" dropdown
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "title style { font family: sans serif }"

  Scenario: Pressing a style's Underline toggle writes = yes to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I press the "Section Label Style Underline" toggle
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "section label style { underline: yes }"

  Scenario: Editing directive_row_offset writes "x y" to the source
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the directive_row_offset field with "0 12"
    And I close the metadata modal with Escape
    Then the editor source and stored source both contain "directive row offset (0, 12)"

  Scenario: Font Size placeholders follow an edited row_height
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the Row Height numeric field with "40"
    Then the "Title Style Font Size" field's placeholder is "60"
    And the "Lyrics Style Font Size" field's placeholder is "24"
    And the "Sequence Style Font Size" field's placeholder is "12"

  Scenario: Notes and chords Font Size placeholders follow an explicit lyrics font size
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I fill the "Lyrics Style Font Size" field with "20"
    Then the "Notes Style Font Size" field's placeholder is "20"
    And the "Chords Style Font Size" field's placeholder is "20"
    And the "Note Dash Style Font Size" field's placeholder is "20"

  Scenario: Modal stays within the editor pane and does not cover the preview pane
    Given the edit-metadata-modal test fixture is loaded with viewport 1400 by 900
    When I open the Edit Metadata modal
    Then the metadata modal stays within the editor pane and does not cover the preview pane

  Scenario: Preview pane stays scrollable while the modal is open
    Given a numbered score with 40 measures and row_height 200 is loaded, and the viewport is 1400 by 900
    Then the preview pane is scrollable
    When I open the Edit Metadata modal
    And I hover the preview pane and scroll the mouse wheel down by 400
    Then the preview pane scroll position is greater than 0

  Scenario: A style row's help spells out its row-height-relative Font Size default
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I open the help for the "Title Style" field
    Then the "Title Style" help contains "Font Size defaults to 150% of Row Height."

  Scenario: A style row's help names the style its Font Size default follows
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    And I open the help for the "Note Dash Style" field
    Then the "Note Dash Style" help contains "Font Size defaults to Notes Style's Font Size."

  Scenario: Bold and Italic are disabled for a font pinned to its single Regular face
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    Then the "Title Style Bold" toggle is disabled
    And the "Title Style Italic" toggle is disabled
    And the "Title Style Underline" toggle is enabled

  Scenario: Bold and Italic stay enabled for a font previewed with the system font
    Given the edit-metadata-modal test fixture is loaded
    When I open the Edit Metadata modal
    Then the "Notes Style Bold" toggle is enabled
    And the "Notes Style Italic" toggle is enabled
