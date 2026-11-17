Feature: Cloud files whose names are slash paths keep their folders

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: A nested cloud file is listed under its folders after load
    Given a cloud file named "CloudPathsA/Advent/O Come.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I click the file tree folder "CloudPathsA"
    And I click the file tree folder "Advent"
    Then the file tree shows the file "O Come"

  Scenario: Renaming a cloud file to a nested path keeps the path after reload
    Given a cloud file named "cloud-paths-b.poentaro" is seeded for the signed-in account
    When the app loads the cloud-backed file list
    And I rename the active file to "CloudPathsB/Sub/renamed"
    Then "CloudPathsB/Sub/renamed.poentaro" exists on the server for the signed-in account
    When I reload the page and reopen the file list
    And I click the file tree folder "CloudPathsB"
    And I click the file tree folder "Sub"
    Then the file tree shows the file "renamed"

  Scenario: A new file created in a folder offline that collides on delivery stays in the folder
    Given the cloud is unreachable
    And I create a file named "CloudPathsC/song"
    And a cloud file named "CloudPathsC/song.poentaro" is seeded for the signed-in account
    When the cloud is reachable again
    Then "CloudPathsC/song 2.poentaro" exists on the server for the signed-in account
    And the file switcher toolbar button shows the folder prefix "CloudPathsC /" and the file name "song 2"

  Scenario: A rename that collides on delivery stays in the folder
    Given a cloud file named "CloudPathsD/orig.poentaro" is seeded for the signed-in account
    And the cloud is unreachable
    When I open the cloud file "CloudPathsD/orig" from the file list
    And I rename the active file in the list to "CloudPathsD/target"
    And a cloud file named "CloudPathsD/target.poentaro" is seeded for the signed-in account
    And the cloud is reachable again
    Then "CloudPathsD/target 2.poentaro" exists on the server for the signed-in account
    And the file switcher toolbar button shows the folder prefix "CloudPathsD /" and the file name "target 2"
