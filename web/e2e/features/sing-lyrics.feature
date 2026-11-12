Feature: Sing lyrics in an exported MP3

  # A lyric part with a `voice` sings in the whole-score MP3 export. The voice
  # is its own wasm module, downloaded only when a score sings.
  Scenario: Exporting MP3 of a score with a voiced lyric part sings it with the singing voice
    Given the export test timeout is extended to 60 seconds
    And downloads of the singing voice are recorded
    And a score whose lyric part sings in a female voice is loaded
    When I open the export menu and choose "MP3"
    Then the inline audio player eventually becomes visible
    And the singing voice was downloaded

  Scenario: Exporting MP3 of a score without a voiced lyric part never downloads the singing voice
    Given the export test timeout is extended to 60 seconds
    And downloads of the singing voice are recorded
    And the single-part export source is loaded
    When I open the export menu and choose "MP3"
    Then the inline audio player eventually becomes visible
    And the singing voice was never downloaded
