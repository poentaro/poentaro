Feature: Shared preview app header on a mobile viewport

  Scenario: App header scrolls horizontally instead of wrapping when viewing a shared score on a mobile viewport
    Given local storage is cleared on a mobile viewport
    When I open the share URL for "shared-mobile-test.poentaro" on the mobile viewport
    Then the shared preview banner is visible, as seen in shared preview mobile
    And the app header only scrolls horizontally, not vertically
