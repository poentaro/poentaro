Feature: Cloud outbox panel: queued work is visible and every halt has a way out

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: C1 The panel lists a waiting file with a Retry now button
    Given a cloud file named "outbox-c1.poentaro" is seeded for the signed-in account
    And every content-save request will be aborted as a network failure
    And I open and edit "outbox-c1" with suffix " 5"
    When I open the sync panel
    Then the sync panel lists "outbox-c1.poentaro" as "waiting to sync"
    When content-save requests are allowed again
    And I click "Retry now" in the sync panel
    Then the content save lands for "outbox-c1.poentaro" containing "1 2 3 4 5"

  Scenario: C2 A permanently failing file offers download and discard
    Given a cloud file named "outbox-c2.poentaro" is seeded for the signed-in account
    And every content-save request for "outbox-c2.poentaro" will fail with a 400 response
    And I open and edit "outbox-c2" with suffix " 5"
    When I open the sync panel
    Then the sync panel lists "outbox-c2.poentaro" as "couldn't sync"
    When I click "Download my copy" in the sync panel
    Then a file named "outbox-c2.poentaro" containing "1 2 3 4 5" is downloaded

  Scenario: C3 Discarding reverts the file to the server version after confirmation
    Given a cloud file named "outbox-c3.poentaro" is seeded for the signed-in account
    And every content-save request for "outbox-c3.poentaro" will fail with a 400 response
    And I open and edit "outbox-c3" with suffix " 5"
    When I open the sync panel
    And I click "Discard" in the sync panel
    And I confirm "Discard my changes"
    Then the editor contains "1 2 3 4"
    And the editor does not contain "1 2 3 4 5"
    And the save badge shows "Saved"

  Scenario: C4 The details layer shows the attempt history
    Given a cloud file named "outbox-c4.poentaro" is seeded for the signed-in account
    And every content-save request for "outbox-c4.poentaro" will fail with a 400 response
    And I open and edit "outbox-c4" with suffix " 5"
    When I open the sync panel
    And I expand the details for "outbox-c4.poentaro"
    Then the details show 1 queued "SaveContent" message
    And the details show an attempt with outcome "400"
