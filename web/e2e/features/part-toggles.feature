Feature: Part hide/solo toggles

  # The "M — Melody" / "C — Chords" legend in the preview header lists only
  # the parts currently enabled by hide/solo state — a hidden or non-soloed part's
  # entry disappears from the legend along with its score content.

  Scenario: Hiding the notes row keeps the lyrics visible
    Given the two-part melody-chords fixture is loaded
    Then the preview contains "Melody" "lyrics"
    And the preview contains the chord content
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain "Melody" "notes"
    And the preview contains "Melody" "lyrics"
    And the preview contains the chord content

  Scenario: Unhiding a part restores its content in the preview
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain "Melody" "notes"
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview contains "Melody" "notes"
    And the preview contains "Melody" "lyrics"

  Scenario: Soloing a part hides all other parts in the preview
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the chord content
    When I solo the "Melody" part
    Then the preview contains "Melody" "notes"
    And the preview does not contain "Melody" "lyrics"
    And the preview does not contain the chord content

  Scenario: Un-soloing restores previously enabled parts
    Given the two-part melody-chords fixture is loaded
    When I solo the "Melody" part
    Then the preview does not contain the chord content
    When I solo the "Melody" part
    Then the preview contains "Melody" "notes"
    And the preview contains the chord content

  Scenario: Soloing multiple parts keeps both visible and hides the rest
    Given the three-part melody-harmony-chords fixture is loaded
    Then the preview contains "Harmony" "notes"
    And the preview contains the chord content
    When I solo the "Harmony" part
    And I solo the "Chords" part
    Then the preview does not contain "Melody" "notes"
    And the preview does not contain "Melody" "lyrics"
    And the preview contains "Harmony" "notes"
    And the preview contains the chord content

  Scenario: A lyric part gets a pill like every other part
    Given the two-part melody-chords fixture is loaded
    Then the part toggles list contains a "Melody" part pill
    And the part toggles list contains a "Melody lyrics" part pill
    And the part toggles list contains a "Chords" part pill

  Scenario: Every pill has exactly a show/hide and a solo button
    Given the two-part melody-chords fixture is loaded
    Then the "Melody" part pill has exactly a show/hide button and a solo button
    And the "Melody lyrics" part pill has exactly a show/hide button and a solo button
    And the "Chords" part pill has exactly a show/hide button and a solo button

  Scenario: Hiding the lyric row keeps the notes visible
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody lyrics" part via its eye toggle, as seen in part toggles
    Then the preview contains "Melody" "notes"
    And the preview does not contain "Melody" "lyrics"

  Scenario: Hiding both rows removes the melody part entirely
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    And I hide the "Melody lyrics" part via its eye toggle, as seen in part toggles
    Then the preview does not contain "Melody" "notes"
    And the preview does not contain "Melody" "lyrics"
    And the preview contains the chord content

  Scenario: Showing a hidden lyric row restores the lyrics
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody lyrics" part via its eye toggle, as seen in part toggles
    Then the preview does not contain "Melody" "lyrics"
    When I hide the "Melody lyrics" part via its eye toggle, as seen in part toggles
    Then the preview contains "Melody" "lyrics"

  Scenario: Pianist view - solo the chords and the lyrics
    Given the two-part melody-chords fixture is loaded
    When I solo the "Chords" part
    And I solo the "Melody lyrics" part
    Then the preview contains the chord content
    And the preview contains "Melody" "lyrics"
    And the preview does not contain "Melody" "notes"

  Scenario: Soloing only a lyric row hides every notes row
    Given the two-part melody-chords fixture is loaded
    When I solo the "Melody lyrics" part
    Then the preview contains "Melody" "lyrics"
    And the preview does not contain "Melody" "notes"
    And the preview does not contain the chord content

  Scenario: Un-soloing the lyric row and notes row restores all rows
    Given the two-part melody-chords fixture is loaded
    When I solo the "Chords" part
    And I solo the "Melody lyrics" part
    And I solo the "Chords" part
    And I solo the "Melody lyrics" part
    Then the preview contains "Melody" "notes"
    And the preview contains "Melody" "lyrics"
    And the preview contains the chord content

  Scenario: Un-soloing keeps an earlier hide of the notes row
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    And I solo the "Chords" part
    And I solo the "Chords" part
    Then the preview does not contain "Melody" "notes"
    And the preview contains "Melody" "lyrics"
    And the preview contains the chord content

  Scenario: Hiding the notes keeps the lyrics at the same position
    Given the two-part melody-chords fixture is loaded
    And I note the horizontal position of the first "Melody" lyric
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the first "Melody" lyric is at the same horizontal position

  Scenario: A shown lyric part is listed in the legend like any other part
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the "Melody lyrics" legend entry
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    Then the preview does not contain the "Melody" legend entry
    And the preview contains the "Melody lyrics" legend entry

  Scenario: A hidden lyric part leaves the legend
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody lyrics" part via its eye toggle, as seen in part toggles
    Then the preview does not contain the "Melody lyrics" legend entry
    And the preview contains the "Melody" legend entry
    And the preview contains the "Chords" legend entry

  Scenario: Lyric row state survives a reload
    Given the two-part melody-chords fixture is loaded
    When I hide the "Melody" part via its eye toggle, as seen in part toggles
    And I solo the "Melody lyrics" part
    And I reload the part toggles page
    Then the preview contains "Melody" "lyrics"
    And the preview does not contain "Melody" "notes"
    And the preview does not contain the chord content

  Scenario: Hiding a part removes its entry from the part-list legend
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the "Chords" legend entry
    And the preview contains the "Melody" legend entry
    When I hide the "Chords" part via its eye toggle, as seen in part toggles
    Then the preview does not contain the "Chords" legend entry
    And the preview contains the "Melody" legend entry

  Scenario: Unhiding a part restores its part-list legend entry
    Given the two-part melody-chords fixture is loaded
    When I hide the "Chords" part via its eye toggle, as seen in part toggles
    Then the preview does not contain the "Chords" legend entry
    When I hide the "Chords" part via its eye toggle, as seen in part toggles
    Then the preview contains the "Chords" legend entry

  Scenario: Soloing a part hides other parts legend entries
    Given the two-part melody-chords fixture is loaded
    Then the preview contains the "Chords" legend entry
    When I solo the "Melody" part
    Then the preview contains the "Melody" legend entry
    And the preview does not contain the "Chords" legend entry

  Scenario: Un-soloing restores previously enabled parts legend entries
    Given the two-part melody-chords fixture is loaded
    When I solo the "Melody" part
    Then the preview does not contain the "Chords" legend entry
    When I solo the "Melody" part
    Then the preview contains the "Melody" legend entry
    And the preview contains the "Chords" legend entry

  Scenario: Soloing multiple parts keeps both legend entries and hides the rest
    Given the three-part melody-harmony-chords fixture is loaded
    Then the preview contains the "Harmony" legend entry
    And the preview contains the "Chords" legend entry
    When I solo the "Harmony" part
    And I solo the "Chords" part
    Then the preview does not contain the "Melody" legend entry
    And the preview contains the "Harmony" legend entry
    And the preview contains the "Chords" legend entry
