Feature: Presets

  # The fixture has a "Melody only" preset (visible parts [M, Mv1]), a
  # "Violin" preset (notation = violin) and the parts Melody [M], Melody
  # lyrics [Mv1] and Chords [C].

  Scenario: Choosing a preset shows only its parts
    Given the presets fixture is loaded
    Then the preset menu shows "Default"
    And the preview contains the chord content
    When I choose the "Melody only" preset
    Then the preset menu shows "Melody only"
    And the "C" part pill is off
    And the "M" part pill is on
    And the preview does not contain the chord content

  Scenario: Turning a pill off under a preset rewrites visible parts, and undo restores it
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I turn the "Mv1" part pill off
    Then the "Melody only" preset has "visible parts [M]"
    And the stored file's "Melody only" preset has "visible parts [M]"
    And the "Mv1" part pill is off
    When I press undo in the main editor
    Then the "Melody only" preset has "visible parts [M, Mv1]"
    And the "Mv1" part pill is on

  Scenario: Soloing a part under a preset rewrites soloed parts, and undo restores it
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I solo the "C" part pill
    Then the "Melody only" preset has "soloed parts [C]"
    And the stored file's "Melody only" preset has "soloed parts [C]"
    And the "C" part pill is soloed
    And the preview contains the chord content
    When I unsolo the "C" part pill
    Then the "Melody only" preset has no "soloed parts"
    And the "C" part pill is not soloed
    And the preview does not contain the chord content
    When I press undo in the main editor
    Then the "Melody only" preset has "soloed parts [C]"
    And the "C" part pill is soloed

  Scenario: On Default, soloing stays in this browser and the file doesn't change
    Given the presets fixture is loaded
    When I solo the "M" part pill
    Then the "M" part pill is soloed
    And the preview does not contain the chord content
    And the stored source is the presets fixture

  Scenario: On Default, pills stay in this browser and the file doesn't change
    Given the presets fixture is loaded
    When I turn the "C" part pill off
    Then the "C" part pill is off
    And the preview does not contain the chord content
    And the stored source is the presets fixture

  Scenario: The chosen preset survives a reload
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I reload the presets page
    Then the preset menu shows "Melody only"
    And the "C" part pill is off

  Scenario: Renaming the chosen preset in the editor keeps it chosen
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I replace "Melody only" with "Tune" in the editor
    Then the preset menu shows "Tune"
    And the "C" part pill is off
    And the preview does not contain the chord content

  Scenario: New preset adds a section with the parts shown now and chooses it
    Given the presets fixture is loaded
    When I turn the "Mv1" part pill off
    And I create a new preset called "No lyrics"
    Then the "No lyrics" preset has "visible parts [M, C]"
    And the "No lyrics" preset has "notation: numbered"
    And the preset menu shows "No lyrics"
    And the "Mv1" part pill is off

  Scenario: A new preset can't take a name already in use
    Given the presets fixture is loaded
    When I open the new preset dialog
    And I type "Melody only" as the preset name
    Then the new preset dialog says "A preset is already called Melody only."
    And the new preset dialog's Create button is disabled

  Scenario: A preset's notation line sets the preview's notation
    Given the presets fixture is loaded
    When I choose the "Violin" preset
    Then the preview shows a staff notation page
    When I choose "Default" in the preset menu
    Then the preview shows numbered pages

  Scenario: Picking a notation under a preset rewrites its notation line
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I switch the preview notation to "cello"
    Then the "Melody only" preset has "notation: cello"
    And the preview shows a staff notation page

  Scenario: In a shared link, picking a notation under a preset changes only the preview
    Given the presets fixture is opened as a shared link
    When I choose the "Violin" preset
    Then the preview shows a staff notation page
    When I switch the preview notation to "numbered"
    Then the preview shows numbered pages

  Scenario: Export > PDF under a preset is named after it
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I export "PDF" and capture the download, as seen in export pdf
    Then the downloaded file is named "test (Melody only).pdf", as seen in export pdf

  Scenario: Export Parts > PDF (ZIP) under a preset holds only its parts
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I export "PDF (ZIP)" and capture the download, as seen in export pdf
    Then the downloaded file is named "test (Melody only).zip", as seen in export pdf
    And the downloaded zip holds only "test (Melody only) - Melody.pdf"

  Scenario: Edit Metadata under a preset writes into the preset's section
    Given the presets fixture is loaded
    When I choose the "Melody only" preset
    And I open the Edit Metadata modal
    Then the edit metadata modal contains "Edit Metadata (Melody only)"
    And the metadata "Title *" field is empty with the placeholder "Test"
    When I fill the metadata "Title *" field with "Tune"
    Then the "Melody only" preset has "title \"Tune\""
    And the "Melody only" preset has "visible parts [M, Mv1]"
    And the editor source and stored source both contain "title \"Test\""
    When I reset the metadata "Title *" field
    Then the metadata "Title *" field is empty with the placeholder "Test"
    And the "Melody only" preset has no "title"
