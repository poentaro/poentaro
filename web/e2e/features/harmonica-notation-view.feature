Feature: Harmonica tab notation view

  Background:
    Given a score in C major is loaded for the harmonica notation view

  Scenario: Switching the preview to Harmonica replaces degree numbers with harmonica tabs
    When I switch the preview notation to "harmonica"
    Then the preview's note heads read "1, (1), 2, (2), 3, (3), (4), 4, (2), 6, 8"
    And the preview flags the unplayable note heads "?"
    And the preview draws 1 slide arrow
    When I switch the preview notation to "numbered"
    Then the preview's note heads read "1, 2, 3, 4, 5, 6, 7, 1, 5, ♯4, 3, 1"

  Scenario: Exporting a PDF from the Harmonica view writes harmonica tabs
    When I export "PDF" from the preview, keeping it as the numbered PDF
    And I switch the preview notation to "harmonica"
    And I export "PDF" from the preview, keeping it as the harmonica PDF
    Then the harmonica PDF differs from the numbered PDF
