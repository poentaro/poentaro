Feature: Play All follows the sequence order

  Scenario: Play All requests every sequence entry, not the written measure range
    Given a "B, A, B" sequence score is loaded with worker requests recorded
    And the Play All button becomes enabled once the soundfont loads
    When I click the Play All button
    Then the Play All audio request spans sequence entries 0 through 2
