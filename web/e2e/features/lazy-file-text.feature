Feature: Cloud files load their text only when opened

  Background:
    Given an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: Start-up reads only the opened file's text
    Given a cloud file named "lazy1-first.poentaro" is seeded for the signed-in account with the text "title: First"
    And a cloud file named "lazy1-second.poentaro" is seeded for the signed-in account with the text "title: Second"
    And a cloud file named "lazy1-third.poentaro" is seeded for the signed-in account with the text "title: Third"
    When the app loads with "lazy1-first.poentaro" open
    Then the editor contains "title: First"
    And the file list shows "lazy1-first", "lazy1-second" and "lazy1-third"
    And the server was asked for the text of "lazy1-first.poentaro" only

  Scenario: Opening another file fetches its text
    Given a cloud file named "lazy2-first.poentaro" is seeded for the signed-in account with the text "title: First"
    And a cloud file named "lazy2-second.poentaro" is seeded for the signed-in account with the text "title: Second"
    And the app loads with "lazy2-first.poentaro" open
    When the user opens "lazy2-second.poentaro" from the file list
    Then the editor contains "title: Second"
    And the server was asked for the text of "lazy2-second.poentaro" once

  Scenario: A file whose text is still loading is not shown as empty
    Given a cloud file named "lazy3-first.poentaro" is seeded for the signed-in account with the text "title: First"
    And a cloud file named "lazy3-second.poentaro" is seeded for the signed-in account with the text "title: Second"
    And the app loads with "lazy3-first.poentaro" open
    And reading the text of "lazy3-second.poentaro" is delayed by 3 seconds
    When the user opens "lazy3-second.poentaro" from the file list
    Then the file switcher shows a loading spinner
    And the editor is read-only
    When the text of "lazy3-second.poentaro" arrives
    Then the editor contains "title: Second"
    And the editor is editable

  Scenario: An unchanged file reopens from the local cache
    Given a cloud file named "lazy4-first.poentaro" is seeded for the signed-in account with the text "title: First"
    And a cloud file named "lazy4-second.poentaro" is seeded for the signed-in account with the text "title: Second"
    And the app loads with "lazy4-first.poentaro" open
    And the user has opened "lazy4-second.poentaro" from the file list
    When the app reloads with "lazy4-second.poentaro" open
    Then the editor contains "title: Second"
    And the server was not asked for the text of "lazy4-second.poentaro" after the reload

  Scenario: A file changed elsewhere is read again
    Given a cloud file named "lazy5-first.poentaro" is seeded for the signed-in account with the text "title: First"
    And a cloud file named "lazy5-second.poentaro" is seeded for the signed-in account with the text "title: Second"
    And the app loads with "lazy5-first.poentaro" open
    And the user has opened "lazy5-second.poentaro" from the file list
    And "lazy5-second.poentaro" is changed on the server to "title: Second v2"
    When the app reloads with "lazy5-second.poentaro" open
    Then the editor contains "title: Second v2"

  Scenario: An unsent edit survives a reload
    Given a cloud file named "lazy7-first.poentaro" is seeded for the signed-in account
    And every content-save request will be aborted as a network failure
    And I open and edit "lazy7-first" with suffix " 5"
    Then the save badge shows "1 change waiting"
    When the app reloads with "lazy7-first.poentaro" open
    Then the editor contains "1 2 3 4 5"

  Scenario: A duplicate of a file opened from the list carries its text
    Given a cloud file named "lazy8-first.poentaro" is seeded for the signed-in account with the text "title: First"
    And a cloud file named "lazy8-second.poentaro" is seeded for the signed-in account with the text "title: Second"
    And the app loads with "lazy8-first.poentaro" open
    And the user has opened "lazy8-second.poentaro" from the file list
    When the user duplicates the open file
    Then the editor contains "title: Second"
