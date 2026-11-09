Feature: Edit Parts modal percussion support

  Scenario: Mode select offers percussion and changing to it updates the source
    Given the edit-parts-modal-percussion test fixture is loaded
    When I open the Edit Parts modal
    Then the mode select for part "C" shows "chords"
    When I change the mode select for part "C" to "percussion"
    Then the mode select for part "C" shows "percussion"
    And the editor source and stored source both contain "\"Chords\" [C]: percussion", as seen in edit parts modal percussion

  Scenario: Soundfont picker shows percussion keys, not GM instruments, for a percussion part
    Given the edit-parts-modal-percussion test fixture with a percussion part is loaded
    When I open the Edit Parts modal
    And I click the soundfont select for part "D"
    Then the percussion sound search modal is visible
    And the percussion sound search modal shows a button "38: Acoustic Snare"
    And the percussion sound search modal has no button "0: Acoustic Grand Piano"

  Scenario: Selecting a percussion key persists to source and updates the button label
    Given the edit-parts-modal-percussion test fixture with a percussion part is loaded
    When I open the Edit Parts modal
    And I click the soundfont select for part "D"
    Then the percussion sound search modal is visible
    When I click the "38: Acoustic Snare" button in the percussion sound search modal
    Then the soundfont select for part "D" shows "38: Acoustic Snare"
    And the editor source and stored source both contain "\"Drums\" [D]: percussion { sound 38 }"

  Scenario: Percussion preview toggles play/pause state
    Given the scenario timeout is extended to 60 seconds
    And the edit-parts-modal-percussion test fixture with a percussion part is loaded
    When I open the Edit Parts modal
    And I click the soundfont select for part "D"
    Then the percussion sound search modal is visible
    When I retry clicking the Preview instrument button for "38: Acoustic Snare" in the percussion search modal until it pauses
    Then clicking Pause preview for "38: Acoustic Snare" in the percussion search modal returns it to Preview instrument
