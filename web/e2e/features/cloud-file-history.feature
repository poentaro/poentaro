Feature: Version history of a cloud file

  # Each scenario seeds its own file: the e2e account outlives a scenario,
  # and reseeding a name it already holds overwrites the content, which
  # records a version of its own.
  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: The History panel lists each earlier version
    Given a cloud file named "history-list.poentaro" is seeded for the signed-in account
    And I open and edit "history-list" with suffix " 5"
    And the content save lands for "history-list.poentaro" containing "1 2 3 4 5"
    When I open the History panel
    Then the History panel lists 1 earlier version

  Scenario: The diff shows what restoring a version would change
    Given a cloud file named "history-diff.poentaro" is seeded for the signed-in account
    And I open and edit "history-diff" with suffix " 5"
    And the content save lands for "history-diff.poentaro" containing "1 2 3 4 5"
    When I open the History panel
    Then the History diff compares the current text ending in "1 2 3 4 5" with the version ending in "1 2 3 4"
    And the History diff marks the removed characters
    When I switch the History diff to inline
    Then the History diff is inline
    And the History diff marks the removed characters

  Scenario: Restoring a version saves it as a new version
    Given a cloud file named "history-restore.poentaro" is seeded for the signed-in account
    And I open and edit "history-restore" with suffix " 5"
    And the content save lands for "history-restore.poentaro" containing "1 2 3 4 5"
    When I open the History panel
    And I click Restore in the History panel
    Then the editor no longer contains "1 2 3 4 5"
    When I press Cmd/Ctrl+S
    Then the last content save for "history-restore.poentaro" no longer contains "1 2 3 4 5"
    And the save badge shows "Saved"
    When I open the History panel
    Then the History panel lists 2 earlier versions
