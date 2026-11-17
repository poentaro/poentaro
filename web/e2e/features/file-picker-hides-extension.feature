Feature: File picker hides the redundant .poentaro extension

  Scenario: File picker hides the redundant .poentaro extension
    Given a local-storage-backed file "my song.poentaro" is seeded to test extension hiding
    When the app loads the seeded extension-hiding test file
    Then the file switcher trigger shows the extension-less name "my song"
    And the active file tab shows the extension-less name "my song"
    When I double-click the active tab name to enter rename mode, as seen in extension hiding
    Then the rename input starts with the extension-less value "my song"
    When I fill the rename input with "renamed" and press Enter, as seen in extension hiding
    Then the active file tab shows the extension-less name "renamed"
    And the file switcher trigger shows the extension-less name "renamed"
