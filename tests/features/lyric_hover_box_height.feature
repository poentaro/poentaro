Feature: Lyric syllable hover/click-target box height

  A lyric syllable's invisible click-target rect (grid_layout::layout_heights
  ::lyric_row_height, consumed by coordinate_resolver::highlights) is sized
  as `row_height * 1.5` for the whole verse row, independent of the
  syllable's own resolved font size (RenderConfig::lyric_font_size /
  lyric_cjk_font_size). When `lyrics_font_size` is set large enough relative
  to `row_height` — CJK syllables first, since they render 20% larger than
  Latin ones — the glyph no longer fits inside its own click-target box.

  Background:
    Given "metadata" sets "row_height" to "30"

  Scenario: CJK syllable overflows its click-target box before Latin does
    Given "metadata" sets "lyrics_font_size" to "40"
    And the score has a note with lyric syllable "詞"
    When it is rendered
    Then the lyric click-target height should be at least the resolved lyric font size
    And the lyric text baseline should be middle-anchored
    And the lyric glyph should be fully contained within its click-target box

  Scenario: Latin syllable still fits its click-target box at the same override
    Given "metadata" sets "lyrics_font_size" to "40"
    And the score has a note with lyric syllable "Love"
    When it is rendered
    Then the lyric click-target height should be at least the resolved lyric font size
    And the lyric text baseline should be middle-anchored
    And the lyric glyph should be fully contained within its click-target box

  Scenario Outline: Click-target height should track the resolved lyric font size
    Given "metadata" sets "lyrics_font_size" to "<lyrics_font_size>"
    And the score has a note with lyric syllable "<syllable>"
    When it is rendered
    Then the lyric click-target height should be at least the resolved lyric font size
    And the lyric text baseline should be middle-anchored
    And the lyric glyph should be fully contained within its click-target box

    Examples:
      | lyrics_font_size | syllable |
      | 18                | Love     |
      | 18                | 詞        |
      | 40                | Love     |
      | 40                | 詞        |
      | 60                | Love     |
      | 60                | 詞        |

  Scenario Outline: Original repro parameters — small row_height, large lyrics_font_size override
    Given "metadata" sets "row_height" to "24"
    And "metadata" sets "lyrics_font_size" to "<lyrics_font_size>"
    And the score has a note with lyric syllable "<syllable>"
    When it is rendered
    Then the lyric click-target height should be at least the resolved lyric font size
    And the lyric text baseline should be middle-anchored
    And the lyric glyph should be fully contained within its click-target box

    Examples:
      | lyrics_font_size | syllable |
      | 40                | 菩        |
      | 40                | Love     |
