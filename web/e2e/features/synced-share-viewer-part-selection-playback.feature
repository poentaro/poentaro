Feature: Playing a part-label selection in a synced share viewer

  # Regression test: in a Synced/shared viewer (no Monaco editor mounted),
  # clicking a part label to select just that part's notes across a range of
  # measures — then pressing play — plays EVERY part in those measures,
  # not just the selected one.
  #
  # Root cause: `useMeasureRangeSelection.ts`'s no-mounted-editor branch
  # (the code path a viewer's part-label/measure/bar-line click always takes,
  # since there's no Monaco editor to round-trip a selection through) never
  # calls `applyNoteSelectionSilently`/`applyLyricSelectionSilently` — it only
  # records the clicked cells for SVG highlight painting and calls
  # `notifySelection` to update `selectedMeasureRange`. That means
  # `useNoteSelection`'s `selectedNoteRangePlaybackInfo` (and the part names
  # it derives) never reflects what was actually selected in the viewer, so
  # `notePlaybackSelectionActive` stays permanently false there — the play
  # button always calls the un-filtered `playSelectedMeasures`, which enables
  # every currently-visible part regardless of which part(s) the user's
  # selection actually touched.

  Background:
    Given clipboard permissions are granted

  Scenario: Selecting only one part in a synced viewer plays only that part
    Given the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with a two-part synced range-select score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a viewer opens the copied sync link in a new page, with enabled-tracks capture, and waits for measures to render
    Then the viewer's parts toolbar is visible and no Monaco editor is mounted
    When the viewer plain-clicks the Melody part label
    Then the viewer's play-measure button reads "Measures 1-2"
    When the viewer clicks the play button
    Then the captured enabled tracks for the viewer are exactly Melody
