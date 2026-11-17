Feature: A folder move is delivered to the cloud as one all-or-nothing batch

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: Moving a folder offline moves all its files together on reconnect
    Given a cloud file named "CloudBatchA/one.poentaro" is seeded for the signed-in account
    And a cloud file named "CloudBatchA/two.poentaro" is seeded for the signed-in account
    And I record the batch renames the cloud answers
    And the cloud is unreachable
    When I open the file tree list
    And I choose "Rename" from the actions menu of the folder "CloudBatchA"
    And I type "CloudBatchA2" in the rename dialog and confirm
    And the cloud is reachable again
    Then "CloudBatchA2/one.poentaro" exists on the server for the signed-in account
    And "CloudBatchA2/two.poentaro" exists on the server for the signed-in account
    And "CloudBatchA/one.poentaro" does not exist on the server for the signed-in account
    And "CloudBatchA/two.poentaro" does not exist on the server for the signed-in account
    And the cloud answered the batch renames with "204"

  Scenario: A collision on one file adjusts the batch instead of half-applying it
    Given a cloud file named "CloudBatchB/one.poentaro" is seeded for the signed-in account
    And a cloud file named "CloudBatchB/two.poentaro" is seeded for the signed-in account
    And I record the batch renames the cloud answers
    And the cloud is unreachable
    When I open the file tree list
    And I choose "Rename" from the actions menu of the folder "CloudBatchB"
    And I type "CloudBatchB2" in the rename dialog and confirm
    And a cloud file named "CloudBatchB2/two.poentaro" is seeded for the signed-in account
    And the cloud is reachable again
    Then "CloudBatchB2/one.poentaro" exists on the server for the signed-in account
    And "CloudBatchB2/two 2.poentaro" exists on the server for the signed-in account
    And "CloudBatchB/one.poentaro" does not exist on the server for the signed-in account
    And "CloudBatchB/two.poentaro" does not exist on the server for the signed-in account
    And the cloud answered the batch renames with "409, 204"

  Scenario: A folder move still waiting to be delivered survives a reload
    Given a cloud file named "CloudBatchC/one.poentaro" is seeded for the signed-in account
    And a cloud file named "CloudBatchC/two.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And folder moves cannot reach the cloud
    And I choose "Rename" from the actions menu of the folder "CloudBatchC"
    And I type "CloudBatchC2" in the rename dialog and confirm
    And I reload the page and reopen the file list
    Then the file tree shows the folder "CloudBatchC2"
    And the file tree does not show the folder "CloudBatchC"
    When folder moves can reach the cloud again
    Then "CloudBatchC2/one.poentaro" exists on the server for the signed-in account
    And "CloudBatchC2/two.poentaro" exists on the server for the signed-in account
