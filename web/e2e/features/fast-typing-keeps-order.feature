Feature: Fast typing keeps characters in order

  Scenario: Typing a word quickly lands every character where it was typed
    Given the score:
      ```
      parts ["A": notes],

      score """
          [A] 1 2 3 4 | 5 6 7 1' | 1 2 3 4 | 5 6 7 1'
          """,
      ```
    When I quickly type "voice voice voice voice voice voice voice voice" on a new line at the end of the editor
    Then the editor ends with "voice voice voice voice voice voice voice voice"
