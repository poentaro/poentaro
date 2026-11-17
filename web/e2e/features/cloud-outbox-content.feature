Feature: Cloud content saves go through the local outbox

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: A1 An edit made while the cloud is unreachable survives a page refresh
    Given a cloud file named "outbox-a1.poentaro" is seeded for the signed-in account
    And every content-save request will be aborted as a network failure
    And I open and edit "outbox-a1" with suffix " 5"
    Then the save badge shows "1 change waiting"
    When I reload the page
    And content-save requests are allowed again
    Then the editor contains "1 2 3 4 5"
    And the content save lands for "outbox-a1.poentaro" containing "1 2 3 4 5"
    And the save badge shows "Saved"

  Scenario: A2 A server error is retried like an outage
    Given a cloud file named "outbox-a2.poentaro" is seeded for the signed-in account
    And the first content-save request will fail with a 503 response
    And I open and edit "outbox-a2" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the outbox retry delay
    Then the content save lands for "outbox-a2.poentaro" containing "1 2 3 4 5"
    And the save badge shows "Saved"

  Scenario: A3 A file that cannot sync does not block another file
    Given a cloud file named "outbox-a3-bad.poentaro" is seeded for the signed-in account
    And a cloud file named "outbox-a3-good.poentaro" is seeded for the signed-in account
    And every content-save request for "outbox-a3-bad.poentaro" will fail with a 400 response
    When I open and edit "outbox-a3-bad" with suffix " 5"
    And I open and edit "outbox-a3-good" with suffix " 6"
    Then the content save lands for "outbox-a3-good.poentaro" containing "1 2 3 4 6"
    And the save badge shows "1 file needs attention"

  Scenario: A4 A save whose response was lost is not reported as a conflict
    Given a cloud file named "outbox-a4.poentaro" is seeded for the signed-in account
    And the first content-save response will be dropped after the server applies it
    And I open and edit "outbox-a4" with suffix " 5" and a fake clock installed
    When I fast-forward the clock past the outbox retry delay
    Then the save badge shows "Saved"
    And no conflict is shown

  Scenario: A5 Closing the tab warns while any file still has unsynced changes
    Given a cloud file named "outbox-a5.poentaro" is seeded for the signed-in account
    And every content-save request for "outbox-a5.poentaro" will fail with a 400 response
    And I open and edit "outbox-a5" with suffix " 5"
    When I close the page without letting the save land
    Then a beforeunload dialog is shown
