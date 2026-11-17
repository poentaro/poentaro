/** The `.poentaro` fixtures of `part-stream-editor.feature`. */

export const SOURCE = `parts ["Melody" [M]: notes, "Alto" [A]: follow [M], "Words" [W]: lyrics [M], "Bass" [B]: notes],

score """
    label="Verse"
    [M] 1 2 3 4
    [W] a b c d
    [B] 1, - 5, -

    label="Chorus"
    [M] 5 6 7 1
    [A] 3 4 5 6
    [W] e f g h
    [B] 4, - 1 -

    [M] 3 4 5 6
    [W] i j k l
    """,
`

export const SHIFTED_MELODY_SOURCE = `parts ["Melody" [M]: notes, "Alto" [A]: follow [M], "Words" [W]: lyrics [M], "Bass" [B]: notes],

score """
    label="Verse"
    [M] 0 1 2 3
    [W] a b c d
    [B] 1, - 5, -

    label="Chorus"
    [M] 4 5 6 7
    [A] 3 4 5 6
    [W] e f g h
    [B] 4, - 1 -

    [M] 1 3 4 5
    [W] i j k l

    [M] 6 0 0 0
    """,
`

export const CHORD_BOUND_LYRICS_SOURCE = `parts ["Harmony" [H]: chords, "Chant" [C]: lyrics [H]],

score """
    [H] 1 - 4 -
    [C] (a) b c (d)

    [H] 5 - - -
    [C] (e) f
    """,
`

const APPENDED_CHORD_LYRICS_SOURCE = CHORD_BOUND_LYRICS_SOURCE.replace(
  '    """,',
  '\n    [C] g h\n    """,',
)

export const OVER_BRACKETED_CHORD_LYRICS_SOURCE = `parts ["Harmony" [H]: chords, "Chant" [C]: lyrics [H]],

score """
    [H] 1 - 4 -
    [C] (a) (b) (c)
    """,
`

export const OVERFLOWING_MELODY_SOURCE = `parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4 5

    [M] 6 7 1 2
    """,
`

export const HELD_NOTE_MELODY_SOURCE = `parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4

    [M] 5 - 6 7

    [M] 1 2 3 4
    """,
`

export const RE_BARRED_MELODY_SOURCE = `parts ["Melody" [M]: notes],

score """
    [M] 1 2 3 4

    [M] 5 6 7 1

    [M] 2 0 0 0
    """,
`

export const UNWRITTEN_LYRICS_SOURCE = `parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4

    [M] 5 6 7 1

    [M] 3 4 5 6
    """,
`

export const FIRST_MEASURE_LYRICS_SOURCE = `parts ["Melody" [M]: notes, "Words" [W]: lyrics [M]],

score """
    [M] 1 2 3 4
    [W] a b c d

    [M] 5 6 7 1

    [M] 3 4 5 6
    """,
`

export const EXPECTED_SOURCES: Record<string, string> = {
  'original fixture': SOURCE,
  'shifted melody': SHIFTED_MELODY_SOURCE,
  're-barred melody': RE_BARRED_MELODY_SOURCE,
  'first measure lyrics': FIRST_MEASURE_LYRICS_SOURCE,
  'appended chord lyrics': APPENDED_CHORD_LYRICS_SOURCE,
}
