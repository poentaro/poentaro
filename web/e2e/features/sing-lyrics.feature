Feature: Sing lyrics with a Chimera voice

  # A part whose sound is a Chimera voice sings its lyric part's syllables in
  # every audio path. The voice is its own wasm module, downloaded only when
  # a score sings.
  Scenario: Exporting MP3 of a score with a singing part sings it with the singing voice
    Given the export test timeout is extended to 60 seconds
    And downloads of the singing voice are recorded
    And a score whose soprano part sings its lyrics is loaded
    When I open the export menu and choose "MP3"
    Then the inline audio player eventually becomes visible
    And the singing voice was downloaded

  Scenario: Playing a score with a singing part sings it with the singing voice
    Given the export test timeout is extended to 60 seconds
    And downloads of the singing voice are recorded
    And a score whose soprano part sings its lyrics is loaded
    When I click the Play All button
    Then the Play All button eventually shows the playing state
    And the singing voice was downloaded

  Scenario: Exporting MP3 of a score without a singing part never downloads the singing voice
    Given the export test timeout is extended to 60 seconds
    And downloads of the singing voice are recorded
    And the single-part export source is loaded
    When I open the export menu and choose "MP3"
    Then the inline audio player eventually becomes visible
    And the singing voice was never downloaded
