Feature: Share links

  Scenario: Opens a shared score preview without saving it, then imports on demand
    Given local storage is cleared, as seen in share
    When I open the share URL for "shared-test.poentaro"
    Then the shared preview banner is visible
    And the file switcher is hidden entirely
    And the preview contains "Shared Score"
    When I reload the page
    Then the file switcher is hidden entirely
    When I open the share URL for "shared-test.poentaro" again
    And I click "Import this score"
    Then the file switcher shows "shared-test"
    And the shared preview banner is gone

  Scenario: Collapses the editor pane and hides its toggle when viewing a shared score
    Given local storage is cleared, as seen in share
    When I open the share URL for "shared-test.poentaro"
    Then the editor pane is collapsed, as seen in share
    And the pane-divider toggle is hidden
    When I click "Discard"
    Then the pane-divider toggle is visible again
    When I click the pane-divider toggle, as seen in share
    Then the editor pane is expanded, as seen in share

  Scenario: Discarding a shared preview does not save it
    Given local storage is cleared, as seen in share
    When I open the share URL for "shared-test.poentaro"
    Then the shared preview banner is visible
    When I click "Discard"
    Then the shared preview banner is gone
    And the file switcher no longer shows "shared-test"

  Scenario: Opens legacy uncompressed share links
    Given local storage is cleared, as seen in share
    When I navigate to a legacy uncompressed share link for "shared-test.poentaro"
    Then the shared preview banner is visible

  Scenario: Tapping a single note in a shared preview only highlights that note, not its whole measure
    Given local storage is cleared, as seen in share
    When I open a shared preview with a valid tappable note, as seen in share
    When I tap the first note, as seen in share
    Then the tapped note is highlighted, as seen in share
    And the measure highlight is not shown, as seen in share

  Scenario: Tapping a note in a shared preview on a mobile viewport does not bring up the keyboard
    Given local storage is cleared on a mobile viewport
    When I open a shared preview with a valid tappable note, as seen in share
    When I tap the first note, as seen in share
    Then no editable element gains focus, as seen in share

  Scenario: Clicking a section label in a shared preview highlights that section
    Given local storage is cleared, as seen in share
    When I open a shared preview with a two-section score, as seen in share
    And I click the section label "B" in the SVG preview, as seen in share
    Then section B's measures are amber-highlighted, as seen in share

  Scenario: Share button copies a compressed link that opens as a preview
    Given clipboard permissions are granted, as seen in share
    And a user file "shared-test.poentaro" is seeded in local storage
    When the app loads, as seen in share
    And I open the file actions menu and click the share button
    And I click the copy static link button
    Then the copy static link button shows "Link copied"
    And the copied share URL matches the expected compressed hash for "shared-test.poentaro"
    When I navigate fresh to the copied share URL
    Then the shared preview banner is visible
    And the preview contains "Shared Score"
