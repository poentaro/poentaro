Feature: The cloud refuses a path that would be both a file and a folder

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: Creating a file named like an existing folder is refused
    Given a cloud file named "ConflictA/song.poentaro" is seeded for the signed-in account
    When I try to create the cloud file "ConflictA"
    Then the cloud answers "file_folder_conflict"

  Scenario: Creating a file inside an existing file is refused
    Given a cloud file named "ConflictB" is seeded for the signed-in account
    When I try to create the cloud file "ConflictB/song.poentaro"
    Then the cloud answers "file_folder_conflict"

  Scenario: Renaming a file onto an existing folder is refused
    Given a cloud file named "ConflictC/song.poentaro" is seeded for the signed-in account
    And a cloud file named "conflict-c-other.poentaro" is seeded for the signed-in account
    When I try to rename the cloud file "conflict-c-other.poentaro" to "ConflictC"
    Then the cloud answers "file_folder_conflict"

  Scenario: A folder placeholder does not conflict with its own folder
    Given a cloud file named "ConflictD/song.poentaro" is seeded for the signed-in account
    When I try to create the cloud file "ConflictD/.keep"
    Then the cloud answers "ok"
