Feature: Part focus editor

  # Edits one part of the `score` as a single stream, one measure per line,
  # in place of the main editor. Every other part shows as read-only ghost
  # lines around each measure, and the stream is re-barred as you type.

  Background:
    Given the part stream editor fixture is loaded

  Scenario: Choosing a part replaces the editor with that part's stream
    When I choose part "M" from the Edit part as stream menu
    Then the part focus editor is shown in place of the main editor
    And the part focus header says "Editing Melody [M]"
    And the part focus stream is:
      """
      1 2 3 4
      5 6 7 1
      3 4 5 6
      """

  Scenario: Other parts appear as read-only ghost lines and labels as header rows
    When I choose part "M" from the Edit part as stream menu
    Then measure 1 shows the header row "Verse"
    And measure 2 shows the header row "Chorus"
    And measure 3 shows the header row "3"
    And measure 1 shows the ghost lines below "[W] a b c d" and "[B] 1, - 5, -"
    And measure 3 shows no ghost line for part "B"
    And the ghost lines cannot be selected or edited

  Scenario: Hiding a part removes its ghost lines
    When I choose part "M" from the Edit part as stream menu
    And I hide part "B" with its toggle
    Then no measure shows a ghost line for part "B"
    And measure 1 shows the ghost lines below "[W] a b c d"

  Scenario: Soloing a part shows only that part's ghost lines
    When I choose part "M" from the Edit part as stream menu
    And I solo part "B" with its toggle
    Then no measure shows a ghost line for part "W"
    And measure 1 shows the ghost lines below "[B] 1, - 5, -"

  Scenario: A note that overflows its measure moves to the next line as I type
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "0 " in the part focus editor
    Then the part focus stream is:
      """
      0 1 2 3
      4 5 6 7
      1 3 4 5
      6
      """
    And part focus line 4 shows the header row "4" and no ghost lines
    And the part focus status says "4 measures changed"

  Scenario: Deleting a note pulls the following notes back
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I press Delete in the part focus editor
    Then the part focus stream is:
      """
      2 3 4 5
      6 7 1 3
      4 5 6
      """

  Scenario: Deleting a note pulls a held note back across the barline
    Given the held note melody fixture is loaded instead
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I press Delete in the part focus editor
    Then the part focus stream is:
      """
      2 3 4 5 -
      6 7 1 2
      3 4
      """
    And the part focus editor has an error marker

  Scenario: The reformat undoes together with the keystroke
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "0" in the part focus editor
    And I press undo in the part focus editor
    Then the part focus stream is:
      """
      1 2 3 4
      5 6 7 1
      3 4 5 6
      """

  Scenario: A stream with an error is left as typed
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "(" in the part focus editor
    Then part focus line 1 is "(1 2 3 4"
    And the part focus editor has an error marker
    And the part focus Apply button is disabled

  Scenario: Pressing Enter mid-measure is undone by the formatter
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor after "1 2" on part focus line 1
    And I press Enter in the part focus editor
    Then the part focus stream is:
      """
      1 2 3 4
      5 6 7 1
      3 4 5 6
      """

  Scenario: A lyric stream is re-cut by its target's notes as I edit
    When I choose part "W" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I press Delete in the part focus editor
    Then the part focus stream is:
      """
      b c d e
      f g h i
      j k l
      """

  Scenario: Typing past the last lyric slot adds a measure on its own line
    When I choose part "W" from the Edit part as stream menu
    And I place the cursor at the end of the part focus stream
    And I type " extra" in the part focus editor
    Then the part focus stream is:
      """
      a b c d
      e f g h
      i j k l
      extra
      """
    And the part focus status says "1 measure changed"

  Scenario: Typing lyrics before the no-lyrics placeholders drops the ones pushed past the end
    Given the unwritten lyrics fixture is loaded instead
    When I choose part "W" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "a b c d " in the part focus editor
    Then the part focus stream is:
      """
      a b c d
      _
      _
      """
    When I apply the part focus editor
    Then the main editor source is the first measure lyrics source

  Scenario: The preview follows the draft as I type
    When I choose part "W" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "x" in the part focus editor
    Then the preview shows the text "xa"

  Scenario: The preview keeps the last good draft while the stream has an error
    When I choose part "W" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "x" in the part focus editor
    And I place the cursor at the end of the part focus stream
    And I type " @" in the part focus editor
    Then the part focus editor has an error marker
    And the preview shows the text "xa"

  Scenario: Cancel restores the preview to the original source
    When I choose part "W" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "x" in the part focus editor
    And the preview shows the text "xa"
    And I click Cancel in the part focus editor
    Then the preview shows the text "a"
    And the preview does not show the text "xa"

  Scenario: Apply writes the stream back and returns to the main editor
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "0 " in the part focus editor
    And I apply the part focus editor
    Then the main editor is shown
    And the main editor source is the shifted melody source

  Scenario: Apply is undoable from the main editor
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 1
    And I type "0 " in the part focus editor
    And I apply the part focus editor
    And I press undo in the main editor
    Then the main editor source is the original fixture source

  Scenario: Cancel returns to the main editor without changing the source
    When I choose part "M" from the Edit part as stream menu
    And I type "0 " in the part focus editor
    And I click Cancel in the part focus editor
    Then the main editor is shown
    And the main editor source is the original fixture source

  Scenario: Escape cancels the part focus editor
    When I choose part "M" from the Edit part as stream menu
    And I press Escape in the part focus editor
    Then the main editor is shown

  Scenario: An ineligible part is disabled in the menu with its reason
    Given the over-bracketed chord lyrics fixture is loaded instead
    When I open the Edit part as stream menu
    Then the menu item for part "C" is disabled
    And the menu item for part "C" explains "3 bracketed items for 2 chord attacks"
    And the menu item for part "H" is enabled

  Scenario: A follow part shows its copied measures as @ and round-trips unchanged
    When I choose part "A" from the Edit part as stream menu
    Then the part focus stream is:
      """
      @
      3 4 5 6
      @
      """
    And the part focus status says "0 measures changed"
    And the part focus Apply button is disabled

  Scenario: A lyric part bound to a chords part marks its measures with barlines
    Given the chord-bound lyrics fixture is loaded instead
    When I choose part "C" from the Edit part as stream menu
    Then the part focus stream is:
      """
      (a) b c (d) |
      (e) f
      """
    And the part focus status says "0 measures changed"

  Scenario: A lyric stream bound to a chords part adds measures past the end of the score
    Given the chord-bound lyrics fixture is loaded instead
    When I choose part "C" from the Edit part as stream menu
    And I place the cursor at the end of the part focus stream
    And I type " | g h" in the part focus editor
    Then the part focus status says "1 measure changed"
    When I apply the part focus editor
    Then the main editor source is the appended chord lyrics source

  Scenario: Applying the untouched stream re-bars an overflowing measure
    Given the overflowing melody fixture is loaded instead
    When I choose part "M" from the Edit part as stream menu
    Then the part focus stream is:
      """
      1 2 3 4
      5 6 7 1
      2
      """
    And the part focus status says "3 measures changed"
    When I apply the part focus editor
    Then the main editor source is the re-barred melody source

  Scenario: Selecting stream text highlights the matching notes in the preview
    When I choose part "M" from the Edit part as stream menu
    And I select "2 3" on part focus line 1
    Then the preview range-selects 2 notes of part "M"

  Scenario: Selecting a lyric stream highlights the matching syllables in the preview
    When I choose part "W" from the Edit part as stream menu
    And I select "f g" on part focus line 2
    Then the preview range-selects 2 lyrics

  Scenario: Clicking a note of the focused part selects it in the stream
    When I choose part "M" from the Edit part as stream menu
    And I click note 6 of part "M" in the preview
    Then the part focus editor selects "6" on line 2

  Scenario: Clicking another part's note jumps the stream to that measure
    When I choose part "M" from the Edit part as stream menu
    And I click note 3 of part "B" in the preview
    Then the part focus caret is at the start of line 2

  Scenario: Selection keeps working while the stream has errors
    When I choose part "M" from the Edit part as stream menu
    And I place the cursor at the start of part focus line 3
    And I type "(" in the part focus editor
    Then the part focus editor has an error marker
    When I select "2 3" on part focus line 1
    Then the preview range-selects 2 notes of part "M"
