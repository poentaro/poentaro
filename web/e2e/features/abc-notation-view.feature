Feature: ABC notation view

  Background:
    Given a score in D major is loaded for the ABC notation view

  Scenario: Switching the preview to ABC replaces degree numbers with letter names
    Then the preview's note heads read "1, 2, 3, 4, 5, 6, 7, 1"
    And the preview's chord symbols read "1, 6m/3, 4△⁷, ♭5"
    When I switch the preview notation to "abc"
    Then the preview's note heads read "D, E, F♯, G, A, B, C♯, D"
    And the preview's chord symbols read "D, Bm/F♯, G△⁷, A♭"
    When I switch the preview notation to "numbered"
    Then the preview's note heads read "1, 2, 3, 4, 5, 6, 7, 1"
    And the preview's chord symbols read "1, 6m/3, 4△⁷, ♭5"

  Scenario: Exporting a PDF from the ABC view writes letter names
    When I export "PDF" from the preview, keeping it as the numbered PDF
    And I switch the preview notation to "abc"
    And I export "PDF" from the preview, keeping it as the abc PDF
    Then the abc PDF differs from the numbered PDF
