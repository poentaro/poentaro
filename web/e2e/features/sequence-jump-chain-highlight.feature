Feature: Sequence-chain drag selection highlights only its own measures

  Scenario: Dragging "C" to the repeated "A" selects both entries in the editor and highlights both measures in the preview, excluding "B"
    Given a sequence chain "A, B, C, A" over sections "A", "B", "C" is seeded for chain highlight
    When the app loads with the chain-highlight sequence toolbar ready
    When I drag from the "C" sequence entry to the repeated "A" sequence entry
    Then the editor selects exactly "C" and the repeated "A" as disjoint ranges
    And the preview highlights exactly the "C" and "A" measures, excluding "B"

  Scenario Outline: Expanded order highlights the clicked occurrence of a repeated section in <notation>
    Given a sequence chain "A, B, C, A" over sections "A", "B", "C" is seeded for chain highlight
    When the app loads with the chain-highlight sequence toolbar ready
    And the chain-highlight preview is set to Expanded order with "<notation>" notation
    And I click the final sequence entry in the chain-highlight toolbar
    Then only the final occurrence of "A" is highlighted in the chain-highlight "<notation>" preview

    Examples:
      | notation |
      | Numbered |
      | Violin   |
