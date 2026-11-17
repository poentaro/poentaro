Feature: Play selection follows parts hidden or shown after selecting

  # Hiding or showing a part after selecting notes used to leave "play
  # selection" playing the parts captured when the selection was made, so a
  # part hidden afterwards still sounded. The selection is now re-derived from
  # its source text whenever the visible parts change.

  Scenario: Hiding a part after selecting stops it playing, and showing it again brings it back
    Given the three-part hidden-part fixture is loaded with enabled-tracks capture
    And the preview has settled, as seen in play selection follows part visibility
    When I Ctrl-click the Melody part label
    And I toggle the visibility of the Harmony part, as seen in play selection follows part visibility
    Then 4 notes are range-selected in total, as seen in part label cmd click play selection with hidden part
    And the play button shows "Selection" and becomes enabled
    When I click the play button
    Then the captured enabled tracks are exactly "B,M"
    When I toggle the visibility of the Harmony part, as seen in play selection follows part visibility
    Then 5 notes are range-selected in total, as seen in part label cmd click play selection with hidden part
    When I play the selection again
    Then the captured enabled tracks are exactly "B,H,M"
