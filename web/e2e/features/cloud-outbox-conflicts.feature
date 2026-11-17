Feature: Conflicting cloud saves in the outbox conflict flow are merged or resolved by the user

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: D1 A stale tab with no pending edits quietly takes the server version
    Given a cloud file named "outbox-d1.poentaro" is seeded for the signed-in account
    And I open "outbox-d1"
    And the server copy of "outbox-d1.poentaro" is changed to "1 2 3 4 9"
    When the tab becomes visible again
    Then the editor contains "1 2 3 4 9"
    And no conflict is shown

  Scenario: D2 Edits on different lines merge without asking
    Given a cloud file named "outbox-d2.poentaro" is seeded for the signed-in account with two separate lines
    And I open "outbox-d2"
    And the server copy of "outbox-d2.poentaro" has its second line changed
    When I change the first line of the open file
    Then the content save lands for "outbox-d2.poentaro" containing both changes
    And a notice says "Merged with changes from another device"

  Scenario: D3 Edits to the same line open the merge view
    Given a cloud file named "outbox-d3.poentaro" is seeded for the signed-in account
    And I open "outbox-d3"
    And the server copy of "outbox-d3.poentaro" is changed to "1 2 3 9"
    When I open and edit "outbox-d3" with suffix " 5"
    Then the save badge shows "1 file needs attention"
    When I open the sync panel
    And I click "Review and merge" in the sync panel
    Then the merge editor shows the server version "1 2 3 9" on the left
    And the Save button in the merge editor is disabled
    When I resolve the conflict markers in the merge editor
    And I click "Save" in the merge editor
    Then the content save lands for "outbox-d3.poentaro" containing the resolved text

  Scenario: D4 Keeping both creates a second file
    Given a cloud file named "outbox-d4.poentaro" is seeded for the signed-in account
    And I open "outbox-d4"
    And the server copy of "outbox-d4.poentaro" is changed to "1 2 3 9"
    And I open and edit "outbox-d4" with suffix " 5"
    When I open the sync panel
    And I click "Keep both files" in the sync panel
    Then a file whose name starts with "outbox-d4 (conflicted copy" is listed in the file switcher
    And the editor of "outbox-d4.poentaro" contains "1 2 3 9"
