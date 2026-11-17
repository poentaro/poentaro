Feature: Cloud outbox structural operations: file create and rename go through the local outbox

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: B1 A file created while offline appears at once and reaches the server later
    Given the cloud is unreachable
    When I create a file named "outbox-b1"
    Then "outbox-b1.poentaro" is listed in the file switcher
    When the cloud is reachable again
    Then "outbox-b1.poentaro" exists on the server for the signed-in account

  Scenario: B2 A rename made while offline is applied once the cloud returns
    Given a cloud file named "outbox-b2.poentaro" is seeded for the signed-in account
    And the cloud is unreachable
    When I rename "outbox-b2" to "outbox-b2-renamed"
    Then "outbox-b2-renamed.poentaro" is listed in the file switcher
    When the cloud is reachable again
    Then "outbox-b2-renamed.poentaro" exists on the server for the signed-in account

  Scenario: B3 A name taken on the server at delivery time renames the local file
    Given the cloud is unreachable
    And I create a file named "outbox-b3"
    And a cloud file named "outbox-b3.poentaro" is seeded for the signed-in account
    When the cloud is reachable again
    Then a file named "outbox-b3 2.poentaro" is listed in the file switcher
