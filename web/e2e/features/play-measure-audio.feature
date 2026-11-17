Feature: Play measure audio playback

  Scenario: Clicking play on a selected measure starts and finishes playback
    Given the app is loaded and the editor is focused, with the disk cache workaround
    When I place the cursor inside measure 0 via line 7
    Then the play-measure button label mentions the measure
    And the play-measure button becomes enabled once the soundfont loads
    When I click the play-measure button
    Then the play-measure button shows the playing state
    And the play-measure button eventually stops showing the playing state
