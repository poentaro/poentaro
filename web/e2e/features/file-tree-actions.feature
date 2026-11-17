Feature: Creating, nesting and moving files and folders from the file actions menu

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.poentaro", "Hymns/Amazing.poentaro" and "Scratch.poentaro" and an empty folder "Empty" with "Scratch.poentaro" active

  Scenario: New creates the file next to the open file
    Given the file tree active file is "Hymns/Advent/O Come.poentaro"
    When the file tree app loads
    And I choose "New" from the file actions menu
    And I name the new file "untitled"
    Then the file switcher toolbar button shows the folder prefix "Hymns / Advent /" and the file name "untitled"

  Scenario: New at the top level when the open file is at the top level
    When the file tree app loads
    And I choose "New" from the file actions menu
    And I name the new file "untitled"
    Then the file switcher toolbar button shows no folder prefix and the file name "untitled"

  Scenario: New asks for a name before creating anything
    When the file tree app loads
    And I choose "New" from the file actions menu
    Then the new file dialog is open

  Scenario: New file in the file tree list keeps the list open behind the name prompt
    When the file tree app loads
    And I open the file tree list
    And I click "New file" in the file tree list
    Then the new file dialog is open
    And the file tree shows the file "Scratch"

  Scenario: A taken file name is rejected
    When the file tree app loads
    And I choose "New" from the file actions menu
    And I type "Scratch" in the new file dialog
    Then the new file dialog cannot be confirmed

  Scenario: New folder asks for a name before creating anything
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    Then the new folder dialog is open

  Scenario: Cancelling the new folder prompt creates nothing
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I cancel the new folder dialog
    And I open the file tree list
    Then the file tree shows no folder "New folder"

  Scenario: New folder is created next to the open file and shown expanded
    Given the file tree active file is "Hymns/Amazing.poentaro"
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I name the new folder "Psalms"
    And I open the file tree list
    Then the file tree shows the folder "Psalms" with a count of 0
    And the file tree shows the empty folder hint
    And the file switcher toolbar button shows the folder prefix "Hymns /" and the file name "Amazing"

  Scenario: The New folder button keeps the file browser open behind the prompt
    When the file tree app loads
    And I open the file tree list
    And I click "New folder" in the file tree list
    Then the new folder dialog is open
    And the file browser is open
    When I name the new folder "Psalms"
    Then the file browser is open
    And the file tree shows the folder "Psalms" with a count of 0

  Scenario: A taken folder name is rejected
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I type "Hymns" in the new folder dialog
    Then the new folder dialog cannot be confirmed

  Scenario: A folder can be created inside a folder
    Given the file tree active file is "Hymns/Advent/O Come.poentaro"
    When the file tree app loads
    And I choose "New folder" from the file actions menu
    And I name the new folder "Psalms"
    And I open the file tree list
    Then the file tree shows the folder "Psalms" with a count of 0
    And the file tree shows the folder "Psalms" nested 3 levels deep

  Scenario: Move to lists every folder and moves the open file into the chosen one
    When the file tree app loads
    And I choose "Move to…" from the file actions menu
    Then the move dialog offers the folders "Top level", "Empty", "Hymns" and "Advent"
    When I pick the folder "Advent" in the move dialog
    And I confirm the move dialog
    Then the file switcher toolbar button shows the folder prefix "Hymns / Advent /" and the file name "Scratch"

  Scenario: Move to the top level
    Given the file tree active file is "Hymns/Amazing.poentaro"
    When the file tree app loads
    And I choose "Move to…" from the file actions menu
    And I pick the folder "Top level" in the move dialog
    And I confirm the move dialog
    Then the file switcher toolbar button shows no folder prefix and the file name "Amazing"

  Scenario: Moving into a folder that already has a file of that name shows a clash message
    Given the file tree also has the file "Empty/Scratch.poentaro"
    When the file tree app loads
    And I choose "Move to…" from the file actions menu
    And I pick the folder "Empty" in the move dialog
    Then the move dialog says "already exists"
    And the move dialog cannot be confirmed
    When I pick the folder "Hymns" in the move dialog
    Then the move dialog shows no message
    And the move dialog can be confirmed

  Scenario: Demo files cannot be moved or renamed
    Given the file tree active file is "01-pitches.poentaro"
    When the file tree app loads
    And I open the file actions menu
    Then the file action "Move to…" is disabled
    And the file action "Rename" is disabled

  Scenario: Renaming with a slash moves the file and creates the folders
    When the file tree app loads
    And I open the file tree list
    And I rename the active file in the list to "Archive/Old/Scratch"
    Then the file switcher toolbar button shows the folder prefix "Archive / Old /" and the file name "Scratch"

  Scenario: Renaming from the menu shows the whole path and accepts a slash
    Given the file tree active file is "Hymns/Amazing.poentaro"
    When the file tree app loads
    And I choose "Rename" from the file actions menu
    Then the rename dialog holds "Hymns/Amazing"
    When I type "Elsewhere/Amazing" in the rename dialog and confirm
    Then the file switcher toolbar button shows the folder prefix "Elsewhere /" and the file name "Amazing"

  Scenario: Rename explains an empty segment, a clash and a relative segment instead of doing nothing
    Given the file tree also has the file "Empty/Scratch.poentaro"
    When the file tree app loads
    And I choose "Rename" from the file actions menu
    And I type "Hymns//Scratch" in the rename dialog
    Then the rename dialog says "empty segment"
    And the rename dialog cannot be confirmed
    When I type "../Scratch" in the rename dialog
    Then the rename dialog says "not allowed"
    When I type "Empty/Scratch" in the rename dialog
    Then the rename dialog says "already exists"
    When I type "Empty/Other" in the rename dialog
    Then the rename dialog shows no message
    And the rename dialog can be confirmed

  Scenario: The inline rename in the list also reports why a path is rejected
    When the file tree app loads
    And I open the file tree list
    And I type "Hymns//Scratch" in the active file's rename input in the list
    Then the list rename input says "empty segment"
