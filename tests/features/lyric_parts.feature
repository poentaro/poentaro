# Lyric parts: lyrics are declared as their own parts (`"Verse 1" [v1] :
# lyrics [b]`) and written on `[v1]`-keyed score lines. A bare, unprefixed
# line no longer attaches to the part above it.

Feature: Lyric parts

  Scenario: A lyric part's line attaches to its target notes part
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts ["Alto": notes, "Verse 1" [v1]: lyrics [Alto]],

      score """
          time=4/4 key=C4 bpm=120
          [Alto] 3213
          [v1] Ma rry had a
          """,
      ```
    When it is compiled
    Then part "Alto" measure 1 has 4 note events
    And part "Alto" measure 1 has 1 lyric verse
    And part "Alto" measure 1 verse 1 has syllables "Ma, rry, had, a"

  Scenario: A held syllable on a lyric line still stretches across notes
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts ["Alto": notes, "Verse 1" [v1]: lyrics [Alto]],

      score """
          time=4/4 key=C4 bpm=120
          [Alto] 1 1 2 3
          [v1] Glo - ry be
          """,
      ```
    When it is compiled
    Then part "Alto" measure 1 verse 1 has syllables "Glo, -, ry, be"

  Scenario: A no-lyrics marker on a lyric line means zero syllables
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts ["Alto": notes, "Verse 1" [v1]: lyrics [Alto]],

      score """
          time=4/4 key=C4 bpm=120
          [Alto] 1 2 3 4
          [v1] _
          """,
      ```
    When it is compiled
    Then part "Alto" measure 1 has 1 lyric verse
    And part "Alto" measure 1 verse 1 has 0 syllables

  Scenario: Several lyric parts of one notes part become verses in declaration order
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts [
          "bass" [b]: notes,
          "Verse 1" [v1]: lyrics [b],
          "Verse 2" [v2]: lyrics [b],
          "Lyrics" [l]: lyrics [b],
      ],

      score """
          time=4/4 key=C4 bpm=120
          [b] 1 2 3 4
          [l] x y z w
          [v2] one two three four
          [v1] a b c d
          """,
      ```
    When it is compiled
    Then part "b" measure 1 has 3 lyric verses
    And part "b" measure 1 verse 1 has syllables "a, b, c, d"
    And part "b" measure 1 verse 2 has syllables "one, two, three, four"
    And part "b" measure 1 verse 3 has syllables "x, y, z, w"

  Scenario: A skipped verse is filled with no lyrics
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts ["Melody": notes, "Verse 1" [v1]: lyrics [Melody], "Verse 2" [v2]: lyrics [Melody]],

      score """
          time=4/4 key=C4 bpm=120
          [Melody] 1 2 3 4
          [v2] la la la la
          """,
      ```
    When it is compiled
    Then part "Melody" measure 1 has 2 lyric verses
    And part "Melody" measure 1 verse 1 has 0 syllables
    And part "Melody" measure 1 verse 2 has syllables "la, la, la, la"

  Scenario: A measure that writes no lyric line has a blank verse
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts ["Alto": notes, "Verse 1" [v1]: lyrics [Alto]],

      score """
          time=4/4 key=C4 bpm=120
          [Alto] 3213
          [v1] Ma rry had a

          [Alto] 3213
          """,
      ```
    When it is compiled
    Then part "Alto" measure 1 has 1 lyric verse
    And part "Alto" measure 2 has 1 lyric verse
    And part "Alto" measure 2 verse 1 has 0 syllables

  Scenario: Each notes part carries its own lyric parts
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts [
          "Soprano": notes,
          "Soprano lyrics" [sl]: lyrics [Soprano],
          "Tenor": notes,
          "Tenor lyrics" [tl]: lyrics [Tenor],
      ],

      score """
          time=4/4 key=C4 bpm=120
          [Soprano] 1 2 3 4
          [sl] la la la la
          [Tenor] 1 2 3 4
          [tl] bum bum bum bum
          """,
      ```
    When it is compiled
    Then part "Soprano" measure 1 verse 1 has syllables "la, la, la, la"
    And part "Tenor" measure 1 verse 1 has syllables "bum, bum, bum, bum"

  Scenario: A lyric line on its own gives its target part rests for notes
    Given the score source:
      ```
      metadata { title "t", author "a" },

      parts ["Alto": notes, "Verse 1" [v1]: lyrics [Alto]],

      score """
          time=4/4 key=C4 bpm=120
          [v1] la la la la
          """,
      ```
    When it is compiled
    Then part "Alto" measure 1 has 4 note events
    And part "Alto" measure 1 has 1 lyric verse
