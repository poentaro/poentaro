Feature: Show the selected note/chord's letter names in a bottom drawer

  # When a click-and-click selection in the preview settles on exactly one
  # note (notes part) or one chord (chords part), a drawer slides up from the
  # bottom of the preview showing what that numbered symbol means in letter
  # names, resolved against the key in effect at that measure. Chords also
  # get a guitar chord diagram for every
  # voicing chords-db knows.

  Scenario: Selecting a single note shows its letter name in the key of C
    Given a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    Then the pitch drawer slides up
    And the pitch drawer shows the letter names "C"

  Scenario: The letter name follows the measure's key, not the first key
    Given a score whose measure 1 is in key C4 and measure 2 is in key G4
    When I click-and-click select the note "1" in measure 2
    Then the pitch drawer shows the letter names "G"

  Scenario: Accidentals are applied to the letter name
    Given a score in key C4 with "[M] 4# 7b 1 1" is loaded
    When I click-and-click select the note "4#" on the M line
    Then the pitch drawer shows the letter names "F#"

  Scenario: A sharp key spells its notes with sharps
    Given a score in key G4 with "[M] 7 1 2 3" is loaded
    When I click-and-click select the note "7" on the M line
    Then the pitch drawer shows the letter names "F#"

  Scenario: A flat key spells its notes with flats
    Given a score in key F4 with "[M] 4 5 6 7" is loaded
    When I click-and-click select the note "4" on the M line
    Then the pitch drawer shows the letter names "Bb"

  Scenario: Octave markers do not change the letter name
    Given a score in key C4 with "[M] 1' 5, 1 1" is loaded
    When I click-and-click select the note "5," on the M line
    Then the pitch drawer shows the letter names "G"

  Scenario: Selecting a single chord shows its chord tones
    Given a score in key C4 with "[C] 1m7 - - -" on a chords part is loaded
    When I click-and-click select the chord "1m7" on the C line
    Then the pitch drawer slides up
    And the pitch drawer shows the chord name "Cm7"
    And the pitch drawer shows the letter names "C Eb G Bb"

  Scenario: Selecting a chord shows its guitar chord diagram
    Given a score in key C4 with "[C] 1m7 - - -" on a chords part is loaded
    When I click-and-click select the chord "1m7" on the C line
    Then the pitch drawer shows a guitar diagram with frets "x 3 1 3 4 x"

  Scenario: Selecting a chord shows every guitar voicing, open shape first
    Given a score in key C4 with "[C] 1 - - -" on a chords part is loaded
    When I click-and-click select the chord "1" on the C line
    Then the pitch drawer shows guitar diagrams with frets:
      | frets          |
      | x 3 2 0 1 0    |
      | 3 3 5 5 5 3    |
      | x x 5 5 5 8    |
      | 8 10 10 9 8 8  |

  Scenario: Selecting a chord shows a keyboard labelled with each tone's interval from the root
    Given a score in key C4 with "[C] 1m7 - - -" on a chords part is loaded
    When I click-and-click select the chord "1m7" on the C line
    Then the pitch drawer shows a keyboard with tones "1:0 b3:3 5:7 b7:10"

  Scenario: Pressing a highlighted keyboard key names its function
    Given a score in key C4 with "[C] 1m7 - - -" on a chords part is loaded
    When I click-and-click select the chord "1m7" on the C line
    And I press the keyboard key with function "Minor 3rd"
    Then the pitch drawer names the pressed keyboard key "Minor 3rd"

  Scenario: Selecting a single note shows no guitar diagram
    Given a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    Then the pitch drawer shows no guitar diagram

  Scenario: A chord with no known guitar voicing still shows its chord tones
    # chords-db has no 7sus2 voicing.
    Given a score in key C4 with "[C] 1sus27 - - -" on a chords part is loaded
    When I click-and-click select the chord "1sus27" on the C line
    Then the pitch drawer shows the chord name "C7sus2"
    And the pitch drawer shows the letter names "C D G Bb"
    And the pitch drawer shows no guitar diagram

  Scenario: A slash chord shows its bass note separately
    Given a score in key C4 with "[C] 1/5 - - -" on a chords part is loaded
    When I click-and-click select the chord "1/5" on the C line
    Then the pitch drawer shows the chord name "C/G"
    And the pitch drawer shows the letter names "C E G"
    And the pitch drawer shows the bass note "G"

  Scenario: Selecting a rest does not open the drawer
    Given a score in key C4 with "[M] 0 2 3 4" is loaded
    When I click-and-click select the rest "0" on the M line
    Then the pitch drawer is hidden

  Scenario: Selecting more than one note does not open the drawer
    Given a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select from note "1" to note "2" on the M line
    Then the pitch drawer is hidden

  Scenario: Clearing the selection slides the drawer away
    Given a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    And I press Escape
    Then the pitch drawer slides down and is hidden

  Scenario: Dragging the drawer's handle down far enough closes it on mobile
    Given the viewport is a mobile phone
    And a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    And I touch-drag the pitch drawer's handle down past half the drawer's height
    Then the pitch drawer slides down and is hidden
    And the note "1" on the M line is still range-selected

  Scenario: A short drag snaps the drawer back open
    Given the viewport is a mobile phone
    And a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    And I touch-drag the pitch drawer's handle down by less than half the drawer's height
    Then the pitch drawer snaps back fully open
    And the pitch drawer shows the letter names "C"

  Scenario: Dragging the drawer closed with a mouse works on desktop too
    Given a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    And I mouse-drag the pitch drawer's handle down past half the drawer's height
    Then the pitch drawer slides down and is hidden

  Scenario: A dismissed drawer reopens on the next single-note selection
    Given the viewport is a mobile phone
    And a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    And I touch-drag the pitch drawer's handle down past half the drawer's height
    And I click-and-click select the note "3" on the M line
    Then the pitch drawer slides up
    And the pitch drawer shows the letter names "E"

  Scenario: Selecting a different single note updates the open drawer
    Given a score in key C4 with "[M] 1 2 3 4" is loaded
    When I click-and-click select the note "1" on the M line
    And I click-and-click select the note "3" on the M line
    Then the pitch drawer shows the letter names "E"

  Scenario: Selecting a single note in a shared link's preview shows its letter name
    Given a shared link to a score in key G4 with "[M] 7 1 2 3" is opened
    When I click-and-click select the note "7" on the M line
    Then the pitch drawer slides up
    And the pitch drawer shows the letter names "F#"
