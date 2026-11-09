Feature: Edit Parts modal

  Scenario: CodeLens Edit Parts link opens the modal
    Given the edit-parts-modal test fixture is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    Then the edit parts modal contains "Edit Parts"

  Scenario: Mode select changes the part mode
    Given the edit-parts-modal test fixture is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    # The "Chords [C]" part starts as "chords". Change it to "notes".
    Then the mode select for part "C" shows "chords", as seen in edit parts modal
    When I change the mode select for part "C" to "notes", as seen in edit parts modal
    Then the mode select for part "C" shows "notes", as seen in edit parts modal

  Scenario: Soundfont select changes the instrument for a part
    Given the edit-parts-modal test fixture is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    # The "Melody [M]" part has no soundfont by default (shows "default sound").
    Then the soundfont select for part "M" shows "default sound", as seen in edit parts modal
    When I select soundfont "40: Violin" for part "M"
    Then the soundfont select for part "M" shows "40: Violin", as seen in edit parts modal

  Scenario: Soundfont select updates UI for a part that follows another part
    Given the edit-parts-modal test fixture with a follow part is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    Then the soundfont select for part "C" shows "default sound", as seen in edit parts modal
    And the mode select for part "C" shows "follow", as seen in edit parts modal
    When I select soundfont "40: Violin" for part "C"
    Then the editor source and stored source both contain "\"Chords\" [C]: follow [M] { sound 40 }"
    And the soundfont select for part "C" shows "40: Violin", as seen in edit parts modal

  Scenario: Octave select changes the MIDI octave offset for a part
    Given the edit-parts-modal test fixture is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    Then the octave select for part "M" shows "0"
    When I change the octave select for part "M" to "-1"
    Then the octave select for part "M" shows "-1"
    When I close the edit parts modal with Escape
    Then the editor source and stored source both contain "\"Melody\" [M]: notes { octave: - 1 }", as seen in edit parts modal

  Scenario: Volume slider changes the MIDI volume for a part
    Given the edit-parts-modal test fixture is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    Then the volume value for part "M" shows "100%"
    When I focus the volume slider for part "M" and press Home
    Then the volume value for part "M" shows "0%"
    When I close the edit parts modal with Escape
    Then the editor source and stored source both contain "\"Melody\" [M]: notes { volume: 0 % }", as seen in edit parts modal

  Scenario: Follow target select changes the followed part
    Given the edit-parts-modal test fixture with multiple followable parts is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    Then the mode select for part "C" shows "follow", as seen in edit parts modal
    And the follow target select for part "C" shows "M"
    When I change the follow target select for part "C" to "H"
    Then the follow target select for part "C" shows "H"
    When I close the edit parts modal with Escape
    Then the editor source and stored source both contain "\"Chords\" [C]: follow [H]", as seen in edit parts modal

  Scenario: Changing soundfont via modal preserves the editor selection
    Given the edit-parts-modal test fixture is loaded
    And the caret is placed on line 7 of the edit-parts-modal fixture with the line selected
    Then the editor selection spans line 7 from column 1 to the end of the line
    And I record the current editor selection
    When I open the Edit Parts modal via CodeLens and change soundfont "40: Violin" for part "M"
    And I close the edit parts modal with Escape
    Then the editor selection is unchanged from before the modal was opened
