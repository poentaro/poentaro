Feature: Guitar chord view

  Background:
    Given a chord progression in C major is loaded for the guitar chord view

  Scenario: Switching the preview to Guitar draws a chord diagram at each chord symbol
    Then the preview shows no guitar chord diagrams
    When I switch the preview notation to "guitar"
    Then the preview's guitar chord diagrams read "x 3 2 0 1 0, x 0 2 2 1 0, 1 3 3 2 1 1, 3 2 0 0 0 3"
    When I switch the preview notation to "numbered"
    Then the preview shows no guitar chord diagrams

  Scenario: The guitar chord view follows each degree with its letter name
    Then the preview's note heads read "1, 2, 3, 4"
    And the preview's chord symbols read "1, 6m, 4, 5"
    When I switch the preview notation to "guitar"
    Then the preview's note heads read "1 (C), 2 (D), 3 (E), 4 (F)"
    And the preview's chord symbols read "1 (C), 6m (Am), 4 (F), 5 (G)"
    When I switch the preview notation to "numbered"
    Then the preview's note heads read "1, 2, 3, 4"
    And the preview's chord symbols read "1, 6m, 4, 5"

  Scenario: Exporting a PDF from the guitar chord view draws the diagrams
    When I export "PDF" from the preview, keeping it as the numbered PDF
    And I switch the preview notation to "guitar"
    And I export "PDF" from the preview, keeping it as the guitar PDF
    Then the guitar PDF differs from the numbered PDF
