Feature: Instrument range

  Scenario: A note outside the part's instrument range is warned on and highlighted
    Given the instrument-range fixture with a violin part is loaded
    Then the editor shows the warning "V: note sounds as F3, below the violin's lowest note G3"
    And the preview highlights a measure with a problem

  Scenario: A part without an instrument is not range-checked
    Given the instrument-range fixture with a part naming no instrument is loaded
    Then the preview highlights no measure

  Scenario: Picking an instrument in the Edit Parts modal writes it to the parts line
    Given the instrument-range fixture with a part naming no instrument is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    And I change the instrument select for part "V" to "guzheng 古箏"
    Then the editor source and stored source both contain "\"Violin\" [V]: notes { instrument: guzheng }", as seen in edit parts modal

  Scenario: Picking an instrument gives a part with no soundfont that instrument's sound
    Given the instrument-range fixture with a part naming no instrument is loaded
    When I open the Edit Parts modal, as seen in edit parts modal
    And I change the instrument select for part "V" to "guzheng 古箏"
    Then the soundfont select for part "V" shows "107: Koto"
    And the editor source and stored source both contain "\"Violin\" [V]: notes { instrument: guzheng }", as seen in edit parts modal
