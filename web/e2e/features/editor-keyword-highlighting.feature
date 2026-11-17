Feature: Editor keyword highlighting
  The document is colored by anyx's tokenizer, the same way as any anyx
  document. Inside the score string, keywords are colored from the score's
  own lexer, so only words the parser recognizes get a keyword color.

  Background:
    Given the editor keyword-highlighting fixture is loaded

  Scenario Outline: Colors the document by its anyx token class
    Then the editor text '<text>' is colored as an anyx "<class>"

    Examples:
      | text      | class      |
      | ## Hymn   | comment    |
      | metadata  | identifier |
      | 30        | number     |
      | "Melody"  | string     |
      | MEL       | keyword    |
      | %         | operator   |

  Scenario Outline: Colors each recognized score keyword by its kind
    Then the editor text "<text>" is colored as a "<kind>" keyword

    Examples:
      | text  | kind          |
      | bpm   | directive-key |
      | break | directive-key |

  Scenario: Leaves the score's notes uncolored by the score string
    Then the editor text "1 2 3 4" is not colored as an anyx "string"
