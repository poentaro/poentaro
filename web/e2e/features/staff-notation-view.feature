Feature: Staff notation view

  Background:
    Given a two-part score is loaded for the staff notation view

  Scenario: Switching the preview to staff notation shows Verovio-engraved staves
    Then the preview shows numbered pages
    When I switch the preview notation to "violin"
    Then the preview shows a staff notation page
    And the staff notation page has a note for every source note

  Scenario: The violin view puts every staff in treble clef
    When I switch the preview notation to "violin"
    Then every staff on the staff notation page is in treble clef

  Scenario: The cello view puts every staff in bass clef
    When I switch the preview notation to "cello"
    Then every staff on the staff notation page is in bass clef

  Scenario: The staff view leaves out hidden parts
    When I switch the preview notation to "violin"
    And I hide the "A" part from the staff view
    Then the staff notation page has 4 notes

  Scenario: Without a preset, the notation goes back to numbered after a reload
    When I switch the preview notation to "violin"
    Then the preview shows a staff notation page
    When I reload the page
    Then the preview shows numbered pages

  Scenario: Exporting a PDF from the staff view engraves the PDF in staff notation
    When I export "PDF" from the preview, keeping it as the numbered PDF
    And I switch the preview notation to "violin"
    And I export "PDF" from the preview, keeping it as the staff PDF
    Then the staff PDF is named "staff.pdf"
    And the staff PDF differs from the numbered PDF

  Scenario: Exporting parts from the staff view zips one staff PDF per part
    When I switch the preview notation to "violin"
    And I export "PDF (ZIP)" from the preview, keeping it as the staff PDF
    Then the staff PDF is named "staff.zip"
    And the staff ZIP holds "staff - Soprano.pdf" and "staff - Alto.pdf"
