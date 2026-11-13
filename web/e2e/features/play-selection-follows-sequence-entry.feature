Feature: Play Selection plays the selected sequence entry

  Scenario: Play Selection on a repeated entry plays that occurrence
    Given an "A, A WITH [w] 2" sequence score is loaded with playback requests recorded
    When I select sequence toolbar entry 2
    And I click Play Selection
    Then the last playback request plays sequence entry 1 as the sequence plays it
