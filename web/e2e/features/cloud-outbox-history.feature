Feature: Cloud outbox history: every message, sent or not, can be inspected and searched

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: H1 A delivered edit stays in the history
    Given a cloud file named "outbox-h1.poentaro" is seeded for the signed-in account
    And I open and edit "outbox-h1" with suffix " 5"
    And the save badge shows "Saved"
    When I open the sync panel
    Then the outbox history lists "outbox-h1.poentaro" as "Sent"
    And the sync panel is not shown

  Scenario: H2 Searching finds text inside a payload and highlights it
    Given a cloud file named "outbox-h2.poentaro" is seeded for the signed-in account
    And I open and edit "outbox-h2" with suffix " 5"
    And the save badge shows "Saved"
    When I open the sync panel
    And I search the outbox history for "1 2 3 4 5"
    Then the outbox history shows "1 2 3 4 5" highlighted in the payload of "outbox-h2.poentaro"

  Scenario: H3 A failing file is listed with its attempt and can be filtered as a problem
    Given a cloud file named "outbox-h3.poentaro" is seeded for the signed-in account
    And every content-save request for "outbox-h3.poentaro" will fail with a 400 response
    And I open and edit "outbox-h3" with suffix " 5"
    When I open the sync panel
    And I filter the outbox history by status "Any problem"
    Then the outbox history lists "outbox-h3.poentaro" as "Couldn't sync"
    When I expand "outbox-h3.poentaro" in the outbox history
    Then the outbox history row for "outbox-h3.poentaro" shows an attempt with outcome "400"
