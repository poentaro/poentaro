Feature: Format score sorts parts within each measure by declaration order

  Scenario: Reorders a measure group's out-of-order part lines to match declaration order
    Given the score source:
      ```
      parts ["Melody": notes, "Bass": notes],

      score """
          [Bass] 5 6 7 1
          [Melody] 1 2 3 4
          """,
      ```
    When it is formatted
    Then the formatted source is:
      ```
      parts ["Melody": notes, "Bass": notes],

      score """
          [Melody] 1 2 3 4
          [Bass] 5 6 7 1
          """,
      ```

  Scenario: Sorts each measure group independently
    Given the score source:
      ```
      parts ["Melody": notes, "Bass": notes, "Drums": percussion],

      score """
          [Bass] 5 6 7 1
          [Drums] x x x x
          [Melody] 1 2 3 4

          [Bass] 2 2 2 2
          [Melody] 5 5 5 5
          """,
      ```
    When it is formatted
    Then the formatted source is:
      ```
      parts ["Melody": notes, "Bass": notes, "Drums": percussion],

      score """
          [Melody] 1 2 3 4
          [Bass] 5 6 7 1
          [Drums] x x x x

          [Melody] 5 5 5 5
          [Bass] 2 2 2 2
          """,
      ```

  Scenario: A lyric part's line moves with its part's note line
    Given the score source:
      ```
      parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody], "Bass": notes],

      score """
          [Bass] 5 6 7 1
          [Melody] 1 2 3 4
          [Melodyv1] la la la la
          """,
      ```
    When it is formatted
    Then the formatted source is:
      ```
      parts ["Melody": notes, "Melody lyrics" [Melodyv1]: lyrics [Melody], "Bass": notes],

      score """
          [Melody] 1 2 3 4
          [Melodyv1] la la la la
          [Bass] 5 6 7 1
          """,
      ```

  Scenario: Multiple consecutive verse lines stay together, in order, with their part
    Given the score source:
      ```
      parts [
          "Melody": notes,
          "Melody verse 1" [Melodyv1]: lyrics [Melody],
          "Melody verse 2" [Melodyv2]: lyrics [Melody],
          "Bass": notes,
      ],

      score """
          [Bass] 5 6 7 1
          [Melody] 1 2 3 4
          [Melodyv1] la la la la
          [Melodyv2] na na na na
          """,
      ```
    When it is formatted
    Then the formatted source is:
      ```
      parts [
          "Melody": notes,
          "Melody verse 1" [Melodyv1]: lyrics [Melody],
          "Melody verse 2" [Melodyv2]: lyrics [Melody],
          "Bass": notes,
      ],

      score """
          [Melody] 1 2 3 4
          [Melodyv1] la la la la
          [Melodyv2] na na na na
          [Bass] 5 6 7 1
          """,
      ```

  Scenario: Each part's lyric line follows its notes line, even after both parts reorder
    Given the score source:
      ```
      parts [
          "Melody": notes,
          "Melody lyrics" [Melodyv1]: lyrics [Melody],
          "Bass": notes,
          "Bass lyrics" [Bassv1]: lyrics [Bass],
      ],

      score """
          [Bass] 5 6 7 1
          [Bassv1] bass words
          [Melody] 1 2 3 4
          [Melodyv1] melody words
          """,
      ```
    When it is formatted
    Then the formatted source is:
      ```
      parts [
          "Melody": notes,
          "Melody lyrics" [Melodyv1]: lyrics [Melody],
          "Bass": notes,
          "Bass lyrics" [Bassv1]: lyrics [Bass],
      ],

      score """
          [Melody] 1 2 3 4
          [Melodyv1] melody words
          [Bass] 5 6 7 1
          [Bassv1] bass words
          """,
      ```

  Scenario: A standalone-caption bare line (no preceding [Key] line) is not resolved to its target part's position — it stays in the unattributable-line fallback, sorted after every recognised part
    Given the score source:
      ```
      parts ["Alto": notes, "Tenor": notes],

      score """
          a caption for this measure unrelated to any note
          [Tenor] 5 6 7 1
          [Alto] 1 2 3 4
          """,
      ```
    When it is formatted
    Then the formatted source is:
      ```
      parts ["Alto": notes, "Tenor": notes],

      score """
          [Alto] 1 2 3 4
          [Tenor] 5 6 7 1
          a caption for this measure unrelated to any note
          """,
      ```
