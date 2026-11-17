Feature: Resolving a cloud storage save conflict

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: Overwriting mine saves the in-memory edit through the merge editor
    Given a cloud file named "conflict.poentaro" is seeded for the signed-in account
    And I open "conflict"
    And the server copy of "conflict.poentaro" is changed to "5 6 7 1"
    And I open and edit "conflict" with suffix " 5"
    Then the save badge shows "1 file needs attention"
    When I open the sync panel
    And I click "Review and merge" in the sync panel
    And I resolve the conflict markers in the merge editor keeping my version
    And I click "Save" in the merge editor
    Then the content save lands for "conflict.poentaro" containing "1 2 3 4 5"
    And the conflict status badge shows exactly "Saved"
    And the editor still contains "1 2 3 4 5"

  Scenario: Discarding mine reloads the remote content
    Given a cloud file named "conflict-discard.poentaro" is seeded for the signed-in account
    And I open "conflict-discard"
    And the server copy of "conflict-discard.poentaro" is changed to "5 6 7 1"
    And I open and edit "conflict-discard" with suffix " 5"
    Then the save badge shows "1 file needs attention"
    When I open the sync panel
    And I click "Discard" in the sync panel
    And I confirm "Discard my changes"
    Then the editor now shows the remote content "5 6 7 1"
    And the editor no longer contains "1 2 3 4 5"
    And the conflict status badge shows exactly "Saved"
