Feature: Copying a part label's notes and pasting them into another part label's notes

  # Clicking a part label selects every note that part sounds across its
  # system (see `part-label-click-selects-notes.feature`), which is really a
  # Monaco multicursor/range selection under the hood. Since Ctrl/Cmd+C and
  # Ctrl/Cmd+V are just the editor's native copy/paste over whatever text is
  # selected, clicking one part label, copying, clicking a *different* part
  # label (in a different system), and pasting replaces that other part's
  # notes with the copied part's notes — with no dedicated copy/paste code
  # for part labels at all.
  #
  # `max measures per system 1` forces each measure onto its own system,
  # and each measure mentions only one of the two declared parts (the other
  # is implicitly rests, suppressed by the default `hide_resting_parts`), so
  # there's exactly one part label per system to click:
  #
  #   System 0 (measure 0): Melody "1 2"
  #   System 1 (measure 1): Harmony "5 6"

  Scenario: Copying one system's part label and pasting into another system's different part label replaces its notes
    Given the part-label copy-paste fixture is loaded
    And clipboard permissions are granted, as seen in share
    When I click system 0's Melody part label
    And I press the copy keyboard shortcut
    And I click system 1's Harmony part label
    And I press the paste keyboard shortcut
    Then system 1's Harmony part now contains just the notes "1 2"
    And system 0's Melody part still contains just the notes "1 2"
