Feature: The ?file= URL param tracks and selects the active file

  Scenario: Switching the active file updates the ?file= URL param
    Given local files "a.poentaro" and "b.poentaro" are seeded for the URL param test
    When the app loads at the root URL
    Then the URL has the "file" param set to "a.poentaro"
    When I open the file list and select "b"
    Then the file switcher trigger shows "b"
    And the URL has the "file" param set to "b.poentaro"

  Scenario: Loading with a ?file= URL param selects that file
    Given local files "a.poentaro" and "b.poentaro" are seeded for the URL param test
    When the app loads with the URL param "file=b.poentaro"
    Then the file switcher trigger shows "b"
    And the active tab in the file list is named "b"

  Scenario: Loading with a non-ASCII ?file= URL param selects that file without reverting
    Given local files "a.poentaro" and "今山古道.poentaro" are seeded for the URL param test
    When the app loads with the URL param naming "今山古道.poentaro"
    Then the file switcher trigger shows "今山古道"
    And the active tab in the file list is named "今山古道"
    And the URL's "file" param still names "今山古道.poentaro"
