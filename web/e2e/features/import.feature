Feature: Import a `.poentaro` source from a previously exported file

  Background:
    Given a local-storage-backed file "test.poentaro" is seeded for import
    And the app loads the seeded import test file

  Scenario: Import recovers the original source from a previously exported PDF
    When I export the active file as a PDF
    And I import the exported PDF file
    Then the recovered file opens under a deduped name "test 2"
    And the Monaco editor model value equals the original source

  Scenario: Import shows a graceful error for a file with no embedded source
    When I import a plain SVG file with no embedded source
    Then an import error is shown with message "Could not import file"
    And the error modal message contains "No embedded source found in this file."
    And the active file is not replaced
