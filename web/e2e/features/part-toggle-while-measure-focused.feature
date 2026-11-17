Feature: Toggling a part while a measure is focused

  # The default demo source (demo/01-pitches.poentaro) declares its parts:
  #
  #   parts ["Melody" [M]: notes, "Melody lyrics" [Mv1]: lyrics [M]],
  #
  # Line 7 in the editor (`[M] 1 2 3 0`) is the first note line of measure 1.
  #
  # Regression: when the cursor is inside a measure, `highlightedSvgs` is shown
  # in the Preview. Toggling a part should re-render `highlightedSvgs` with the
  # new part filter, but the effect that fires the re-render only depended on
  # `selectedMeasureRange` — so parts changes were silently ignored.

  Scenario: Toggling a part rerenders the highlighted SVG while a measure is focused
    Given the app has loaded and the cursor is on the first measure's note line
    Then the measure highlight rect is visible, as seen in part toggle while measure focused
    When I hide the first part via its eye toggle
    Then the highlighted preview SVG content changes
