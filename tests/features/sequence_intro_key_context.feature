Feature: A key change before the first `sequence`-listed section survives into exported audio

  A measure's `key=` directive changes what pitch each scale degree (`1`-`7`)
  resolves to for every later measure, until the next `key=` directive. When a
  `sequence` is present, playback (and therefore MIDI/WAV/MP3 export)
  follows the sequence's resolved order instead of written order — but any
  measure written before the sequence's first listed section (e.g. an
  unlabeled intro, or one under a label the sequence never lists) never
  appears in that resolved order. Its `key=` directive should still carry
  forward as context for the measures that ARE played, the same way BPM/key
  context is accumulated for a measure-range selection.

  Scenario: A key set before the first sequence-listed section is not dropped from exported audio
    Given the score source:
      ```
      metadata { title "t" },

      parts ["Melody" [M]: notes],

      sequence ["A"],

      score """
          key=F4
          [M] 1

          label="A"
          [M] 1
          """,
      ```
    When the score's audio is generated
    Then the first sounded note is MIDI pitch 65
