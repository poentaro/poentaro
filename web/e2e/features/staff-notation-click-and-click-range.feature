Feature: Click-and-click in the staff notation view selects exactly the clicked span

  # A property test: whichever two measures are clicked, and wherever on
  # their staves, the selection is exactly the measures from the first click
  # to the second (in either order) — never a measure more or less. The score
  # has the things that make Verovio's measure boxes spill past their
  # barlines: slurs and ties across barlines, ledger-line notes, lyrics and a
  # second staff.

  Scenario: Every click-and-click selects exactly the span between the two clicked measures
    Given a multi-system two-staff score with cross-barline slurs is shown in the staff notation view
    Then every click-and-click on two staff measures selects exactly the measures between them
