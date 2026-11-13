Feature: Sequence jump toolbar shows each entry's line choice

  Scenario: An entry that picks a lyric line with WITH names that line on its button
    Given a score whose sequence is "Verse, Verse WITH [w] 2" is loaded
    Then the sequence toolbar shows buttons "Verse, Verse (w 2)" in order
