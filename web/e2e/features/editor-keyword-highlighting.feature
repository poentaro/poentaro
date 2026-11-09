Feature: Editor keyword highlighting
  Keywords are colored from the Rust lexers' highlight tokens, so only words
  the parser actually recognizes get a keyword color.

  Background:
    Given the editor keyword-highlighting fixture is loaded

  Scenario Outline: Colors each recognized keyword by its kind
    Then the editor text "<text>" is colored as a "<kind>" keyword

    Examples:
      | text    | kind           |
      | score   | section-header |
      | height  | metadata-key   |
      | notes   | part-kind      |
      | follow  | part-kind      |
      | bpm     | directive-key  |
      | break   | directive-key  |

  Scenario: Leaves unrecognized keys uncolored
    Then the editor text "colour" is not colored as a keyword
