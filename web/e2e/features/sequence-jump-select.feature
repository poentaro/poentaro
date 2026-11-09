Feature: Sequence jump select

  Background:
    Given a two-section source with a passthrough sequence "A, B" is loaded

  Scenario: Clicking the "A" sequence entry selects measures 0-1 and highlights lines 8-11 in Monaco
    When I click sequence entry button 0
    Then the sequence entry selected measure range is "0-1"
    And the sequence entry Monaco selection spans lines 8 to 11

  Scenario: Clicking the "B" sequence entry selects measures 2-3 and highlights lines 13-16 in Monaco
    When I click sequence entry button 1
    Then the sequence entry selected measure range is "2-3"
    And the sequence entry Monaco selection spans lines 13 to 16
