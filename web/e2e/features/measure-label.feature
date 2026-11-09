Feature: Measure label

  Scenario: Shows measure number when cursor is placed on a note line
    Given the app is loaded and the editor is focused
    When I jump to line 7 and wait for the selection debounce
    Then the play-measure button shows "Measure 1"

  Scenario: Detects measure when cursor is at end of last character of a note line
    Given the app is loaded and the editor is focused
    When I jump to line 7, press End, and wait for the selection debounce
    Then the play-measure button shows "Measure 1"

  Scenario: Detects measure when cursor is at end of last character of a Chinese lyric line
    Given the Chinese-lyric measure-label test fixture is loaded
    When I jump to line 13, press End, and wait for the selection debounce
    Then the play-measure button shows "Measure 1"
