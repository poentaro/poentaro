Feature: Cross-part click-and-click selection is always contiguous

  # A cross-part range covers the time between its two endpoints in every
  # row between them (see `cross_row_range` in
  # `crates/poentaro-wasm/src/selection_range/helpers.rs`). It used to bound
  # each part by the clicked note's position within its measure, so a part
  # with a different rhythm was left with a hole.

  Scenario: A held note in a middle part is selected when the range passes over it
    Given the score:
      ```
      parts ["A": notes, "B": notes, "C": notes],

      score """
          [A] 1 2 3 4
          [B] 5---
          [C] 3 4 5 6
          """,
      ```
    When I click note 3 of part A
    And I click note 4 of part C
    Then notes 3 and 4 of part A are range-selected
    And note 1 of part B is range-selected
    And notes 3 and 4 of part C are range-selected
    And notes 1 and 2 of part A are not range-selected
    And notes 1 and 2 of part C are not range-selected

  Scenario: A part with a different rhythm is selected by time, not by note position
    Given the score:
      ```
      parts ["A": notes, "B": notes],

      score """
          [A] 1 2 3 4
          [B] 5_ 6_ 7_ 1_ 2_ 3_ 4_ 5_
          """,
      ```
    When I click note 2 of part A
    And I click note 5 of part B
    Then notes 2 and 3 of part A are range-selected
    And notes 3, 4 and 5 of part B are range-selected
    And notes 1, 2, 6, 7 and 8 of part B are not range-selected

  Scenario: A lyric-to-note range over a held note leaves no hole
    Given the score:
      ```
      parts ["A": notes, "Lyrics" [L]: lyrics [A], "B": notes, "C": notes],

      score """
          [A] 1 2 3 4
          [L] 一 二 三 四
          [B] 5---
          [C] 3 4 5 6
          """,
      ```
    When I click syllable 3 of part L
    And I click note 4 of part C
    Then syllables 3 and 4 of part L are range-selected
    And note 1 of part B is range-selected
    And notes 3 and 4 of part C are range-selected
    And syllables 1 and 2 of part L are not range-selected
    And notes 1, 2, 3 and 4 of part A are not range-selected
