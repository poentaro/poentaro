Feature: Folder rows have their own actions menu, folders can be deleted to the bin and restored

  Background:
    Given the file tree is seeded with files "Hymns/Advent/O Come.poentaro", "Hymns/Amazing.poentaro" and "Scratch.poentaro" and an empty folder "Empty" with "Scratch.poentaro" active

  Scenario: A folder row has a menu with its own actions
    When the file tree app loads
    And I open the file tree list
    And I open the actions menu of the folder "Hymns"
    Then the folder menu offers "New file here"
    And the folder menu offers "New folder here"
    And the folder menu offers "Import here"
    And the folder menu offers "Rename"
    And the folder menu offers "Move to…"
    And the folder menu offers "Delete folder"
    And the folder menu offers "Share folder"

  Scenario: The read-only Demo row has no menu
    When the file tree app loads
    And I open the file tree list
    Then the folder "Demo" has no actions menu

  Scenario: New file here creates the file inside that folder
    When the file tree app loads
    And I open the file tree list
    And I choose "New file here" from the actions menu of the folder "Hymns"
    And I name the new file "untitled"
    Then the file switcher toolbar button shows the folder prefix "Hymns /" and the file name "untitled"

  Scenario: New folder here creates a folder inside that folder and shows it
    When the file tree app loads
    And I open the file tree list
    And I choose "New folder here" from the actions menu of the folder "Empty"
    And I name the new folder "Psalms"
    And I open the file tree list
    Then the file tree shows the folder "Psalms" nested 2 levels deep
    And the file tree shows the empty folder hint

  Scenario: The file browser stays open behind the New folder here prompt
    When the file tree app loads
    And I open the file tree list
    And I choose "New folder here" from the actions menu of the folder "Empty"
    Then the new folder dialog is open
    And the file browser is open
    When I name the new folder "Psalms"
    Then the file browser is open
    And the file tree shows the folder "Psalms" nested 2 levels deep

  Scenario: The file browser stays open behind the Move prompt
    When the file tree app loads
    And I open the file tree list
    And I choose "Move to…" from the actions menu of the folder "Hymns"
    Then the file browser is open
    When I cancel the move dialog
    Then the file browser is open

  Scenario: The file browser stays open behind the Delete folder prompt
    When the file tree app loads
    And I open the file tree list
    And I choose "Delete folder" from the actions menu of the folder "Hymns"
    Then the file browser is open
    When I confirm the delete folder dialog
    Then the file browser is open
    And the file tree does not show the folder "Hymns"

  Scenario: The file browser stays open behind the Share folder dialog
    When the file tree app loads
    And I open the share dialog of the folder "Hymns"
    Then the share dialog is titled 'Share folder "Hymns"'
    And the file browser is open
    When I close the share dialog
    Then the file browser is open

  Scenario: Import here puts the recovered file inside that folder
    When the file tree app loads
    And I export the active file as a PDF
    And I open the file tree list
    And I import the exported PDF file into the folder "Empty" as "Recovered"
    Then the file switcher toolbar button shows the folder prefix "Empty /" and the file name "Recovered"

  Scenario: Renaming a folder renames everything inside it
    When the file tree app loads
    And I open the file tree list
    And I choose "Rename" from the actions menu of the folder "Hymns"
    Then the rename dialog holds "Hymns"
    When I type "Songs" in the rename dialog and confirm
    Then the file browser is open
    And the file tree shows the folder "Songs" with a count of 2
    And the file tree does not show the folder "Hymns"

  Scenario: Renaming a folder with a slash moves it
    When the file tree app loads
    And I open the file tree list
    And I choose "Rename" from the actions menu of the folder "Hymns"
    And I type "Empty/Hymns" in the rename dialog and confirm
    When I open the file tree list
    Then the file tree shows the folder "Empty" with a count of 2
    And the file tree shows the folder "Hymns" nested 2 levels deep

  Scenario: Renaming a folder onto an existing folder explains the clash and changes nothing
    When the file tree app loads
    And I open the file tree list
    And I choose "Rename" from the actions menu of the folder "Hymns"
    And I type "Empty" in the rename dialog
    Then the rename dialog says "already exists"
    And the rename dialog cannot be confirmed
    When I type "Hymns//Songs" in the rename dialog
    Then the rename dialog says "empty segment"
    When I type "Songs" in the rename dialog
    Then the rename dialog shows no message
    When I cancel the rename dialog
    And I open the file tree list
    Then the file tree shows the folder "Hymns" with a count of 2
    And the file tree shows the folder "Empty" with a count of 0

  Scenario: Move to for a folder does not offer the folder itself or anything inside it
    When the file tree app loads
    And I open the file tree list
    And I choose "Move to…" from the actions menu of the folder "Hymns"
    Then the move dialog offers only the folders "Top level" and "Empty"

  Scenario: Moving a folder into another folder
    When the file tree app loads
    And I open the file tree list
    And I choose "Move to…" from the actions menu of the folder "Hymns"
    And I pick the folder "Empty" in the move dialog
    And I confirm the move dialog
    When I open the file tree list
    Then the file tree shows the folder "Empty" with a count of 2
    And the file tree shows the folder "Hymns" nested 2 levels deep

  Scenario: Moving a folder onto a folder of the same name explains the clash and changes nothing
    Given the file tree also has the file "Empty/Hymns/Old.poentaro"
    When the file tree app loads
    And I open the file tree list
    And I choose "Move to…" from the actions menu of the folder "Hymns"
    And I pick the folder "Empty" in the move dialog
    Then the move dialog says "already exists"
    And the move dialog cannot be confirmed
    When I cancel the move dialog
    And I open the file tree list
    Then the file tree shows the folder "Hymns" with a count of 2
    And the file tree shows the folder "Empty" with a count of 1

  Scenario: Deleting a folder asks first and says how many files go to the bin
    When the file tree app loads
    And I open the file tree list
    And I choose "Delete folder" from the actions menu of the folder "Hymns"
    Then the delete folder dialog says "2 files"
    When I cancel the delete folder dialog
    And I open the file tree list
    Then the file tree shows the folder "Hymns" with a count of 2

  Scenario: Deleting a folder moves its files to the bin with their original locations
    When the file tree app loads
    And I open the file tree list
    And I choose "Delete folder" from the actions menu of the folder "Hymns"
    And I confirm the delete folder dialog
    And I open the file tree list
    Then the file tree does not show the folder "Hymns"
    When I open the bin
    Then the bin lists "O Come" located in "Hymns/Advent"
    And the bin lists "Amazing" located in "Hymns"

  Scenario: Restoring puts a file back at its original path
    When the file tree app loads
    And I open the file tree list
    And I choose "Delete folder" from the actions menu of the folder "Hymns"
    And I confirm the delete folder dialog
    And I open the bin
    And I restore "Hymns/Amazing.poentaro" from the bin
    Then the file switcher toolbar button shows the folder prefix "Hymns /" and the file name "Amazing"

  Scenario: Restoring recreates a folder that is gone
    When the file tree app loads
    And I open the file tree list
    And I choose "Delete folder" from the actions menu of the folder "Hymns"
    And I confirm the delete folder dialog
    And I open the bin
    And I restore "Hymns/Advent/O Come.poentaro" from the bin
    Then the file switcher toolbar button shows the folder prefix "Hymns / Advent /" and the file name "O Come"

  Scenario: Restoring gets a unique name when the original path is taken
    Given the file tree bin also holds "Hymns/Amazing.poentaro"
    When the file tree app loads
    And I open the bin
    And I restore "Hymns/Amazing.poentaro" from the bin
    Then the file switcher toolbar button shows the folder prefix "Hymns /" and the file name "Amazing 2"
