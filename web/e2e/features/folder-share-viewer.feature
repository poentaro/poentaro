Feature: Viewing and importing a shared folder

  # A shared folder opens read-only for anyone with the link. Every scenario
  # uses its own folder names because the signed-in account is shared by the
  # whole suite run.
  Background:
    Given clipboard permissions are granted
    And an account is signed in as "e2e-test-user@poentaro.test"

  Scenario: A viewer of a synced folder link sees a tree rooted at the folder and can open a nested file
    Given a cloud file named "ViewOne/Advent/o-come.poentaro" titled "O Come Emmanuel" is seeded for the signed-in account
    And a cloud file named "ViewOne/amazing.poentaro" titled "Amazing Grace" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewOne"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    Then the viewer banner says "Synced folder: ViewOne"
    And the viewer banner does not say "Shared by"
    And the viewer's preview contains "O Come Emmanuel"
    When the viewer opens the file list
    Then the viewer's file list shows the file "amazing"
    And the viewer's file list has no Demo folder
    And the viewer's file list offers no editing actions
    When the viewer expands the folder "Advent"
    Then the viewer's file list shows the file "o-come"
    When the viewer selects the file "amazing"
    Then the viewer's preview contains "Amazing Grace"
    And the viewer has no editor
    And the viewer's page URL has the synced share param

  Scenario: Changes by the owner appear after the viewer reloads
    Given a cloud file named "ViewTwo/keep.poentaro" titled "Keep Me" is seeded for the signed-in account
    And a cloud file named "ViewTwo/rename-me.poentaro" titled "Old Name Song" is seeded for the signed-in account
    And a cloud file named "ViewTwo/edit-me.poentaro" titled "Before Edit" is seeded for the signed-in account
    And a cloud file named "ViewTwo/delete-me.poentaro" titled "Doomed Song" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewTwo"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    And the viewer opens the file list
    Then the viewer's file list shows the file "delete-me"
    When the owner adds the cloud file "ViewTwo/brand-new.poentaro" titled "Brand New Song"
    And the owner renames the cloud file "ViewTwo/rename-me.poentaro" to "ViewTwo/renamed.poentaro"
    And the owner edits the cloud file "ViewTwo/edit-me.poentaro" to be titled "After Edit"
    And the owner deletes the cloud file "ViewTwo/delete-me.poentaro"
    Then the viewer's file list shows the file "delete-me"
    When the viewer reloads the page
    And the viewer opens the file list
    Then the viewer's file list shows the file "brand-new"
    And the viewer's file list shows the file "renamed"
    And the viewer's file list shows no file "rename-me"
    And the viewer's file list shows no file "delete-me"
    When the viewer selects the file "edit-me"
    Then the viewer's preview contains "After Edit"

  Scenario: A viewer sees that a stopped share is no longer shared, after reloading
    Given a cloud file named "ViewFour/song.poentaro" titled "Short Lived" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewFour"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    Then the viewer's preview contains "Short Lived"
    When the owner stops sharing the folder
    And the viewer reloads the page
    Then the viewer sees the "no longer shared" screen
    When the viewer clicks Discard
    Then the viewer's page URL has no synced share param
    And the viewer sees no shared folder banner
    And the viewer sees no shared viewer notice

  Scenario: A viewer can discard a synced folder without importing it
    Given a cloud file named "ViewNine/song.poentaro" titled "Not For Me" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewNine"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    Then the viewer's preview contains "Not For Me"
    When the viewer clicks Discard
    Then the viewer's page URL has no synced share param
    And the viewer sees no shared folder banner
    And the viewer's preview no longer contains "Not For Me"

  Scenario: Importing a synced folder copies it into the viewer's own files
    Given a cloud file named "ViewFive/Advent/o-come.poentaro" titled "Import One" is seeded for the signed-in account
    And a cloud file named "ViewFive/amazing.poentaro" titled "Import Two" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewFive"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog
    And the viewer imports the folder
    Then the viewer's page URL has no synced share param
    And the viewer sees no shared folder banner

  Scenario: A folder name already taken in the viewer's files becomes a numbered name
    Given a cloud file named "ViewSix/song.poentaro" titled "Clash Song" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewSix"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer who already has the local folder "ViewSix" opens the synced link of the share dialog
    And the viewer imports the folder
    Then the viewer's page URL has no synced share param
    And the viewer's folder list has the folder "ViewSix 2"

  Scenario: A viewer signed in to the cloud imports the folder into the cloud
    Given a cloud file named "ViewSeven/song.poentaro" titled "Cloud Copy" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewSeven"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a cloud viewer opens the synced link of the share dialog
    And the viewer imports the folder
    Then the cloud account holds the file "ViewSeven 2/song.poentaro"

  Scenario: A cloud viewer can import the folder only once their own files have loaded
    Given a cloud file named "ViewTen/song.poentaro" titled "Cloud Copy" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewTen"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a cloud viewer whose files are still loading opens the synced link of the share dialog
    Then the viewer cannot import the folder yet
    When the viewer's files finish loading
    And the viewer imports the folder
    Then the cloud account holds the file "ViewTen 2/song.poentaro"

  Scenario: A static folder link opens read-only and can be discarded
    Given the file tree is seeded with files "Static/Advent/a.poentaro", "Static/b.poentaro" and "c.poentaro" and an empty folder "Spare" with "c.poentaro" active
    When the file tree app loads
    And I open the share dialog of the folder "Static"
    And a viewer opens the static link of the share dialog
    Then the viewer banner says "Shared folder: Static"
    When the viewer opens the file list
    Then the viewer's file list shows the file "b"
    And the viewer's file list offers no editing actions
    When the viewer clicks Discard
    Then the viewer's page URL has no share hash
    And the viewer sees no shared folder banner

  Scenario: A static folder link can be imported
    Given the file tree is seeded with files "Static/Advent/a.poentaro", "Static/b.poentaro" and "c.poentaro" and an empty folder "Spare" with "c.poentaro" active
    When the file tree app loads
    And I open the share dialog of the folder "Static"
    And a viewer opens the static link of the share dialog
    And the viewer imports the folder
    Then the viewer's page URL has no share hash
    And the viewer's folder list has the folder "Static"

  Scenario: On a phone the folder picker and export stay on screen, ahead of the banner's Import button
    Given a cloud file named "ViewEight/song.poentaro" titled "Phone Song" is seeded for the signed-in account
    And the cloud app is open
    When I open the share dialog of the folder "ViewEight"
    And I open the synced tab of the share dialog
    And I start sync in the share dialog
    Then the share dialog shows a synced link
    When a viewer opens the synced link of the share dialog on a phone
    Then the viewer's folder picker is fully inside the screen
    And the viewer's export button is fully inside the screen
    And the viewer's import button is after the export button

  Scenario: On a phone a single synced file's export stays on screen, ahead of the banner's Import button
    Given the owner is signed in as "e2e-test-user@poentaro.test"
    And the file store is seeded with the synced score
    When the owner loads the app and clicks "Sync"
    Then the synced link is copied
    When a viewer opens the copied sync link in a new page
    And the viewer resizes to a phone screen
    Then the viewer's export button is fully inside the screen
    And the viewer's import button is after the export button
